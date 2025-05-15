#!/bin/bash
set -e

CRATE_NAME="nucleation"
OUT_NAME="${CRATE_NAME}"

echo "INFO: Cleaning previous build artifacts (pkg/)..."
rm -rf pkg/

echo "INFO: Building WASM package (target: bundler, feature: wasm)..."
wasm-pack build --target bundler --out-dir pkg --out-name "${OUT_NAME}" --features wasm

echo "INFO: Build complete. Verifying essential files in pkg/..."
ls -l pkg/

if [ ! -f pkg/"${OUT_NAME}_bg.js" ]; then
    echo "ERROR: pkg/${OUT_NAME}_bg.js (JS helper with class wrappers) is MISSING. Build failed."
    exit 1
fi

if ! grep -q "class SchematicWrapper" pkg/"${OUT_NAME}_bg.js"; then
    echo "ERROR: SchematicWrapper class not found in pkg/${OUT_NAME}_bg.js. 'wasm' feature might not have enabled bindings correctly."
    exit 1
fi
echo "INFO: Essential JS helper file and SchematicWrapper class found."

echo "INFO: Modifying pkg/package.json for npm publishing..."
node -e "
const fs = require('fs');
const path = require('path');

const pkgPath = path.join('pkg', 'package.json');
const cargoTomlPath = path.join('..', 'Cargo.toml'); // Assuming Cargo.toml is one level up from where pkg is created by wasm-pack

if (!fs.existsSync(pkgPath)) {
  console.error('ERROR: pkg/package.json does not exist. wasm-pack build may have failed.');
  process.exit(1);
}

const pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf8'));

// Ensure 'files' array includes all necessary outputs for bundler target
pkg.files = [
    '${OUT_NAME}.js',         // e.g., nucleation.js
    '${OUT_NAME}_bg.wasm',    // e.g., nucleation_bg.wasm
    '${OUT_NAME}_bg.js',      // e.g., nucleation_bg.js (contains JS wrappers)
    '${OUT_NAME}.d.ts',       // e.g., nucleation.d.ts
    'README.md'
];
pkg.files = [...new Set(pkg.files)]; // Remove duplicates

// Ensure standard fields are set (wasm-pack usually handles these from Cargo.toml)
pkg.name = pkg.name || '${CRATE_NAME}';
pkg.module = pkg.module || '${OUT_NAME}.js';
pkg.main = pkg.main || '${OUT_NAME}.js';     // For CommonJS compatibility if needed, often same as module
pkg.types = pkg.types || '${OUT_NAME}.d.ts';

// Attempt to synchronize version from root Cargo.toml if not already set by wasm-pack
if (!pkg.version && fs.existsSync(cargoTomlPath)) {
    try {
        const cargoTomlContent = fs.readFileSync(cargoTomlPath, 'utf8');
        const versionMatch = cargoTomlContent.match(/^version\s*=\s*\"([^\"]+)\"/m);
        if (versionMatch && versionMatch[1]) {
            pkg.version = versionMatch[1];
            console.log('INFO: Set version in pkg/package.json from root Cargo.toml:', pkg.version);
        }
    } catch (e) {
        console.warn('WARN: Could not read version from root Cargo.toml:', e.message);
    }
}
if (!pkg.version) {
    console.warn('WARN: Version not set in pkg/package.json and could not be read from Cargo.toml. Please set manually or ensure wasm-pack sets it.');
}

fs.writeFileSync(pkgPath, JSON.stringify(pkg, null, 2));
console.log('INFO: pkg/package.json has been successfully modified.');
"

# Create/Copy README.md into pkg/ if it doesn't exist in pkg/ yet
if [ ! -f "pkg/README.md" ]; then
  if [ -f "README.md" ]; then # Check for README.md in the project root
    echo "INFO: Copying project README.md to pkg/"
    cp README.md pkg/README.md
  else
    echo "INFO: Creating a basic README.md in pkg/"
    echo \"# ${CRATE_NAME}\" > pkg/README.md
    echo \"\" >> pkg/README.md
    echo \"WebAssembly library built from Rust using '--target bundler'.\" >> pkg/README.md
  fi
fi

echo ""
echo "--------------------------------------------------------------------"
echo " BUILD SCRIPT COMPLETE (target: bundler, feature: wasm)"
echo "--------------------------------------------------------------------"
echo "The 'pkg/' directory is prepared for testing or publishing."
echo "Key files for CDN usage (manual setup required in JS):"
echo "  - pkg/${OUT_NAME}_bg.js (Contains JS wrappers like SchematicWrapper)"
echo "  - pkg/${OUT_NAME}_bg.wasm (The WASM binary)"
echo "--------------------------------------------------------------------"