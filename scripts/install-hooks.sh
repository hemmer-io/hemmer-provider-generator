#!/bin/bash
#
# Install Git Hooks for Hemmer Provider Generator
#
# This script installs the pre-commit hook that automatically runs
# code quality checks before each commit.
#

set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

HOOK_DIR=".git/hooks"
HOOK_SCRIPT="scripts/pre-commit"
HOOK_DEST="$HOOK_DIR/pre-commit"

echo ""
echo "Installing Hemmer Provider Generator Git Hooks..."
echo ""

# Check if we're in a git repository
if [ ! -d ".git" ]; then
    echo -e "${RED}Error:${NC} Not in a git repository root directory"
    echo "Please run this script from the root of the repository"
    exit 1
fi

# Check if hook script exists
if [ ! -f "$HOOK_SCRIPT" ]; then
    echo -e "${RED}Error:${NC} Hook script not found at $HOOK_SCRIPT"
    exit 1
fi

# Check if hook directory exists
if [ ! -d "$HOOK_DIR" ]; then
    echo -e "${RED}Error:${NC} Git hooks directory not found at $HOOK_DIR"
    exit 1
fi

# Backup existing pre-commit hook if it exists
if [ -f "$HOOK_DEST" ]; then
    BACKUP="$HOOK_DEST.backup.$(date +%s)"
    echo -e "${YELLOW}Warning:${NC} Existing pre-commit hook found"
    echo "   Backing up to: $BACKUP"
    mv "$HOOK_DEST" "$BACKUP"
fi

# Install the hook
cp "$HOOK_SCRIPT" "$HOOK_DEST"
chmod +x "$HOOK_DEST"

echo -e "${GREEN}Done${NC} Pre-commit hook installed successfully!"
echo ""
echo "The hook will now run automatically before each commit and check:"
echo "  - Code formatting (cargo fmt)"
echo "  - Linting (cargo clippy)"
echo "  - Tests (cargo test)"
echo ""
echo "Tips:"
echo "  - To bypass the hook temporarily: git commit --no-verify"
echo "  - To uninstall: rm $HOOK_DEST"
echo "  - To reinstall: run this script again"
echo ""

exit 0
