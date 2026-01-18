//! Detect client type patterns in SDK source code

use crate::{workspace_detector::PackageInfo, AnalyzerError, Result};
use std::fs;
use std::path::Path;
use syn::{Item, ItemMod, ItemStruct};
use walkdir::WalkDir;

/// Result of client type detection
#[derive(Debug, Clone)]
pub struct ClientPattern {
    /// Detected pattern (e.g., "aws_sdk_{service}::Client")
    pub pattern: String,
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
    /// Sample client types found
    #[allow(dead_code)]
    pub samples: Vec<String>,
    /// Whether clients are async
    #[allow(dead_code)]
    pub async_client: bool,
}

/// Detect client type pattern from SDK source code
pub fn detect_pattern(repo_path: &Path, crates: &[&PackageInfo]) -> Result<ClientPattern> {
    let mut found_clients = Vec::new();

    // Analyze first 15 crates for better pattern confidence
    // (increased from 5 to improve detection accuracy)
    for pkg in crates.iter().take(15) {
        if let Some(clients) = find_clients_in_package(repo_path, pkg)? {
            found_clients.extend(clients);
        }
    }

    if found_clients.is_empty() {
        // Fallback: try to infer from crate names
        return Ok(infer_client_pattern_from_names(crates));
    }

    // Generate pattern from found clients
    let pattern = generate_client_pattern(&found_clients, crates);
    let confidence = calculate_client_confidence(&found_clients, crates);

    Ok(ClientPattern {
        pattern,
        confidence,
        samples: found_clients.iter().take(5).cloned().collect(),
        async_client: true, // Default to async (most modern SDKs)
    })
}

/// Find Client struct definitions in a package
fn find_clients_in_package(_repo_path: &Path, pkg: &PackageInfo) -> Result<Option<Vec<String>>> {
    // Get package directory from manifest path
    let manifest_path = Path::new(&pkg.manifest_path);
    let pkg_dir = manifest_path
        .parent()
        .ok_or_else(|| AnalyzerError::Other(anyhow::anyhow!("Invalid manifest path")))?;

    let src_dir = pkg_dir.join("src");
    if !src_dir.exists() {
        return Ok(None);
    }

    let mut clients = Vec::new();

    // Walk through .rs files
    for entry in WalkDir::new(&src_dir)
        .max_depth(3) // Limit depth for performance
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("rs"))
    {
        if let Ok(content) = fs::read_to_string(entry.path()) {
            if let Some(client) = find_client_in_source(&content, &pkg.name) {
                clients.push(client);
            }
        }
    }

    Ok(Some(clients))
}

/// Find Client struct in source code using syn
fn find_client_in_source(source: &str, pkg_name: &str) -> Option<String> {
    // Parse the source file
    let Ok(syntax_tree) = syn::parse_file(source) else {
        return None;
    };

    // Look for pub struct Client
    for item in syntax_tree.items {
        if let Some(client_type) = extract_client_from_item(&item, pkg_name) {
            return Some(client_type);
        }
    }

    None
}

/// Extract client type from AST item
fn extract_client_from_item(item: &Item, pkg_name: &str) -> Option<String> {
    match item {
        Item::Struct(ItemStruct { ident, vis, .. }) => {
            // Check for pub struct Client
            if ident == "Client" && matches!(vis, syn::Visibility::Public(_)) {
                // Convert crate name to module path (aws-sdk-s3 → aws_sdk_s3)
                let module = pkg_name.replace('-', "_");
                return Some(format!("{module}::Client"));
            }
        },
        Item::Mod(ItemMod {
            content: Some((_, items)),
            ..
        }) => {
            // Recursively search in modules
            for inner_item in items {
                if let Some(client) = extract_client_from_item(inner_item, pkg_name) {
                    return Some(client);
                }
            }
        },
        _ => {},
    }

    None
}

/// Infer client pattern from crate names when AST parsing fails
fn infer_client_pattern_from_names(crates: &[&PackageInfo]) -> ClientPattern {
    if crates.is_empty() {
        return ClientPattern {
            pattern: "{service}::Client".to_string(),
            confidence: 0.3,
            samples: vec![],
            async_client: true,
        };
    }

    // Try to infer from first crate name
    let first = &crates[0].name;
    let module = first.replace('-', "_");

    ClientPattern {
        pattern: format!("{module}::Client")
            .replace(&extract_service_from_name(first), "{service}"),
        confidence: 0.5, // Medium-low confidence for inference
        samples: vec![format!("{module}::Client")],
        async_client: true,
    }
}

/// Extract service name from crate name (e.g., "aws-sdk-s3" → "s3")
fn extract_service_from_name(name: &str) -> String {
    name.split('-').next_back().unwrap_or(name).to_string()
}

/// Generate client pattern from found client types
fn generate_client_pattern(clients: &[String], _crates: &[&PackageInfo]) -> String {
    if clients.is_empty() {
        return "{service}::Client".to_string();
    }

    // Get first client as template
    let first = &clients[0];

    // Extract service from the client type itself
    // e.g., "aws_sdk_accessanalyzer::Client" -> find "accessanalyzer"
    // by looking for the last underscore-separated part before "::Client"
    if let Some(module_part) = first.strip_suffix("::Client") {
        // Find the last underscore and extract what comes after
        if let Some(last_underscore_pos) = module_part.rfind('_') {
            let service_part = &module_part[last_underscore_pos + 1..];
            // Replace the service part with {service} placeholder
            return first.replace(service_part, "{service}");
        }
    }

    first.clone()
}

/// Calculate confidence for client pattern
fn calculate_client_confidence(clients: &[String], crates: &[&PackageInfo]) -> f32 {
    if clients.is_empty() {
        return 0.0;
    }

    let analyzed = clients.len();
    let total = crates.len().min(15); // We analyze first 15 crates

    let coverage = analyzed as f32 / total as f32;

    // Check consistency
    let _pattern = generate_client_pattern(clients, crates);
    let consistent = clients
        .iter()
        .filter(|c| {
            // Check if client matches pattern structure
            c.contains("::Client") && c.contains('_')
        })
        .count();

    let consistency = consistent as f32 / analyzed as f32;

    // Combined score
    let score = (coverage * 0.4 + consistency * 0.6).clamp(0.0, 1.0);

    match score {
        s if s >= 0.9 => 0.95,
        s if s >= 0.8 => 0.90,
        s if s >= 0.7 => 0.85,
        s => s,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_service_from_name() {
        assert_eq!(extract_service_from_name("aws-sdk-s3"), "s3");
        assert_eq!(extract_service_from_name("gcp-sdk-storage"), "storage");
        assert_eq!(extract_service_from_name("kubernetes"), "kubernetes");
    }

    #[test]
    fn test_infer_client_pattern() {
        let pkgs = [PackageInfo {
            name: "aws-sdk-s3".to_string(),
            version: "1.0.0".to_string(),
            manifest_path: "".to_string(),
        }];
        let refs: Vec<_> = pkgs.iter().collect();

        let pattern = infer_client_pattern_from_names(&refs);
        assert!(pattern.pattern.contains("{service}"));
        assert!(pattern.pattern.contains("::Client"));
        assert!(pattern.confidence >= 0.4 && pattern.confidence <= 0.6);
    }

    #[test]
    fn test_find_client_in_source() {
        let source = r#"
            pub struct Client {
                inner: Inner,
            }

            impl Client {
                pub fn new(config: &Config) -> Self {
                    Self { inner: Inner::new(config) }
                }
            }
        "#;

        let result = find_client_in_source(source, "aws-sdk-s3");
        assert_eq!(result, Some("aws_sdk_s3::Client".to_string()));
    }

    #[test]
    fn test_generate_client_pattern() {
        let clients = vec![
            "aws_sdk_s3::Client".to_string(),
            "aws_sdk_ec2::Client".to_string(),
        ];

        let pkgs = [
            PackageInfo {
                name: "aws-sdk-s3".to_string(),
                version: "1.0.0".to_string(),
                manifest_path: "".to_string(),
            },
            PackageInfo {
                name: "aws-sdk-ec2".to_string(),
                version: "1.0.0".to_string(),
                manifest_path: "".to_string(),
            },
        ];
        let refs: Vec<_> = pkgs.iter().collect();

        let pattern = generate_client_pattern(&clients, &refs);
        assert_eq!(pattern, "aws_sdk_{service}::Client");
    }
}
