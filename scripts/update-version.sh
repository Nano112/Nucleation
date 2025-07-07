#!/bin/bash

# Version update script for Nucleation
# Updates version across all configuration files from a central source

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Function to extract value from TOML file
extract_toml_value() {
    local file="$1"
    local key="$2"
    grep "^$key = " "$file" | sed 's/.*= *"\([^"]*\)".*/\1/' | head -1
}

# Function to update version in a file
update_version_in_file() {
    local file="$1"
    local new_version="$2"
    local new_description="$3"
    
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        sed -i '' "s/^version = \".*\"/version = \"$new_version\"/" "$file"
        if [ -n "$new_description" ]; then
            sed -i '' "s/^description = \".*\"/description = \"$new_description\"/" "$file"
        fi
    else
        # Linux
        sed -i "s/^version = \".*\"/version = \"$new_version\"/" "$file"
        if [ -n "$new_description" ]; then
            sed -i "s/^description = \".*\"/description = \"$new_description\"/" "$file"
        fi
    fi
}

# Function to update API definition
update_api_definition() {
    local file="$1"
    local new_version="$2"
    
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        sed -i '' "s/version: \".*\".to_string(),/version: \"$new_version\".to_string(),/" "$file"
    else
        # Linux
        sed -i "s/version: \".*\".to_string(),/version: \"$new_version\".to_string(),/" "$file"
    fi
}

# Main script
main() {
    print_info "Starting version update process..."
    
    # Check if we're in the right directory
    if [ ! -f "version.toml" ]; then
        print_error "version.toml not found. Make sure you're in the project root directory."
        exit 1
    fi
    
    # Load version from version.toml
    VERSION=$(extract_toml_value "version.toml" "version")
    DESCRIPTION=$(extract_toml_value "version.toml" "description")
    
    if [ -z "$VERSION" ]; then
        print_error "Could not extract version from version.toml"
        exit 1
    fi
    
    if [ -z "$DESCRIPTION" ]; then
        print_error "Could not extract description from version.toml"
        exit 1
    fi
    
    print_info "Updating to version: $VERSION"
    print_info "Description: $DESCRIPTION"
    echo
    
    # Update Cargo.toml
    if [ -f "Cargo.toml" ]; then
        update_version_in_file "Cargo.toml" "$VERSION" "$DESCRIPTION"
        print_success "Updated Cargo.toml"
    else
        print_warning "Cargo.toml not found, skipping"
    fi
    
    # Update pyproject.toml
    if [ -f "pyproject.toml" ]; then
        update_version_in_file "pyproject.toml" "$VERSION" "$DESCRIPTION"
        print_success "Updated pyproject.toml"
    else
        print_warning "pyproject.toml not found, skipping"
    fi
    
    # Update src/api_definition.rs
    if [ -f "src/api_definition.rs" ]; then
        update_api_definition "src/api_definition.rs" "$VERSION"
        print_success "Updated src/api_definition.rs"
    else
        print_warning "src/api_definition.rs not found, skipping"
    fi
    
    # Update Cargo.lock
    if command -v cargo &> /dev/null; then
        print_info "Updating Cargo.lock..."
        cargo check > /dev/null 2>&1
        print_success "Updated Cargo.lock"
    else
        print_warning "cargo not found, skipping Cargo.lock update"
    fi
    
    echo
    print_success "All files updated successfully!"
    print_info "New version: $VERSION"
}

# Check if script is being sourced or executed
if [ "${BASH_SOURCE[0]}" == "${0}" ]; then
    main "$@"
fi
