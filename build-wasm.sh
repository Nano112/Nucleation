#!/bin/bash
set -e # Exit on any error

CRATE_NAME="nucleation" # Your crate name, used for wasm-pack's default output names
WEB_OUT_NAME="${CRATE_NAME}-web" # Distinct name for the web-targeted JS/WASM

# Clean previous build artifacts
echo "Cleaning previous build artifacts..."
rm -rf pkg-bundler/ pkg-web/ pkg/

# 1. Build for bundlers (this will be the primary entry in package.json)
echo "Building WASM package for bundlers (e.g., Webpack, Vite, Parcel, Bun build)..."
wasm-pack build --target bundler --out-dir pkg-bundler --out-name "${CRATE_NAME}"
# Expected output: pkg-bundler/nucleation.js, pkg-bundler/nucleation_bg.wasm, pkg-bundler/nucleation.d.ts, pkg-bundler/package.json

# 2. Build for direct web use (ES Modules in browser, CDN)
echo "Building WASM package for direct web use (ES Modules)..."
wasm-pack build --target web --out-dir pkg-web --out-name "${WEB_OUT_NAME}"
# Expected output: pkg-web/nucleation-web.js, pkg-web/nucleation-web_bg.wasm, pkg-web/nucleation-web.d.ts

# 3. Prepare the final 'pkg' directory for publishing
echo "Preparing final 'pkg' directory for publishing..."
mkdir -p pkg

# Copy primary (bundler) files to pkg/
echo "Copying bundler-targeted files to pkg/..."
cp pkg-bundler/"${CRATE_NAME}".js pkg/
cp pkg-bundler/"${CRATE_NAME}"_bg.wasm pkg/ # This is the main WASM binary
cp pkg-bundler/"${CRATE_NAME}".d.ts pkg/
cp pkg-bundler/package.json pkg/ # Start with bundler's package.json

# Copy web-targeted JavaScript file to pkg/
# The web-targeted JS is designed to load its WASM.
# It will look for <WEB_OUT_NAME>_bg.wasm.
# For simplicity, we'll assume the WASM binary is identical for both targets.
# If nucleation-web.js needs nucleation-web_bg.wasm specifically,
# you would copy pkg-web/nucleation-web_bg.wasm to pkg/nucleation-web_bg.wasm
# and add it to package.json files.
# Here, we assume nucleation-web.js can be made to load nucleation_bg.wasm (see JS init call).
echo "Copying web-targeted JavaScript to pkg/..."
cp pkg-web/"${WEB_OUT_NAME}".js pkg/
# Optionally, copy its .d.ts if you want separate types for the web version
# cp pkg-web/"${WEB_OUT_NAME}".d.ts pkg/"${WEB_OUT_NAME}".d.ts

# 4. Modify the package.json (now at pkg/package.json)
echo "Modifying pkg/package.json..."
node -e "
const fs = require('fs');
const pkgPath = './pkg/package.json'; // Path relative to this script's execution (project root)

if (!fs.existsSync(pkgPath)) {
  console.error('Error: pkg/package.json does not exist. Bundler build might have failed.');
  process.exit(1);
}

const pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf8'));

// Define files to be included in the npm package
// These paths are relative to the 'pkg/' directory because 'npm publish' will be run from 'pkg/'
pkg.files = [
    '${CRATE_NAME}.js',          // Main JS for bundlers
    '${CRATE_NAME}_bg.wasm',     // The core WASM binary
    '${CRATE_NAME}.d.ts',        // Main TypeScript definitions
    '${WEB_OUT_NAME}.js',        // JS for direct web/CDN use
    // '${WEB_OUT_NAME}.d.ts',   // Optional: TS definitions for web version if copied
    'README.md'                 // Include README
];

// Define entry points
pkg.module = '${CRATE_NAME}.js';    // ES module entry for bundlers
pkg.main = '${CRATE_NAME}.js';      // CommonJS/Default entry (often same as module for modern libs)
pkg.types = '${CRATE_NAME}.d.ts';   // Main TypeScript definitions

// Optional: Add a 'browser' field or use 'exports' for more specific resolution
// pkg.browser = '${WEB_OUT_NAME}.js'; // Older way to specify browser entry
// Example 'exports' (more modern, check Node.js/bundler compatibility):
// pkg.exports = {
//   '.': {
//     'import': './${CRATE_NAME}.js',    // For ESM bundlers
//     'require': './${CRATE_NAME}.js',   // For CJS bundlers (if your .js is UMD or CJS compatible)
//     'types': './${CRATE_NAME}.d.ts'
//   },
//   './web': {
//     'import': './${WEB_OUT_NAME}.js',
//     // 'types': './${WEB_OUT_NAME}.d.ts' // if you have separate web types
//   },
//   './package.json': './package.json' // Allow access to package.json
// };


// Ensure other essential fields are present or add them
pkg.name = '${CRATE_NAME}'; // Ensure package name is correct
// pkg.version will be set by CI or manually before publish
// pkg.description = 'Your package description';
// pkg.repository = { type: 'git', url: 'your_repo_url' };
// pkg.keywords = ['wasm', 'rust', 'your', 'keywords'];
// pkg.author = 'Your Name';
// pkg.license = 'MIT OR Apache-2.0';

fs.writeFileSync(pkgPath, JSON.stringify(pkg, null, 2));
console.log('Successfully modified pkg/package.json');
"

# 5. Create a README.md in pkg/ if it doesn't exist (or copy from project root)
if [ ! -f "pkg/README.md" ]; then
  if [ -f "README.md" ]; then
    echo "Copying project README.md to pkg/"
    cp README.md pkg/README.md
  else
    echo "Creating a basic README.md in pkg/"
    echo \"# ${CRATE_NAME}\" > pkg/README.md
    echo \"\" >> pkg/README.md
    echo \"WebAssembly library built from Rust.\" >> pkg/README.md
  fi
fi

# For local testing: copy to wasm-test if it exists
if [ -d "wasm-test" ]; then
  echo "Copying files to wasm-test directory for local testing..."
  cp pkg/"${CRATE_NAME}".js pkg/"${CRATE_NAME}"_bg.wasm pkg/"${WEB_OUT_NAME}".js wasm-test/
fi

echo ""
echo "Build complete!"
echo "The 'pkg' directory is ready for 'npm publish' (run from within 'pkg/')."
echo "It includes:"
echo "  - ${CRATE_NAME}.js (for bundlers)"
echo "  - ${CRATE_NAME}_bg.wasm (main WASM binary)"
echo "  - ${WEB_OUT_NAME}.js (for direct browser/CDN use)"
echo "  - package.json (configured for publishing)"
echo "  - README.md"