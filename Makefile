# Nucleation Makefile
# Provides convenient commands for development and version management

.PHONY: help version-update version-check test build clean publish dev

# Default target
help:
	@echo "📦 Nucleation Development Commands"
	@echo "=================================="
	@echo ""
	@echo "Version Management:"
	@echo "  version-update    Update all files with version from version.toml"
	@echo "  version-check     Check current version across all files"
	@echo "  version-bump-patch Bump patch version (0.1.0 -> 0.1.1)"
	@echo "  version-bump-minor Bump minor version (0.1.0 -> 0.2.0)"
	@echo "  version-bump-major Bump major version (0.1.0 -> 1.0.0)"
	@echo ""
	@echo "Development:"
	@echo "  test             Run all tests"
	@echo "  build            Build the project"
	@echo "  clean            Clean build artifacts"
	@echo "  dev              Run in development mode"
	@echo ""
	@echo "Publishing:"
	@echo "  publish          Publish to all registries (requires version bump)"
	@echo ""
	@echo "💡 To bump version: edit version.toml, then run 'make version-update'"

# Version management targets
version-update:
	@echo "🔄 Updating version across all files..."
	@./scripts/update-version.sh

version-check:
	@echo "📋 Current version information:"
	@echo "================================"
	@printf "%-18s %s\n" "version.toml:" "$$(grep 'version = ' version.toml | sed 's/.*= *"\([^"]*\)".*/\1/')"
	@printf "%-18s %s\n" "Cargo.toml:" "$$(grep '^version = ' Cargo.toml | sed 's/.*= *"\([^"]*\)".*/\1/')"
	@printf "%-18s %s\n" "pyproject.toml:" "$$(grep '^version = ' pyproject.toml | sed 's/.*= *"\([^"]*\)".*/\1/')"
	@printf "%-18s %s\n" "api_definition.rs:" "$$(grep 'version: "' src/api_definition.rs | sed 's/.*version: *"\([^"]*\)".*/\1/')"

version-bump-patch:
	@echo "🔼 Bumping patch version..."
	@current=$$(grep "version = " version.toml | sed 's/.*= *"\([^"]*\)".*/\1/'); \
	major=$$(echo $$current | cut -d. -f1); \
	minor=$$(echo $$current | cut -d. -f2); \
	patch=$$(echo $$current | cut -d. -f3); \
	new_patch=$$((patch + 1)); \
	new_version="$$major.$$minor.$$new_patch"; \
	sed -i '' "s/version = \".*\"/version = \"$$new_version\"/" version.toml; \
	echo "Updated version from $$current to $$new_version"; \
	$(MAKE) version-update

version-bump-minor:
	@echo "🔼 Bumping minor version..."
	@current=$$(grep "version = " version.toml | sed 's/.*= *"\([^"]*\)".*/\1/'); \
	major=$$(echo $$current | cut -d. -f1); \
	minor=$$(echo $$current | cut -d. -f2); \
	new_minor=$$((minor + 1)); \
	new_version="$$major.$$new_minor.0"; \
	sed -i '' "s/version = \".*\"/version = \"$$new_version\"/" version.toml; \
	echo "Updated version from $$current to $$new_version"; \
	$(MAKE) version-update

version-bump-major:
	@echo "🔼 Bumping major version..."
	@current=$$(grep "version = " version.toml | sed 's/.*= *"\([^"]*\)".*/\1/'); \
	major=$$(echo $$current | cut -d. -f1); \
	new_major=$$((major + 1)); \
	new_version="$$new_major.0.0"; \
	sed -i '' "s/version = \".*\"/version = \"$$new_version\"/" version.toml; \
	echo "Updated version from $$current to $$new_version"; \
	$(MAKE) version-update

# Development targets
test:
	@echo "🧪 Running tests..."
	cargo test

build:
	@echo "🔨 Building project..."
	cargo build --release

clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean
	rm -rf pkg/
	rm -rf target/
	rm -rf release-artifacts/

dev:
	@echo "🚀 Starting development server..."
	cargo run

# Publishing target
publish: version-check
	@echo "📤 Publishing to all registries..."
	@echo "⚠️  Make sure you've bumped the version and committed changes!"
	@read -p "Continue with publish? (y/N): " confirm; \
	if [ "$$confirm" = "y" ] || [ "$$confirm" = "Y" ]; then \
		echo "Publishing..."; \
		git add -A && git commit -m "Bump version to $$(grep 'version = ' version.toml | sed 's/.*= *\"\([^\"]*\)\".*/\1/')" || true; \
		git push; \
	else \
		echo "Publish cancelled."; \
	fi

# Install development dependencies
install-deps:
	@echo "📥 Installing development dependencies..."
	cargo install wasm-pack
	cargo install maturin
	pip install toml
