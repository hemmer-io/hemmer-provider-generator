# hemmer-provider-generator-analyzer

Automated SDK analyzer that generates Phase 2 metadata YAML files by analyzing provider SDK repositories.

## Overview

The SDK analyzer automatically detects patterns in cloud provider SDK repositories and generates provider metadata YAML files with confidence scores. This reduces the time to add new cloud providers from ~4 hours to ~30 minutes.

## Features

- **Workspace Detection**: Analyzes Cargo workspace structure using `cargo_metadata`
- **Crate Pattern Detection**: Identifies SDK crate naming patterns (e.g., `aws-sdk-{service}`)
- **Client Type Detection**: Parses Rust source with `syn` to find Client types
- **Config Detection**: Discovers configuration crates and attributes
- **Error Categorization**: Maps error variants to standard categories using heuristics
- **Confidence Scoring**: Provides 0.0-1.0 confidence scores for each detected field
- **Annotated Output**: Generates YAML with TODO markers for low-confidence fields

## Usage

### As a Library

```rust
use hemmer_provider_generator_analyzer::SdkAnalyzer;
use std::path::PathBuf;

let analyzer = SdkAnalyzer::new(
    PathBuf::from("./aws-sdk-rust"),
    "aws".to_string()
);

let result = analyzer.analyze().expect("Analysis failed");

// Check confidence
println!("Overall confidence: {:.2} ({})",
    result.confidence.overall,
    result.confidence.level()
);

// Write YAML
result.write_yaml("providers/aws.sdk-metadata.yaml")
    .expect("Failed to write YAML");
```

### Via CLI

```bash
# Analyze local SDK checkout
hemmer-provider-generator analyze-sdk \
  --sdk-path ~/code/aws-sdk-rust \
  --name aws

# Custom output location
hemmer-provider-generator analyze-sdk \
  --sdk-path ./custom-sdk \
  --name custom-cloud \
  --output ./my-providers/custom.yaml

# Set minimum confidence threshold
hemmer-provider-generator analyze-sdk \
  --sdk-path ./sdk \
  --name provider \
  --min-confidence 0.6 \
  --verbose
```

## Confidence Levels

- **HIGH (0.8-1.0)**: Can use as-is, minimal review needed
- **MEDIUM (0.6-0.8)**: Likely correct, verify before use
- **LOW (<0.6)**: Needs manual review and editing

### Scoring Weights

Overall confidence is calculated as a weighted average:
- Crate pattern: 30%
- Client type: 30%
- Config crate: 15%
- Config attributes: 5%
- Error categorization: 20%

## Architecture

```
crates/analyzer/
├── src/
│   ├── lib.rs                    # Public API
│   ├── analyzer.rs               # Core orchestration
│   ├── workspace_detector.rs     # Cargo workspace analysis
│   ├── crate_pattern_detector.rs # Crate naming patterns
│   ├── client_detector.rs        # Client type detection (syn)
│   ├── config_detector.rs        # Config pattern detection
│   ├── error_detector.rs         # Error categorization
│   ├── confidence.rs             # Confidence scoring
│   └── output.rs                 # YAML generation
```

## Automation Target

**Goal**: 75% automation for adding new cloud providers

The analyzer automates detection of:
- SDK crate patterns
- Client type patterns
- Configuration setup
- Basic error categorization

Fields that always need manual review:
- Display name and descriptions
- Config attribute setters and extractors
- Error metadata imports
- Comprehensive error categorization

## Testing

```bash
# Run unit tests
cargo test --package hemmer-provider-generator-analyzer

# Run with verbose output
cargo test --package hemmer-provider-generator-analyzer -- --nocapture
```

Test coverage: 25 unit tests covering all detection modules.

## Future Enhancements

- [ ] Git repository cloning support (issue #116)
- [ ] Interactive mode for ambiguous fields
- [ ] Incremental updates to existing metadata
- [ ] ML-based pattern detection

## License

Apache 2.0 - See [../../LICENSE](../../LICENSE)