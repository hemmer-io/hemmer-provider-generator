#!/bin/sh
# Hemmer Provider Generator Installation Script
# Usage: curl -fsSL https://raw.githubusercontent.com/hemmer-io/hemmer-provider-generator/main/install.sh | sh

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

echo "${CYAN}Hemmer Provider Generator Installer${NC}"
echo "======================================"
echo ""

# Check if cargo is installed
if ! command -v cargo >/dev/null 2>&1; then
    echo "${RED}Error: Cargo is not installed.${NC}"
    echo ""
    echo "Please install Rust and Cargo first:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo ""
    echo "Then run this script again."
    exit 1
fi

echo "${GREEN}✓${NC} Cargo found: $(cargo --version)"
echo ""

# Determine installation method
if [ -n "${HEMMER_DEV}" ]; then
    echo "${YELLOW}→${NC} Installing from source (development mode)..."
    cargo install --git https://github.com/hemmer-io/hemmer-provider-generator.git
else
    echo "${YELLOW}→${NC} Installing from crates.io..."
    cargo install hemmer-provider-generator
fi

echo ""
echo "${GREEN}✓${NC} Installation complete!"
echo ""

# Check if cargo bin directory is in PATH
CARGO_BIN="${HOME}/.cargo/bin"
if [ -d "${CARGO_BIN}" ]; then
    if ! echo "${PATH}" | grep -q "${CARGO_BIN}"; then
        echo "${YELLOW}Warning:${NC} ${CARGO_BIN} is not in your PATH"
        echo ""
        echo "Add this to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
        echo "  export PATH=\"\${HOME}/.cargo/bin:\${PATH}\""
        echo ""
    fi
fi

# Verify installation
if command -v hemmer-provider-generator >/dev/null 2>&1; then
    echo "${GREEN}✓${NC} hemmer-provider-generator is ready to use!"
    echo ""
    echo "Version: $(hemmer-provider-generator --version)"
    echo ""
    echo "Quick start:"
    echo "  hemmer-provider-generator --help"
    echo ""
    echo "Examples:"
    echo "  # Parse a spec file"
    echo "  hemmer-provider-generator parse --spec storage-v1.json -v"
    echo ""
    echo "  # Generate a provider"
    echo "  hemmer-provider-generator generate --spec s3.json --service s3 --output ./provider-s3"
    echo ""
    echo "  # Generate unified multi-service provider"
    echo "  hemmer-provider-generator generate-unified --provider aws --spec-dir ./models --output ./provider-aws"
    echo ""
else
    echo "${RED}Error:${NC} Installation succeeded but hemmer-provider-generator command not found."
    echo "You may need to add ${CARGO_BIN} to your PATH."
fi
