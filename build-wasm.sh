#!/bin/bash
set -e

CRATE_NAME="nucleation"
WEB_OUT_NAME="${CRATE_NAME}-web"

echo "Cleaning previous build artifacts..."
rm -rf pkg-bundler/ pkg-web/ pkg/

echo "Building WASM package for bundlers..."
wasm-pack build --target bundler --out-dir pkg-bundler --out-name "${CRATE_NAME}"

echo "Building WASM package for direct web use..."
wasm-pack build --target web --out-dir pkg-web --out-name "${WEB_OUT_NAME}"

echo "Preparing final 'pkg' directory for publishing..."
mkdir -p pkg

echo "Copying bundler-targeted files to pkg/..."
cp pkg-bundler/"${CRATE_NAME}".js pkg/
cp pkg-bundler/"${CRATE_NAME}"_bg.wasm pkg/
cp pkg-bundler/"${CRATE_NAME}".d.ts pkg/
cp pkg-bundler/package.json pkg/

echo "Copying web-targeted JavaScript files to pkg/..."
cp pkg-web/"${WEB_OUT_NAME}".js pkg/
cp pkg-web/"${WEB_OUT_NAME}_bg.js" pkg/

echo "Modifying pkg/package.json..."
node -e "
const fs = require('fs');
const pkgPath = './pkg/package.json';
if (!fs.existsSync(pkgPath)) {
  console.error('Error: pkg/package.json does not exist.'); process.exit(1);
}
const pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf8'));
pkg.files = [
    '${CRATE_NAME}.js',
    '${CRATE_NAME}_bg.wasm',
    '${CRATE_NAME}.d.ts',
    '${WEB_OUT_NAME}.js',
    '${WEB_OUT_NAME}_bg.js',
    'README.md'
];
pkg.module = '${CRATE_NAME}.js';
pkg.main = '${CRATE_NAME}.js';
pkg.types = '${CRATE_NAME}.d.ts';
pkg.name = '${CRATE_NAME}';
fs.writeFileSync(pkgPath, JSON.stringify(pkg, null, 2));
console.log('Successfully modified pkg/package.json');
"

if [ ! -f "pkg/README.md" ]; then
  if [ -f "README.md" ]; then
    cp README.md pkg/README.md
  else
    echo \"# ${CRATE_NAME}\" > pkg/README.md; echo \"\" >> pkg/README.md; echo \"WebAssembly library built from Rust.\" >> pkg/README.md
  fi
fi

if [ -d "wasm-test" ]; then
  echo "Copying files to wasm-test directory..."
  cp pkg/"${CRATE_NAME}".js pkg/"${CRATE_NAME}"_bg.wasm pkg/"${WEB_OUT_NAME}".js pkg/"${WEB_OUT_NAME}_bg.js" wasm-test/
fi

echo ""
echo "Build complete!"
echo "The 'pkg' directory includes:"
echo "  - ${CRATE_NAME}.js, ${CRATE_NAME}_bg.wasm, ${CRATE_NAME}.d.ts (for bundlers)"
echo "  - ${WEB_OUT_NAME}.js, ${WEB_OUT_NAME}_bg.js (for web/CDN, uses ${CRATE_NAME}_bg.wasm)"