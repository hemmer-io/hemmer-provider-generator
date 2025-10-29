# Hemmer Provider Generator Installation Script for Windows
# Usage: irm https://raw.githubusercontent.com/hemmer-io/hemmer-provider-generator/main/install.ps1 | iex

$ErrorActionPreference = "Stop"

# Colors for output
function Write-ColorOutput($ForegroundColor) {
    $fc = $host.UI.RawUI.ForegroundColor
    $host.UI.RawUI.ForegroundColor = $ForegroundColor
    if ($args) {
        Write-Output $args
    }
    $host.UI.RawUI.ForegroundColor = $fc
}

Write-ColorOutput Cyan "Hemmer Provider Generator Installer"
Write-Output "======================================"
Write-Output ""

# Check if cargo is installed
$cargoVersion = Get-Command cargo -ErrorAction SilentlyContinue
if (-not $cargoVersion) {
    Write-ColorOutput Red "Error: Cargo is not installed."
    Write-Output ""
    Write-Output "Please install Rust and Cargo first:"
    Write-Output "  https://rustup.rs/"
    Write-Output ""
    Write-Output "Or run this command in PowerShell:"
    Write-Output "  irm https://win.rustup.rs/x86_64 | iex"
    Write-Output ""
    Write-Output "Then run this script again."
    exit 1
}

Write-ColorOutput Green "✓ Cargo found: $(cargo --version)"
Write-Output ""

# Determine installation method
if ($env:HEMMER_DEV) {
    Write-ColorOutput Yellow "→ Installing from source (development mode)..."
    cargo install --git https://github.com/hemmer-io/hemmer-provider-generator.git
} else {
    Write-ColorOutput Yellow "→ Installing from crates.io..."
    cargo install hemmer-provider-generator
}

Write-Output ""
Write-ColorOutput Green "✓ Installation complete!"
Write-Output ""

# Check if cargo bin directory is in PATH
$cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
if (Test-Path $cargoBin) {
    $pathParts = $env:PATH -split ";"
    if (-not ($pathParts -contains $cargoBin)) {
        Write-ColorOutput Yellow "Warning: $cargoBin is not in your PATH"
        Write-Output ""
        Write-Output "To add it permanently, run this in an elevated PowerShell:"
        Write-Output "  [Environment]::SetEnvironmentVariable('Path', `$env:Path + ';$cargoBin', 'User')"
        Write-Output ""
        Write-Output "Or add it to your current session:"
        Write-Output "  `$env:PATH += ';$cargoBin'"
        Write-Output ""
    }
}

# Verify installation
$hemmerGen = Get-Command hemmer-provider-generator -ErrorAction SilentlyContinue
if ($hemmerGen) {
    Write-ColorOutput Green "✓ hemmer-provider-generator is ready to use!"
    Write-Output ""
    Write-Output "Version: $(hemmer-provider-generator --version)"
    Write-Output ""
    Write-Output "Quick start:"
    Write-Output "  hemmer-provider-generator --help"
    Write-Output ""
    Write-Output "Examples:"
    Write-Output "  # Parse a spec file"
    Write-Output "  hemmer-provider-generator parse --spec storage-v1.json -v"
    Write-Output ""
    Write-Output "  # Generate a provider"
    Write-Output "  hemmer-provider-generator generate --spec s3.json --service s3 --output .\provider-s3"
    Write-Output ""
    Write-Output "  # Generate unified multi-service provider"
    Write-Output "  hemmer-provider-generator generate-unified --provider aws --spec-dir .\models --output .\provider-aws"
    Write-Output ""
} else {
    Write-ColorOutput Red "Error: Installation succeeded but hemmer-provider-generator command not found."
    Write-Output "You may need to add $cargoBin to your PATH."
    Write-Output ""
    Write-Output "Restart your terminal or run:"
    Write-Output "  `$env:PATH += ';$cargoBin'"
}
