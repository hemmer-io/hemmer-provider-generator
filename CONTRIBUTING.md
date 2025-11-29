# Contributing to Hemmer Provider Generator

Thank you for your interest in contributing to the Hemmer Provider Generator! This document provides guidelines and information about our development workflow.

## Development Setup

### Prerequisites

- **Rust**: 1.82 or newer ([install via rustup](https://rustup.rs/))
- **Git**: For version control

### Initial Setup

1. **Fork and clone the repository**
   ```bash
   git clone https://github.com/YOUR_USERNAME/hemmer-provider-generator.git
   cd hemmer-provider-generator
   ```

2. **Build the project**
   ```bash
   cargo build
   ```

3. **Run tests to verify setup**
   ```bash
   cargo test
   ```

4. **Install pre-commit hooks** (recommended)
   ```bash
   ./scripts/install-hooks.sh
   ```

## Development Workflow

### Making Changes

1. **Create a new branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes**
   - Write code following Rust best practices
   - Add tests for new functionality
   - Update documentation as needed

3. **Format and lint your code**
   ```bash
   cargo fmt --all
   cargo clippy --workspace --all-features --all-targets -- -D warnings
   ```

4. **Run tests**
   ```bash
   cargo test --workspace --all-features
   ```

5. **Commit your changes**
   We use [Conventional Commits](https://www.conventionalcommits.org/) format:
   ```
   feat: add AWS SDK Smithy parser
   fix: correct type mapping for nested structures
   docs: update README with usage examples
   test: add integration tests for S3 resource generation
   ci: update GitHub Actions workflow
   ```

   Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`, `ci`

6. **Push and create a Pull Request**
   ```bash
   git push origin feature/your-feature-name
   ```

## CI/CD Pipeline

All pull requests automatically run through our CI pipeline:

### Automated Checks

1. **Format Check** (`cargo fmt`)
   - Ensures code follows Rust formatting standards
   - Must pass for PR to be merged

2. **Clippy Linting** (`cargo clippy`)
   - Catches common mistakes and non-idiomatic code
   - Runs with `-D warnings` (warnings are treated as errors)

3. **Tests** (Multi-platform)
   - Runs on: Ubuntu, macOS, Windows
   - Rust versions: stable, beta (nightly allowed to fail)
   - Includes both unit and integration tests
   - Also runs documentation tests

4. **Security Audit** (`cargo audit`)
   - Checks for known security vulnerabilities in dependencies
   - Runs as informational (won't block PR)

5. **Documentation** (`cargo doc`)
   - Ensures documentation builds without warnings
   - Validates all doc comments

6. **Code Coverage**
   - Generates coverage reports using `cargo-llvm-cov`
   - HTML reports available as PR artifacts
   - Coverage summary printed in workflow logs

### Pre-commit Hooks

Pre-commit hooks run locally before each commit:

- **cargo fmt**: Auto-format code (auto-fixes if needed)
- **cargo clippy**: Lint code
- **cargo test**: Run all tests

Install with: `./scripts/install-hooks.sh`

Bypass temporarily with: `git commit --no-verify`

## Pull Request Process

1. **Title Format**
   Use Conventional Commits format for PR titles:
   ```
   feat: implement Smithy parser for AWS SDK
   fix: correct Rust type generation for HashMaps
   docs: add examples for GCP provider generation
   test: add unit tests for template engine
   refactor: simplify parser architecture
   chore: update dependencies to latest versions
   ci: update GitHub Actions workflow
   ```

   **Types:**
   - `feat`: New feature or capability
   - `fix`: Bug fix
   - `docs`: Documentation changes only
   - `test`: Adding or updating tests
   - `refactor`: Code changes that neither fix bugs nor add features
   - `chore`: Maintenance tasks, dependency updates
   - `ci`: Changes to CI/CD configuration
   - `perf`: Performance improvements
   - `style`: Code style changes (formatting, etc.)

   **Include issue reference:**
   ```
   feat: implement AWS SDK parser (Issue #2)
   fix: resolve template rendering edge case (Issue #15)
   ```

2. **Ensure all CI checks pass**
   - All tests must pass on all platforms
   - Code must be formatted and linted
   - Documentation must build

3. **Keep PRs focused**
   - One feature/fix per PR
   - Include relevant tests
   - Update documentation

4. **Write clear descriptions**
   - Explain what changed and why
   - Link related issues with `Closes #123`
   - Add labels as appropriate
   - Follow the PR template if provided

5. **Respond to reviews**
   - Address feedback promptly
   - Push fixup commits or squash as preferred
   - Re-request review when ready

## Code Style

### Rust Style

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for formatting (enforced by CI)
- Address all `cargo clippy` warnings
- Prefer explicit error types over generic errors
- Use meaningful variable and function names
- Add doc comments for public APIs

### Example

```rust
/// Parse AWS SDK Smithy model and extract service definitions
///
/// # Arguments
///
/// * `model_path` - Path to the Smithy model file
///
/// # Errors
///
/// Returns `GeneratorError::Parser` if:
/// - The file cannot be read
/// - The Smithy model syntax is invalid
/// - Required service definitions are missing
///
/// # Example
///
/// ```no_run
/// use hemmer_provider_generator::parse_smithy_model;
///
/// let service = parse_smithy_model("aws-sdk-s3.smithy")?;
/// println!("Found {} operations", service.operations.len());
/// # Ok::<(), hemmer_provider_generator::GeneratorError>(())
/// ```
pub fn parse_smithy_model(model_path: &str) -> Result<ServiceDefinition> {
    let content = std::fs::read_to_string(model_path)?;
    let model = smithy::parse(&content)?;
    extract_service_definition(&model)
}
```

## Testing

### Test Organization

- **Unit tests**: In the same file as the code (using `#[cfg(test)] mod tests`)
- **Integration tests**: In `tests/` directory
- **Doc tests**: In documentation comments

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_mapping() {
        let smithy_type = SmithyType::String;
        let rust_type = map_to_rust_type(&smithy_type);
        assert_eq!(rust_type, "String");
    }

    #[test]
    fn test_operation_classification() {
        let op = Operation::new("CreateBucket");
        assert!(op.is_create_operation());
    }
}
```

### Running Tests

```bash
# All tests
cargo test --workspace

# Specific package
cargo test --package hemmer-provider-generator

# Specific test
cargo test test_type_mapping

# With output
cargo test -- --nocapture
```

## Project Structure

Understanding the codebase organization:

```
hemmer-provider-generator/
├── crates/
│   ├── parser/          # SDK parsing (Smithy, OpenAPI)
│   ├── generator/       # Code and manifest generation
│   ├── templates/       # Template engine and templates
│   └── cli/            # Command-line interface
├── tests/              # Integration tests
├── examples/           # Example usage and generated outputs
└── docs/              # Additional documentation
```

## Development Phases

We're following a phased approach. Check the [project board](https://github.com/orgs/hemmer-io/projects/3) for current status:

1. **Phase 1**: Project setup and foundation
2. **Phase 2**: AWS SDK parser implementation
3. **Phase 3**: Generator core implementation
4. **Phase 4**: AWS provider generation (MVP)
5. **Phase 5**: CLI interface and production readiness

See the [README](README.md) for detailed phase descriptions and milestones.

## Release Process

Releases are automated via GitHub Actions:

1. **Create a version tag**
   ```bash
   git tag -a v0.1.0 -m "Release v0.1.0"
   git push origin v0.1.0
   ```

2. **GitHub Actions will**:
   - Build binaries for all platforms (Linux, macOS, Windows)
   - Create a GitHub release
   - Attach compiled binaries
   - (Future) Publish to crates.io

## Getting Help

- **Issues**: [GitHub Issues](https://github.com/hemmer-io/hemmer-provider-generator/issues)
- **Discussions**: [GitHub Discussions](https://github.com/hemmer-io/hemmer-provider-generator/discussions)
- **Main Hemmer Project**: [hemmer-io/hemmer](https://github.com/hemmer-io/hemmer)
- **Project Board**: [Provider Generator](https://github.com/orgs/hemmer-io/projects/3)

## Code of Conduct

Be respectful and constructive. We're all here to build great software together.

## License

By contributing to Hemmer Provider Generator, you agree that your contributions will be licensed under the Apache License 2.0.

Hemmer Provider Generator is licensed under Apache-2.0, which:
- Allows commercial use and integration
- Includes explicit patent grants
- Protects contributors with clear terms
- Is widely accepted in enterprise environments

See [LICENSE](LICENSE) for the full license text.
