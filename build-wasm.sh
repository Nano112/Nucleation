#!/bin/bash
# Production-ready build script for the 'nucleation' WASM package.
# This script supports both modern bundlers (like Vite) and direct CDN usage.
set -e

# --- Configuration ---
CRATE_NAME="nucleation"
OUT_NAME="${CRATE_NAME}"
CDN_LOADER_FILENAME="${CRATE_NAME}-cdn-loader.js"

# --- Build Steps ---

echo "INFO: Cleaning previous build artifacts..."
rm -rf pkg/

echo "INFO: Building WASM package with --target web for explicit initialization..."
wasm-pack build --target web --out-dir pkg --out-name "${OUT_NAME}" --features wasm
echo "INFO: Base package created successfully."

# --- Create a dedicated loader for CDN usage ---
echo "INFO: Creating custom loader for CDN usage (pkg/${CDN_LOADER_FILENAME})..."
cat << EOF > pkg/"${CDN_LOADER_FILENAME}"
// This loader is for use in a browser via <script type="module"> from a CDN.
// It ensures that the .wasm file is loaded from the correct relative path.

// Import the real init function and all the classes from the main module.
import init, * as wasm from './${OUT_NAME}.js';

// The default export is a new initializer function for CDN use.
// It calls the real 'init' but provides the URL to the .wasm file.
export default async function() {
  const wasmUrl = new URL('./${OUT_NAME}_bg.wasm', import.meta.url);
  await init(wasmUrl);
}

// Re-export all the named classes (SchematicWrapper, etc.).
export * from './${OUT_NAME}.js';
EOF
echo "INFO: CDN loader created."

# --- Configure package.json for publishing ---
echo "INFO: Configuring pkg/package.json for bundler and CDN exports..."
node -e "
const fs = require('fs');
const path = require('path');
const pkgPath = './pkg/package.json';
const pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf8'));

// Define all files to be included in the npm package.
pkg.files = [
    '${OUT_NAME}.js',
    '${OUT_NAME}.d.ts',
    '${OUT_NAME}_bg.wasm',
    '${OUT_A-NAME}_bg.d.ts',
    '${CDN_LOADER_FILENAME}',
    'README.md'
];
pkg.files = [...new Set(pkg.files)]; // Ensure no duplicates.

// Define the main entry points for bundlers.
pkg.module = './${OUT_NAME}.js';
pkg.main = './${OUT_NAME}.js';
pkg.types = './${OUT_NAME}.d.ts';
pkg.name = '${CRATE_NAME}';

// Use the 'exports' field for modern, explicit module resolution.
pkg.exports = {
    // The main entry for bundlers: 'import init from \"nucleation\"'
    '.': {
        'import': './${OUT_NAME}.js',
        'types': './${OUT_NAME}.d.ts'
    },
    // The entry for CDN users: 'import init from \"nucleation/cdn-loader\"'
    './cdn-loader': {
        'import': './${CDN_LOADER_FILENAME}'
    },
    './package.json': './package.json'
};

// Synchronize the package version from the root Cargo.toml file.
const cargoTomlPath = path.join('..', 'Cargo.toml');
if (!pkg.version && fs.existsSync(cargoTomlPath)) {
    try {
        const cargoTomlContent = fs.readFileSync(cargoTomlPath, 'utf8');
        const versionMatch = cargoTomlContent.match(/^version\s*=\s*\"([^\"]+)\"/m);
        if (versionMatch && versionMatch[1]) {
            pkg.version = versionMatch[1];
        }
    } catch (e) {
        console.warn('WARN: Could not read version from root Cargo.toml.', e);
    }
}

fs.writeFileSync(pkgPath, JSON.stringify(pkg, null, 2));
console.log('INFO: pkg/package.json configured successfully.');
"

# --- Final Touches ---
if [ ! -f "pkg/README.md" ]; then
  if [ -f "../README.md" ]; then
    cp ../README.md pkg/README.md;
  fi
fi

echo ""
echo "===================================================================="
echo " ✅ BUILD COMPLETE"
echo "===================================================================="
echo
echo " This package now supports two primary use cases:"
echo
echo " 1) For BUNDLERS (Vite, Webpack, etc.):"
echo "    -------------------------------------"
echo "    import init, { SchematicWrapper } from '${CRATE_NAME}';"
echo "    await init();"
echo "    const schematic = new SchematicWrapper();"
echo
echo " 2) For CDN (in a browser <script type=\"module\">):"
echo "    ------------------------------------------------"
echo "    import init, { SchematicWrapper } from 'https://cdn.jsdelivr.net/npm/${CRATE_NAME}@latest/${CDN_LOADER_FILENAME}';"
echo "    await init();"
echo "    const schematic = new SchematicWrapper();"
echo
echo "===================================================================="