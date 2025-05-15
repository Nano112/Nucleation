#!/bin/bash
set -e  # Exit on any error

# Build for both bundler and web targets
echo "Building WASM package for bundlers (webpack, rollup, etc.)"
wasm-pack build --target bundler --out-dir pkg-bundler

echo "Building WASM package for direct web use"
wasm-pack build --target web --out-dir pkg-web

# Merge the outputs into a single package
echo "Merging packages..."
mkdir -p pkg
cp -r pkg-bundler/* pkg/

# Edit package.json to include both web and bundler files
node -e "
const pkg = require('./pkg/package.json');
pkg.files = [...new Set([...pkg.files, 'nucleation_bg.wasm', 'nucleation.js', 'nucleation_bg.js'])];
pkg.module = 'nucleation.js';
pkg.types = 'nucleation.d.ts';
pkg.main = 'nucleation.js';
require('fs').writeFileSync('./pkg/package.json', JSON.stringify(pkg, null, 2));
"

# For local testing
if [ -d "wasm-test" ]; then
  echo "Copying files to wasm-test directory"
  cp -r pkg/*.js pkg/*.wasm wasm-test/
fi

echo "WASM build completed successfully"