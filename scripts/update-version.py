#!/usr/bin/env python3
"""
Version update script for Nucleation
Updates version across all configuration files from a central source
"""

import os
import re
import sys
import toml
from pathlib import Path

def load_version_config():
    """Load version from version.toml"""
    config_path = Path(__file__).parent.parent / "version.toml"
    if not config_path.exists():
        print(f"Error: {config_path} not found")
        sys.exit(1)
    
    with open(config_path, 'r') as f:
        config = toml.load(f)
    
    return config['package']['version'], config['package']['description']

def update_cargo_toml(version, description):
    """Update Cargo.toml version"""
    cargo_path = Path(__file__).parent.parent / "Cargo.toml"
    
    with open(cargo_path, 'r') as f:
        content = f.read()
    
    # Update version
    content = re.sub(r'^version = ".*"', f'version = "{version}"', content, flags=re.MULTILINE)
    # Update description
    content = re.sub(r'^description = ".*"', f'description = "{description}"', content, flags=re.MULTILINE)
    
    with open(cargo_path, 'w') as f:
        f.write(content)
    
    print(f"Updated Cargo.toml to version {version}")

def update_pyproject_toml(version, description):
    """Update pyproject.toml version"""
    pyproject_path = Path(__file__).parent.parent / "pyproject.toml"
    
    with open(pyproject_path, 'r') as f:
        content = f.read()
    
    # Update version
    content = re.sub(r'^version = ".*"', f'version = "{version}"', content, flags=re.MULTILINE)
    # Update description
    content = re.sub(r'^description = ".*"', f'description = "{description}"', content, flags=re.MULTILINE)
    
    with open(pyproject_path, 'w') as f:
        f.write(content)
    
    print(f"Updated pyproject.toml to version {version}")

def update_api_definition(version):
    """Update src/api_definition.rs version"""
    api_def_path = Path(__file__).parent.parent / "src" / "api_definition.rs"
    
    with open(api_def_path, 'r') as f:
        content = f.read()
    
    # Update the version in the nucleation_api function
    content = re.sub(
        r'version: ".*?".to_string\(\),',
        f'version: "{version}".to_string(),',
        content
    )
    
    with open(api_def_path, 'w') as f:
        f.write(content)
    
    print(f"Updated api_definition.rs to version {version}")

def update_cargo_lock():
    """Update Cargo.lock by running cargo check"""
    os.system("cargo check > /dev/null 2>&1")
    print("Updated Cargo.lock")

def main():
    version, description = load_version_config()
    
    print(f"Updating to version {version}")
    print(f"Description: {description}")
    print()
    
    update_cargo_toml(version, description)
    update_pyproject_toml(version, description)
    update_api_definition(version)
    update_cargo_lock()
    
    print()
    print("✅ All files updated successfully!")
    print(f"New version: {version}")

if __name__ == "__main__":
    main()
