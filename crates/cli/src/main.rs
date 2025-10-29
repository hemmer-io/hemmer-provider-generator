//! Hemmer Provider Generator CLI
//!
//! Command-line interface for generating Hemmer providers from cloud SDKs.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use colored::*;
use hemmer_provider_generator_generator::ProviderGenerator;
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
            generate_unified_command(
                &provider,
                specs.as_deref(),
                spec_dir.as_deref(),
                format,
                filter.as_deref(),
                services.as_deref(),
                &version,
                output.as_path(),
                cli.verbose,
            )?;
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

fn generate_unified_command(
    provider_name: &str,
    spec_paths: Option<&[PathBuf]>,
    spec_dir: Option<&Path>,
    format: Option<SpecFormat>,
    filter: Option<&[String]>,
    service_names: Option<&[String]>,
    version: &str,
    output: &Path,
    verbose: bool,
) -> Result<()> {
    use hemmer_provider_generator_common::{Provider, ProviderDefinition};
    use std::fs;

    // Discover spec files
    let discovered_specs: Vec<PathBuf> = if let Some(dir) = spec_dir {
        println!(
            "{} Scanning directory for specs: {}",
            "→".cyan(),
            dir.display()
        );
        discover_specs(dir, format, filter, verbose)?
    } else if let Some(paths) = spec_paths {
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
        provider_name.yellow(),
        discovered_specs.len()
    );

    // Parse provider enum from string
    let provider = match provider_name.to_lowercase().as_str() {
        "aws" => Provider::Aws,
        "gcp" => Provider::Gcp,
        "azure" => Provider::Azure,
        "kubernetes" | "k8s" => Provider::Kubernetes,
        _ => anyhow::bail!("Unknown provider: {}", provider_name),
    };

    // Parse all specs
    let mut services = Vec::new();
    for (i, spec_path) in discovered_specs.iter().enumerate() {
        println!(
            "{} Parsing spec {}/{}: {}",
            "→".cyan(),
            i + 1,
            discovered_specs.len(),
            spec_path.display()
        );

        // Detect format if not specified
        let detected_format = format.unwrap_or_else(|| detect_format(spec_path));

        // Get service name
        let inferred_name = infer_service_name(spec_path);
        let service_name = service_names
            .and_then(|names| names.get(i).map(String::as_str))
            .or_else(|| inferred_name.as_deref())
            .unwrap_or("unknown");

        if verbose {
            println!("  Format: {}", detected_format);
            println!("  Service: {}", service_name);
        }

        // Parse based on format
        let service_def = match detected_format {
            SpecFormat::Smithy => {
                let parser = SmithyParser::from_file(spec_path, service_name, version)
                    .context(format!("Failed to load Smithy spec: {}", spec_path.display()))?;
                parser.parse().context("Failed to parse Smithy spec")?
            }
            SpecFormat::Openapi => {
                let parser = OpenApiParser::from_file(spec_path, service_name, version)
                    .context(format!("Failed to load OpenAPI spec: {}", spec_path.display()))?;
                parser.parse().context("Failed to parse OpenAPI spec")?
            }
            SpecFormat::Discovery => {
                let parser = DiscoveryParser::from_file(spec_path, service_name, version)
                    .context(format!("Failed to load Discovery doc: {}", spec_path.display()))?;
                parser.parse().context("Failed to parse Discovery doc")?
            }
            SpecFormat::Protobuf => {
                let parser = ProtobufParser::from_file(spec_path, service_name, version).context(
                    format!("Failed to load Protobuf FileDescriptorSet: {}", spec_path.display()),
                )?;
                parser
                    .parse()
                    .context("Failed to parse Protobuf FileDescriptorSet")?
            }
        };

        println!(
            "{} Parsed {} resources from {}",
            "✓".green(),
            service_def.resources.len(),
            service_name.yellow()
        );

        services.push(service_def);
    }

    // Create unified provider definition
    let provider_def = ProviderDefinition {
        provider,
        provider_name: provider_name.to_string(),
        sdk_version: version.to_string(),
        services,
    };

    println!(
        "\n{} Total: {} services, {} resources",
        "✓".green().bold(),
        provider_def.services.len(),
        provider_def
            .services
            .iter()
            .map(|s| s.resources.len())
            .sum::<usize>()
    );

    // TODO: Generate unified provider (requires new generator implementation)
    println!(
        "\n{} {}",
        "→".cyan(),
        "Generating unified provider files...".bold()
    );
    println!(
        "{} This feature is under development. Use 'generate' command for single services.",
        "!".yellow()
    );

    // For now, just show what would be generated
    println!("\n{}", "Would generate:".bold());
    println!("  📁 {}/", output.display());
    println!("  📄   ├── provider.k (unified schema)");
    println!("  📄   ├── Cargo.toml");
    println!("  📄   └── src/");
    println!("  📄       ├── lib.rs ({}Provider)", provider_name);
    for service in &provider_def.services {
        println!("  📁       ├── {}/", service.name);
        println!("  📄       │   ├── mod.rs");
        println!("  📁       │   └── resources/ ({} resources)", service.resources.len());
    }

    println!("\n{}", "See issue #16 for implementation progress".yellow());

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
        // Remove version suffixes like "-v1"
        s.split('-')
            .next()
            .unwrap_or(s)
            .split('.')
            .next()
            .unwrap_or(s)
            .to_string()
    })
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
                // Recurse into subdirectories
                walk_dir(&path, specs, format, filter, verbose)?;
            } else if path.is_file() {
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

    println!(
        "{} Discovered {} spec files",
        "✓".green(),
        specs.len()
    );

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
