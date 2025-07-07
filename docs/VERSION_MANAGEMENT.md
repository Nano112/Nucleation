# Version Management Guide

This document explains how to manage versions in the Nucleation project using the centralized version management system.

## Overview

Version information is centrally managed in `version.toml` and automatically synchronized across all configuration files:

- `Cargo.toml` (Rust crate)
- `pyproject.toml` (Python package)
- `src/api_definition.rs` (API version)
- `Cargo.lock` (automatically updated)

## Quick Start

### 1. Bump Version (Recommended)

Use the Makefile commands to automatically bump versions:

```bash
# Bump patch version (0.1.48 → 0.1.49)
make version-bump-patch

# Bump minor version (0.1.48 → 0.2.0)
make version-bump-minor

# Bump major version (0.1.48 → 1.0.0)
make version-bump-major
```

These commands will:
1. Update `version.toml` with the new version
2. Run the version sync script to update all files
3. Display confirmation of the changes

### 2. Manual Version Update

If you prefer to manually edit the version:

1. Edit `version.toml`:
   ```toml
   [package]
   version = "0.2.0"  # Change this
   description = "A high-performance Minecraft schematic parser and utility library"
   ```

2. Sync all files:
   ```bash
   make version-update
   ```

### 3. Check Version Consistency

Verify all files have the same version:

```bash
make version-check
```

This will display the version from each file, making it easy to spot inconsistencies.

## Available Scripts

Three version update scripts are provided for different environments:

### Shell Script (Recommended)
```bash
./scripts/update-version.sh
```
- ✅ No external dependencies
- ✅ Works on macOS and Linux
- ✅ Colored output
- ✅ Error handling

### Python Script
```bash
./scripts/update-version.py
```
- ⚠️ Requires Python and `toml` package
- ✅ Cross-platform
- ✅ More robust TOML parsing

### Rust Script
```bash
rust-script scripts/update-version.rs
```
- ⚠️ Requires `rust-script` and dependencies
- ✅ Type-safe
- ✅ Native Rust solution

## Makefile Commands

The project includes a comprehensive Makefile with version management:

```bash
# Version Management
make help                 # Show all available commands
make version-update       # Update all files from version.toml
make version-check        # Check version consistency
make version-bump-patch   # Bump patch version
make version-bump-minor   # Bump minor version  
make version-bump-major   # Bump major version

# Development
make test                 # Run all tests
make build                # Build the project
make clean                # Clean build artifacts
make dev                  # Development mode

# Publishing
make publish              # Publish to all registries
```

## CI/CD Integration

The GitHub Actions workflow automatically:

1. **Detects version changes** by reading from `version.toml`
2. **Builds artifacts** with the correct version
3. **Creates releases** when version bumps are detected
4. **Publishes** to multiple registries (crates.io, npm, PyPI)

The CI/CD pipeline will only trigger releases when:
- The version in `Cargo.toml` changes on the main branch
- All tests pass
- All builds succeed

## Best Practices

### For Development

1. **Always use the centralized system**
   - Don't manually edit version numbers in individual files
   - Use `make version-bump-*` or edit `version.toml` + `make version-update`

2. **Check consistency regularly**
   ```bash
   make version-check
   ```

3. **Test before releasing**
   ```bash
   make test
   make build
   ```

### For Releases

1. **Follow semantic versioning**
   - Patch: Bug fixes and internal improvements
   - Minor: New features, backward compatible
   - Major: Breaking changes

2. **Version bump workflow**
   ```bash
   # Example: adding a new feature
   make version-bump-minor
   git add -A
   git commit -m "Bump version to $(grep 'version = ' version.toml | sed 's/.*= *\"\([^\"]*\)\".*/\1/')"
   git push
   ```

3. **Use the publish command**
   ```bash
   make publish  # Interactive with confirmation
   ```

## Troubleshooting

### Version Mismatch

If `make version-check` shows inconsistencies:

```bash
# Fix by syncing from version.toml
make version-update
```

### CI/CD Not Triggering

Ensure:
1. Version actually changed in `Cargo.toml`
2. Changes are pushed to main/master branch
3. All tests are passing

### Script Permissions

If scripts aren't executable:

```bash
chmod +x scripts/update-version.sh
chmod +x scripts/update-version.py
```

### Missing Dependencies

For Python script:
```bash
pip install toml
```

For Rust script:
```bash
cargo install rust-script
```

## File Structure

```
Nucleation/
├── version.toml              # Central version configuration
├── Makefile                  # Convenient commands
├── scripts/
│   ├── update-version.sh     # Shell script (recommended)
│   ├── update-version.py     # Python script
│   └── update-version.rs     # Rust script
├── Cargo.toml               # Auto-updated from version.toml
├── pyproject.toml           # Auto-updated from version.toml
├── src/api_definition.rs    # Auto-updated from version.toml
└── .github/workflows/ci.yml # Uses version.toml for releases
```

## Benefits

✅ **Single source of truth** - Edit version in one place  
✅ **Consistency guaranteed** - All files stay in sync  
✅ **Automation friendly** - Scripts handle the tedious work  
✅ **CI/CD integration** - Automatic releases on version bumps  
✅ **Developer friendly** - Simple commands for common tasks  
✅ **Error prevention** - No more forgotten version updates  

This system eliminates the common problem of forgetting to update versions in multiple files and ensures consistency across the entire project.
