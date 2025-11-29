# Scripts

This directory contains development scripts for the Hemmer Provider Generator.

## Git Hooks

### Installation

To install the Git hooks, run from the repository root:

```bash
./scripts/install-hooks.sh
```

### What the hooks do

The pre-commit hook runs automatically before each commit and performs:

1. **Code formatting** - Runs `cargo fmt` to check formatting (auto-fixes if needed)
2. **Linting** - Runs `cargo clippy` to catch common issues
3. **Tests** - Runs `cargo test` to ensure all tests pass

### Bypassing hooks

To temporarily skip the pre-commit checks:

```bash
git commit --no-verify
```

### Uninstalling

To remove the pre-commit hook:

```bash
rm .git/hooks/pre-commit
```
