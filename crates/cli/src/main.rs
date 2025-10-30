//! Hemmer Provider Generator CLI
//!
//! Command-line interface for generating Hemmer providers from cloud SDKs.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use colored::*;
use hemmer_provider_generator_generator::{ProviderGenerator, UnifiedProviderGenerator};
use hemmer_provider_generator_parser::{
    DiscoveryParser, OpenApiParser, ProtobufParser, SmithyParser,
};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "hemmer-provider-generator")]
#[command(version, about = "Generate Hemmer providers from cloud provider SDKs", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse a spec file and display the extracted service definition
    #[command(after_help = "EXAMPLES:\n  \
        # Parse AWS Smithy spec\n  \
        hemmer-provider-generator parse --spec s3.json --format smithy\n\n  \
        # Parse GCP Discovery document\n  \
        hemmer-provider-generator parse --spec storage-v1.json --format discovery\n\n  \
        # Auto-detect format\n  \
        hemmer-provider-generator parse --spec kubernetes-api.json")]
    Parse {
        /// Path to the spec file
        #[arg(short, long)]
        spec: PathBuf,

        /// Spec format (auto-detected if not specified)
        #[arg(short, long)]
        format: Option<SpecFormat>,

        /// Service name
        #[arg(long)]
        service: Option<String>,

        /// API version
        #[arg(long, default_value = "v1")]
        version: String,
    },

    /// Generate a provider from a single spec file
    #[command(after_help = "EXAMPLES:\n  \
        # Generate from AWS Smithy spec\n  \
        hemmer-provider-generator generate \\\n    \
        --spec s3.json \\\n    \
        --format smithy \\\n    \
        --service s3 \\\n    \
        --output ./providers/aws-s3\n\n  \
        # Generate from GCP Discovery document\n  \
        hemmer-provider-generator generate \\\n    \
        --spec storage-v1.json \\\n    \
        --format discovery \\\n    \
        --service storage \\\n    \
        --output ./providers/gcp-storage\n\n  \
        # Generate from Protobuf FileDescriptorSet\n  \
        hemmer-provider-generator generate \\\n    \
        --spec service.pb \\\n    \
        --format protobuf \\\n    \
        --service storage \\\n    \
        --output ./providers/grpc-storage")]
    Generate {
        /// Path to the spec file
        #[arg(short, long)]
        spec: PathBuf,

        /// Spec format (auto-detected if not specified)
        #[arg(short, long)]
        format: Option<SpecFormat>,

        /// Service name (required for some formats)
        #[arg(long)]
        service: String,

        /// API version
        #[arg(long, default_value = "v1")]
        version: String,

        /// Output directory
        #[arg(short, long, default_value = "./output")]
        output: PathBuf,
    },

    /// Generate a unified provider from multiple spec files
    #[command(after_help = "EXAMPLES:\n  \
        # Generate unified AWS provider with S3 and DynamoDB\n  \
        hemmer-provider-generator generate-unified \\\n    \
        --provider aws \\\n    \
        --specs s3.json,dynamodb.json \\\n    \
        --format smithy \\\n    \
        --output ./provider-aws\n\n  \
        # Scan entire directory for specs\n  \
        hemmer-provider-generator generate-unified \\\n    \
        --provider aws \\\n    \
        --spec-dir ./aws-sdk/models/ \\\n    \
        --format smithy \\\n    \
        --output ./provider-aws\n\n  \
        # Filter services by pattern\n  \
        hemmer-provider-generator generate-unified \\\n    \
        --provider aws \\\n    \
        --spec-dir ./aws-sdk/models/ \\\n    \
        --filter s3,dynamodb,ec2 \\\n    \
        --output ./provider-aws")]
    GenerateUnified {
        /// Provider name (e.g., "aws", "gcp", "azure")
        #[arg(short, long)]
        provider: String,

        /// Comma-separated list of spec file paths
        #[arg(short, long, value_delimiter = ',', conflicts_with = "spec_dir")]
        specs: Option<Vec<PathBuf>>,

        /// Directory containing spec files (alternative to --specs)
        #[arg(long, conflicts_with = "specs")]
        spec_dir: Option<PathBuf>,

        /// Spec format (auto-detected if not specified)
        #[arg(short, long)]
        format: Option<SpecFormat>,

        /// Comma-separated list of service names to include (filters discovered specs)
        #[arg(long, value_delimiter = ',')]
        filter: Option<Vec<String>>,

        /// Comma-separated list of explicit service names (auto-detected if not specified)
        #[arg(long, value_delimiter = ',')]
        services: Option<Vec<String>>,

        /// API version
        #[arg(long, default_value = "v1")]
        version: String,

        /// Output directory
        #[arg(short, long, default_value = "./output")]
        output: PathBuf,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum SpecFormat {
    /// AWS Smithy JSON AST
    Smithy,
    /// OpenAPI 3.0 (Kubernetes, Azure)
    Openapi,
    /// Google Discovery Document
    Discovery,
    /// Protocol Buffer FileDescriptorSet
    Protobuf,
}

impl std::fmt::Display for SpecFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpecFormat::Smithy => write!(f, "Smithy"),
            SpecFormat::Openapi => write!(f, "OpenAPI"),
            SpecFormat::Discovery => write!(f, "Discovery"),
            SpecFormat::Protobuf => write!(f, "Protobuf"),
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        println!("{} Verbose mode enabled", "→".cyan());
    }

    match cli.command {
        Commands::Parse {
            spec,
            format,
            service,
            version,
        } => {
            parse_command(
                spec.as_path(),
                format,
                service.as_deref(),
                &version,
                cli.verbose,
            )?;
        }
        Commands::Generate {
            spec,
            format,
            service,
            version,
            output,
        } => {
            generate_command(
                spec.as_path(),
                format,
                &service,
                &version,
                output.as_path(),
                cli.verbose,
            )?;
        }

        Commands::GenerateUnified {
            provider,
            specs,
            spec_dir,
            format,
            filter,
            services,
            version,
            output,
        } => {
            generate_unified_command(UnifiedConfig {
                provider_name: &provider,
                spec_paths: specs.as_deref(),
                spec_dir: spec_dir.as_deref(),
                format,
                filter: filter.as_deref(),
                service_names: services.as_deref(),
                version: &version,
                output: output.as_path(),
                verbose: cli.verbose,
            })?;
        }
    }

    Ok(())
}

fn parse_command(
    spec_path: &Path,
    format: Option<SpecFormat>,
    service_name: Option<&str>,
    version: &str,
    verbose: bool,
) -> Result<()> {
    println!("{} Parsing spec file: {}", "→".cyan(), spec_path.display());

    // Detect format if not specified
    let detected_format = format.unwrap_or_else(|| {
        let detected = detect_format(spec_path);
        println!(
            "{} Auto-detected format: {}",
            "→".cyan(),
            detected.to_string().yellow()
        );
        detected
    });

    // Infer service name from filename if not provided
    let service = service_name
        .map(String::from)
        .or_else(|| infer_service_name(spec_path))
        .unwrap_or_else(|| "unknown".to_string());

    if verbose {
        println!("  Format: {}", detected_format);
        println!("  Service: {}", service);
        println!("  Version: {}", version);
    }

    // Parse based on format
    let service_def = match detected_format {
        SpecFormat::Smithy => {
            println!("{} Using Smithy parser", "→".cyan());
            let parser = SmithyParser::from_file(spec_path, &service, version)
                .context("Failed to load Smithy spec")?;
            parser.parse().context("Failed to parse Smithy spec")?
        }
        SpecFormat::Openapi => {
            println!("{} Using OpenAPI parser", "→".cyan());
            let parser = OpenApiParser::from_file(spec_path, &service, version)
                .context("Failed to load OpenAPI spec")?;
            parser.parse().context("Failed to parse OpenAPI spec")?
        }
        SpecFormat::Discovery => {
            println!("{} Using Discovery parser", "→".cyan());
            let parser = DiscoveryParser::from_file(spec_path, &service, version)
                .context("Failed to load Discovery doc")?;
            parser.parse().context("Failed to parse Discovery doc")?
        }
        SpecFormat::Protobuf => {
            println!("{} Using Protobuf parser", "→".cyan());
            let parser = ProtobufParser::from_file(spec_path, &service, version)
                .context("Failed to load Protobuf FileDescriptorSet")?;
            parser
                .parse()
                .context("Failed to parse Protobuf FileDescriptorSet")?
        }
    };

    // Display results
    println!("\n{}", "✓ Parse successful!".green().bold());
    println!("\n{}", "Service Definition:".bold());
    println!("  Name: {}", service_def.name.yellow());
    println!("  Version: {}", service_def.sdk_version.yellow());
    println!("  Provider: {:?}", service_def.provider);
    println!("  Resources: {}", service_def.resources.len());

    if verbose {
        println!("\n{}", "Resources:".bold());
        for resource in &service_def.resources {
            println!("  • {} ({})", resource.name.cyan(), {
                let mut ops = Vec::new();
                if resource.operations.create.is_some() {
                    ops.push("C");
                }
                if resource.operations.read.is_some() {
                    ops.push("R");
                }
                if resource.operations.update.is_some() {
                    ops.push("U");
                }
                if resource.operations.delete.is_some() {
                    ops.push("D");
                }
                ops.join("")
            });
            println!("    Fields: {}", resource.fields.len());
            println!("    Outputs: {}", resource.outputs.len());
        }
    }

    Ok(())
}

fn generate_command(
    spec_path: &Path,
    format: Option<SpecFormat>,
    service_name: &str,
    version: &str,
    output: &Path,
    verbose: bool,
) -> Result<()> {
    println!(
        "{} Generating provider from: {}",
        "→".cyan(),
        spec_path.display()
    );

    // Detect format if not specified
    let detected_format = format.unwrap_or_else(|| {
        let detected = detect_format(spec_path);
        println!(
            "{} Auto-detected format: {}",
            "→".cyan(),
            detected.to_string().yellow()
        );
        detected
    });

    if verbose {
        println!("  Format: {}", detected_format);
        println!("  Service: {}", service_name);
        println!("  Version: {}", version);
        println!("  Output: {}", output.display());
    }

    // Parse based on format
    println!("{} Parsing spec...", "→".cyan());
    let service_def = match detected_format {
        SpecFormat::Smithy => {
            let parser = SmithyParser::from_file(spec_path, service_name, version)
                .context("Failed to load Smithy spec")?;
            parser.parse().context("Failed to parse Smithy spec")?
        }
        SpecFormat::Openapi => {
            let parser = OpenApiParser::from_file(spec_path, service_name, version)
                .context("Failed to load OpenAPI spec")?;
            parser.parse().context("Failed to parse OpenAPI spec")?
        }
        SpecFormat::Discovery => {
            let parser = DiscoveryParser::from_file(spec_path, service_name, version)
                .context("Failed to load Discovery doc")?;
            parser.parse().context("Failed to parse Discovery doc")?
        }
        SpecFormat::Protobuf => {
            let parser = ProtobufParser::from_file(spec_path, service_name, version)
                .context("Failed to load Protobuf FileDescriptorSet")?;
            parser
                .parse()
                .context("Failed to parse Protobuf FileDescriptorSet")?
        }
    };

    println!(
        "{} Parsed {} resources",
        "✓".green(),
        service_def.resources.len()
    );

    // Generate provider
    println!("{} Generating provider files...", "→".cyan());
    let generator = ProviderGenerator::new(service_def).context("Failed to create generator")?;
    generator
        .generate_to_directory(output)
        .context("Failed to generate provider")?;

    println!("\n{}", "✓ Generation complete!".green().bold());
    println!("\n{}", "Generated files:".bold());
    println!("  📄 {}/provider.k", output.display());
    println!("  📄 {}/src/lib.rs", output.display());
    println!("  📄 {}/Cargo.toml", output.display());
    println!("\n{}", "Next steps:".bold());
    println!("  1. Review generated files in {}", output.display());
    println!(
        "  2. Build provider: cd {} && cargo build",
        output.display()
    );
    println!("  3. Install in hemmer provider directory");

    Ok(())
}

/// Configuration for unified provider generation
struct UnifiedConfig<'a> {
    provider_name: &'a str,
    spec_paths: Option<&'a [PathBuf]>,
    spec_dir: Option<&'a Path>,
    format: Option<SpecFormat>,
    filter: Option<&'a [String]>,
    service_names: Option<&'a [String]>,
    version: &'a str,
    output: &'a Path,
    verbose: bool,
}

fn generate_unified_command(config: UnifiedConfig) -> Result<()> {
    use hemmer_provider_generator_common::{Provider, ProviderDefinition};

    // Discover spec files
    let discovered_specs: Vec<PathBuf> = if let Some(dir) = config.spec_dir {
        println!(
            "{} Scanning directory for specs: {}",
            "→".cyan(),
            dir.display()
        );
        discover_specs(dir, config.format, config.filter, config.verbose)?
    } else if let Some(paths) = config.spec_paths {
        paths.to_vec()
    } else {
        anyhow::bail!("Either --specs or --spec-dir must be provided");
    };

    if discovered_specs.is_empty() {
        anyhow::bail!("No spec files found");
    }

    println!(
        "{} Generating unified {} provider from {} specs",
        "→".cyan(),
        config.provider_name.yellow(),
        discovered_specs.len()
    );

    // Parse provider enum from string
    let provider = match config.provider_name.to_lowercase().as_str() {
        "aws" => Provider::Aws,
        "gcp" => Provider::Gcp,
        "azure" => Provider::Azure,
        "kubernetes" | "k8s" => Provider::Kubernetes,
        _ => anyhow::bail!("Unknown provider: {}", config.provider_name),
    };

    // Parse all specs
    let mut services = Vec::new();
    let mut skipped = 0;
    for (i, spec_path) in discovered_specs.iter().enumerate() {
        println!(
            "{} Parsing spec {}/{}: {}",
            "→".cyan(),
            i + 1,
            discovered_specs.len(),
            spec_path.display()
        );

        // Detect format if not specified
        let detected_format = config.format.unwrap_or_else(|| detect_format(spec_path));

        // Get service name
        let inferred_name = infer_service_name(spec_path);
        let service_name = config
            .service_names
            .and_then(|names| names.get(i).map(String::as_str))
            .or(inferred_name.as_deref())
            .unwrap_or("unknown");

        if config.verbose {
            println!("  Format: {}", detected_format);
            println!("  Service: {}", service_name);
        }

        // Parse based on format - wrap in error handling to skip failures
        let service_def_result: Result<_> = (|| {
            let service_def = match detected_format {
                SpecFormat::Smithy => {
                    let parser =
                        SmithyParser::from_file(spec_path, service_name, config.version).context(
                            format!("Failed to load Smithy spec: {}", spec_path.display()),
                        )?;
                    parser.parse().context("Failed to parse Smithy spec")?
                }
                SpecFormat::Openapi => {
                    let parser =
                        OpenApiParser::from_file(spec_path, service_name, config.version).context(
                            format!("Failed to load OpenAPI spec: {}", spec_path.display()),
                        )?;
                    parser.parse().context("Failed to parse OpenAPI spec")?
                }
                SpecFormat::Discovery => {
                    let parser =
                        DiscoveryParser::from_file(spec_path, service_name, config.version)
                            .context(format!(
                                "Failed to load Discovery doc: {}",
                                spec_path.display()
                            ))?;
                    parser.parse().context("Failed to parse Discovery doc")?
                }
                SpecFormat::Protobuf => {
                    let parser = ProtobufParser::from_file(spec_path, service_name, config.version)
                        .context(format!(
                            "Failed to load Protobuf FileDescriptorSet: {}",
                            spec_path.display()
                        ))?;
                    parser
                        .parse()
                        .context("Failed to parse Protobuf FileDescriptorSet")?
                }
            };
            Ok(service_def)
        })();

        match service_def_result {
            Ok(service_def) => {
                println!(
                    "{} Parsed {} resources from {}",
                    "✓".green(),
                    service_def.resources.len(),
                    service_name.yellow()
                );
                services.push(service_def);
            }
            Err(e) => {
                eprintln!("{} Skipping {}: {}", "⚠".yellow(), spec_path.display(), e);
                skipped += 1;
            }
        }
    }

    if skipped > 0 {
        println!(
            "\n{} Skipped {} spec(s) due to parse errors",
            "⚠".yellow(),
            skipped
        );
    }

    // Create unified provider definition
    let provider_def = ProviderDefinition {
        provider,
        provider_name: config.provider_name.to_string(),
        sdk_version: config.version.to_string(),
        services,
    };

    let total_resources: usize = provider_def
        .services
        .iter()
        .map(|s| s.resources.len())
        .sum();

    println!(
        "\n{} Total: {} services, {} resources",
        "✓".green().bold(),
        provider_def.services.len(),
        total_resources
    );

    // Generate unified provider
    println!(
        "\n{} {}",
        "→".cyan(),
        "Generating unified provider files...".bold()
    );

    let generator =
        UnifiedProviderGenerator::new(provider_def).context("Failed to create generator")?;
    generator
        .generate_to_directory(config.output)
        .context("Failed to generate unified provider")?;

    println!("\n{}", "✓ Generation complete!".green().bold());
    println!("\n{}", "Generated files:".bold());
    println!("  📄 {}/provider.k", config.output.display());
    println!("  📄 {}/Cargo.toml", config.output.display());
    println!("  📄 {}/README.md", config.output.display());
    println!("  📄 {}/src/lib.rs", config.output.display());
    println!("\n{}", "Next steps:".bold());
    println!("  1. Review generated files in {}", config.output.display());
    println!(
        "  2. Build provider: cd {} && cargo build",
        config.output.display()
    );
    println!("  3. Install in hemmer provider directory");

    Ok(())
}

/// Detect spec format from file extension and content
fn detect_format(path: &Path) -> SpecFormat {
    // Try extension first
    if let Some(ext) = path.extension() {
        if let Some("pb") = ext.to_str() {
            return SpecFormat::Protobuf;
        }
    }

    // Try filename patterns
    if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
        if filename.contains("smithy") || filename.contains("model") {
            return SpecFormat::Smithy;
        }
        if filename.contains("openapi") || filename.contains("swagger") {
            return SpecFormat::Openapi;
        }
        if filename.contains("discovery") {
            return SpecFormat::Discovery;
        }
    }

    // Try reading file content
    if let Ok(content) = std::fs::read_to_string(path) {
        // Check for format-specific markers
        if content.contains("\"smithy\"") && content.contains("\"shapes\"") {
            return SpecFormat::Smithy;
        }
        if content.contains("\"openapi\"") && content.contains("\"paths\"") {
            return SpecFormat::Openapi;
        }
        if content.contains("\"discoveryVersion\"") && content.contains("\"resources\"") {
            return SpecFormat::Discovery;
        }
    }

    // Default to Smithy (most common for AWS)
    SpecFormat::Smithy
}

/// Infer service name from filename
fn infer_service_name(path: &Path) -> Option<String> {
    path.file_stem().and_then(|s| s.to_str()).map(|s| {
        let mut name = s.to_string();

        // Remove common domain suffixes (e.g., .k8s.io, .apiserver, .googleapis.com)
        name = name
            .replace(".k8s.io", "")
            .replace(".apiserver", "")
            .replace(".googleapis.com", "")
            .replace(".azure.com", "");

        // Remove common spec file suffixes
        name = name
            .replace("_openapi", "")
            .replace("-openapi", "")
            .replace("_discovery", "")
            .replace("-discovery", "");

        // Remove version suffixes like "-v1", "__v1"
        name = name.split("__v").next().unwrap_or(&name).to_string();
        name = name.split("-v").next().unwrap_or(&name).to_string();

        // Split on . and take first part (handles remaining dots)
        name = name.split('.').next().unwrap_or(&name).to_string();

        // Apply snake_case conversion to clean up __ and format properly
        sanitize_name(&name)
    })
}

/// Sanitize a name for use as Rust identifier (module/directory name)
fn sanitize_name(s: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();

    for (i, &ch) in chars.iter().enumerate() {
        if ch.is_uppercase() {
            let should_add_underscore = i > 0
                && (chars[i - 1].is_lowercase()
                    || chars[i - 1].is_ascii_digit()
                    || (i + 1 < chars.len() && chars[i + 1].is_lowercase()));
            if should_add_underscore && !result.ends_with('_') {
                result.push('_');
            }
            result.push(ch.to_ascii_lowercase());
        } else if ch == '-' || ch == ' ' || ch == '.' {
            if !result.is_empty() && !result.ends_with('_') {
                result.push('_');
            }
        } else {
            result.push(ch);
        }
    }

    // Clean up multiple consecutive underscores
    while result.contains("__") {
        result = result.replace("__", "_");
    }

    // Remove leading/trailing underscores
    result.trim_matches('_').to_string()
}

/// Discover spec files in a directory
fn discover_specs(
    dir: &Path,
    format: Option<SpecFormat>,
    filter: Option<&[String]>,
    verbose: bool,
) -> Result<Vec<PathBuf>> {
    use std::fs;

    if !dir.is_dir() {
        anyhow::bail!("Not a directory: {}", dir.display());
    }

    let mut specs = Vec::new();

    // Walk directory recursively
    fn walk_dir(
        dir: &Path,
        specs: &mut Vec<PathBuf>,
        format: Option<SpecFormat>,
        filter: Option<&[String]>,
        verbose: bool,
    ) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Skip __test__ directories
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    if dir_name.contains("__test__") {
                        if verbose {
                            println!("  Skipping test directory: {}", path.display());
                        }
                        continue;
                    }
                }

                // Recurse into subdirectories
                walk_dir(&path, specs, format, filter, verbose)?;
            } else if path.is_file() {
                // Skip files with non-spec extensions
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if !matches!(ext, "json" | "pb") {
                        continue;
                    }
                } else {
                    // Skip files without extensions
                    continue;
                }

                // Check if file matches format
                let detected_format = detect_format(&path);

                // If format specified, skip non-matching files
                if let Some(expected_format) = format {
                    if !matches_format(&detected_format, &expected_format) {
                        continue;
                    }
                }

                // Check if service name matches filter
                if let Some(filter_list) = filter {
                    if let Some(service_name) = infer_service_name(&path) {
                        if !filter_list.iter().any(|f| service_name.contains(f)) {
                            if verbose {
                                println!("  Skipping {} (not in filter)", path.display());
                            }
                            continue;
                        }
                    }
                }

                if verbose {
                    println!("  Found: {}", path.display());
                }
                specs.push(path);
            }
        }
        Ok(())
    }

    walk_dir(dir, &mut specs, format, filter, verbose)?;

    println!("{} Discovered {} spec files", "✓".green(), specs.len());

    Ok(specs)
}

fn matches_format(detected: &SpecFormat, expected: &SpecFormat) -> bool {
    matches!(
        (detected, expected),
        (SpecFormat::Smithy, SpecFormat::Smithy)
            | (SpecFormat::Openapi, SpecFormat::Openapi)
            | (SpecFormat::Discovery, SpecFormat::Discovery)
            | (SpecFormat::Protobuf, SpecFormat::Protobuf)
    )
}
