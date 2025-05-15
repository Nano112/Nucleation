#!/bin/bash
set -e

# --- Configuration ---
CRATE_NAME="nucleation"
BUNDLER_OUT_NAME="${CRATE_NAME}"
WEB_OUT_NAME="${CRATE_NAME}-web"
# --- End Configuration ---

echo "INFO: Starting WASM build process for ${CRATE_NAME}..."
echo "INFO: Cleaning previous build artifacts (pkg-bundler/, pkg-web/, pkg/)..."
rm -rf pkg-bundler/ pkg-web/ pkg/
echo "INFO: Previous build artifacts cleaned."

# 1. Build for Bundlers
echo ""
echo "INFO: Building WASM package for BUNDLERS..."
wasm-pack build --target bundler --out-dir pkg-bundler --out-name "${BUNDLER_OUT_NAME}"
echo "INFO: Bundler build complete. Files in pkg-bundler/:"
ls -l pkg-bundler/ # Expecting nucleation.js, nucleation_bg.wasm, nucleation_bg.js, nucleation.d.ts

# 2. Build for Direct Web Use / CDN
echo ""
echo "INFO: Building WASM package for WEB/CDN..."
wasm-pack build --target web --out-dir pkg-web --out-name "${WEB_OUT_NAME}"
echo "INFO: Web build complete. Files in pkg-web/:"
ls -l pkg-web/ # Expecting nucleation-web.js, nucleation-web_bg.wasm, nucleation-web.d.ts (NO _bg.js here)

# 3. Prepare the final 'pkg' directory
echo ""
echo "INFO: Preparing final 'pkg' directory for publishing..."
mkdir -p pkg

# 3a. Copy BUNDLER output (primary npm entries)
echo "INFO: Copying BUNDLER files to pkg/..."
cp pkg-bundler/"${BUNDLER_OUT_NAME}".js pkg/
cp pkg-bundler/"${BUNDLER_OUT_NAME}".d.ts pkg/
cp pkg-bundler/"${BUNDLER_OUT_NAME}"_bg.wasm pkg/
if [ -f pkg-bundler/"${BUNDLER_OUT_NAME}"_bg.js ]; then # Bundler target DOES produce _bg.js
    cp pkg-bundler/"${BUNDLER_OUT_NAME}"_bg.js pkg/
else
    echo "ERROR: BUNDLER JS helper (${BUNDLER_OUT_NAME}_bg.js) not found. This is critical."
    exit 1
fi
cp pkg-bundler/package.json pkg/ # Start with bundler's package.json

# 3b. Copy WEB output
echo "INFO: Copying WEB files to pkg/..."
cp pkg-web/"${WEB_OUT_NAME}".js pkg/         # e.g., nucleation-web.js
cp pkg-web/"${WEB_OUT_NAME}"_bg.wasm pkg/    # e.g., nucleation-web_bg.wasm
# NO _bg.js to copy for web target, as it's inlined in nucleation-web.js
# Optionally copy web's .d.ts if you want separate types or if it's different:
if [ -f pkg-web/"${WEB_OUT_NAME}".d.ts ]; then
    cp pkg-web/"${WEB_OUT_NAME}".d.ts pkg/"${WEB_OUT_NAME}".d.ts
fi

# 4. Modify the package.json (now at pkg/package.json)
echo ""
echo "INFO: Modifying pkg/package.json..."
node -e "
const fs = require('fs');
const pkgPath = './pkg/package.json';
if (!fs.existsSync(pkgPath)) { console.error('ERROR: pkg/package.json does not exist.'); process.exit(1); }
const pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf8'));

pkg.name = '${CRATE_NAME}'; // Ensure crate name is set
pkg.files = [
    '${BUNDLER_OUT_NAME}.js',
    '${BUNDLER_OUT_NAME}_bg.wasm',
    '${BUNDLER_OUT_NAME}_bg.js', // Bundler helper JS
    '${BUNDLER_OUT_NAME}.d.ts',

    '${WEB_OUT_NAME}.js',         // Web main JS
    '${WEB_OUT_NAME}_bg.wasm',    // Web WASM
    // '${WEB_OUT_NAME}.d.ts',    // If you copied web-specific .d.ts
    'README.md'
];
pkg.files = [...new Set(pkg.files)]; // Remove duplicates if any

// Main entries for bundlers (npm/yarn install)
pkg.module = './${BUNDLER_OUT_NAME}.js'; // ES module (for bundlers)
pkg.main = './${BUNDLER_OUT_NAME}.js';   // Could be CommonJS if your bundler output was CJS
pkg.types = './${BUNDLER_OUT_NAME}.d.ts';

// Modern 'exports' field for explicit resolution
pkg.exports = {
  '.': { // For 'import ... from \"nucleation\"'
    'import': './${BUNDLER_OUT_NAME}.js',
    // 'require': './<some_cjs_wrapper_if_you_make_one>.js', // If you provide a CJS wrapper
    'types': './${BUNDLER_OUT_NAME}.d.ts'
  },
  './web': { // For 'import ... from \"nucleation/web\"' or direct CDN to this file
    'import': './${WEB_OUT_NAME}.js',
    'types': './${WEB_OUT_NAME}.d.ts' // Assuming you copy nucleation-web.d.ts
  },
  './package.json': './package.json'
};
// Note: Ensure nucleation-web.d.ts is copied if referenced in exports['./web'].types

fs.writeFileSync(pkgPath, JSON.stringify(pkg, null, 2));
console.log('INFO: Successfully modified pkg/package.json');
"

# 5. Create/Copy README.md into pkg/
if [ ! -f "pkg/README.md" ]; then
  if [ -f "README.md" ]; then cp README.md pkg/README.md;
  else echo \"# ${CRATE_NAME}\" > pkg/README.md; fi
fi

echo ""
echo "--------------------------------------------------------------------"
echo " BUILD COMPLETE"
echo "--------------------------------------------------------------------"
echo "The 'pkg' directory is ready for 'npm publish' (run from within 'pkg/')."
echo "Key files:"
echo "  - ${BUNDLER_OUT_NAME}.js / _bg.wasm / _bg.js (For bundlers, default import)"
echo "  - ${WEB_OUT_NAME}.js / _bg.wasm (For CDN/direct web, import from '${CRATE_NAME}/web' or direct CDN path)"
echo "--------------------------------------------------------------------"