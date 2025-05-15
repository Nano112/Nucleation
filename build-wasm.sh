#!/bin/bash
set -e

CRATE_NAME="nucleation"
BUNDLER_OUT_NAME="${CRATE_NAME}"
WEB_OUT_NAME="${CRATE_NAME}-web"

echo "Cleaning previous build artifacts..."
rm -rf pkg-bundler/ pkg-web/ pkg/

echo "Building WASM package for bundlers..."
wasm-pack build --target bundler --out-dir pkg-bundler --out-name "${BUNDLER_OUT_NAME}"
# Produces: pkg-bundler/nucleation.js, pkg-bundler/nucleation_bg.wasm, pkg-bundler/nucleation_bg.js (if needed by nucleation_bg.wasm), nucleation.d.ts

echo "Building WASM package for direct web use..."
wasm-pack build --target web --out-dir pkg-web --out-name "${WEB_OUT_NAME}"
# Produces: pkg-web/nucleation-web.js, pkg-web/nucleation-web_bg.wasm, pkg-web/nucleation-web_bg.js (if needed by nucleation-web_bg.wasm), nucleation-web.d.ts

echo "Preparing final 'pkg' directory for publishing..."
mkdir -p pkg

# 1. Copy bundler output (these are the primary npm entries)
echo "Copying bundler-targeted files to pkg/..."
cp pkg-bundler/"${BUNDLER_OUT_NAME}".js pkg/
cp pkg-bundler/"${BUNDLER_OUT_NAME}".d.ts pkg/
cp pkg-bundler/"${BUNDLER_OUT_NAME}"_bg.wasm pkg/
if [ -f pkg-bundler/"${BUNDLER_OUT_NAME}"_bg.js ]; then # Copy bundler's JS helper if it exists
    cp pkg-bundler/"${BUNDLER_OUT_NAME}"_bg.js pkg/
fi
cp pkg-bundler/package.json pkg/ # Start with bundler's package.json

# 2. Copy web output (JS, its helper JS, and its specific WASM)
echo "Copying web-targeted files to pkg/..."
cp pkg-web/"${WEB_OUT_NAME}".js pkg/
if [ -f pkg-web/"${WEB_OUT_NAME}"_bg.js ]; then # Copy web's JS helper if it exists
    cp pkg-web/"${WEB_OUT_NAME}"_bg.js pkg/
fi
cp pkg-web/"${WEB_OUT_NAME}"_bg.wasm pkg/ # Copy web's specific WASM file

echo "Modifying pkg/package.json..."
node -e "
const fs = require('fs');
const pkgPath = './pkg/package.json';
if (!fs.existsSync(pkgPath)) { console.error('Error: pkg/package.json does not exist.'); process.exit(1); }
const pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf8'));

let filesToInclude = [
    '${BUNDLER_OUT_NAME}.js',
    '${BUNDLER_OUT_NAME}.d.ts',
    '${BUNDLER_OUT_NAME}_bg.wasm',
    '${WEB_OUT_NAME}.js',
    '${WEB_OUT_NAME}_bg.wasm',
    'README.md'
];
if (fs.existsSync('./pkg/${BUNDLER_OUT_NAME}_bg.js')) { filesToInclude.push('${BUNDLER_OUT_NAME}_bg.js'); }
if (fs.existsSync('./pkg/${WEB_OUT_NAME}_bg.js')) { filesToInclude.push('${WEB_OUT_NAME}_bg.js'); }
pkg.files = [...new Set(filesToInclude)]; // Ensure unique

pkg.module = '${BUNDLER_OUT_NAME}.js';
pkg.main = '${BUNDLER_OUT_NAME}.js';
pkg.types = '${BUNDLER_OUT_NAME}.d.ts';
pkg.name = '${CRATE_NAME}';
fs.writeFileSync(pkgPath, JSON.stringify(pkg, null, 2));
console.log('Successfully modified pkg/package.json');
"

if [ ! -f "pkg/README.md" ]; then
  if [ -f "README.md" ]; then cp README.md pkg/README.md; else echo \"# ${CRATE_NAME}\" > pkg/README.md; fi
fi

echo ""
echo "Build complete!"
echo "The 'pkg' directory includes:"
echo "  - ${CRATE_NAME}.js, ${CRATE_NAME}_bg.wasm, ${CRATE_NAME}.d.ts (for bundlers)"
echo "  - ${WEB_OUT_NAME}.js, ${WEB_OUT_NAME}_bg.js (for web/CDN, uses ${CRATE_NAME}_bg.wasm)"