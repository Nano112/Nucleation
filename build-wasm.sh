#!/bin/bash
set -e

CRATE_NAME="nucleation"
OUT_NAME="${CRATE_NAME}"
CDN_LOADER_FILENAME="${CRATE_NAME}-cdn-loader.js"

echo "INFO: Cleaning previous build artifacts (pkg/)..."
rm -rf pkg/

echo "INFO: Building WASM package (target: bundler)..."
wasm-pack build --target bundler --out-dir pkg --out-name "${OUT_NAME}" --features wasm

# --- Entry Point for Bundlers (Promise Proxy) ---
echo "INFO: Overwriting pkg/${OUT_NAME}.js with a promise-based entry point for bundlers..."
cat << EOF > pkg/"${OUT_NAME}.js"
import init, * as wasm from './${OUT_NAME}_bg.js';

// The default export is a promise that resolves with the initialized wasm exports.
// Bundlers will automatically resolve the path to the wasm file.
const wasmPromise = init().then(() => wasm);
export default wasmPromise;
EOF

echo "INFO: Creating type definitions for the promise-based API (pkg/${OUT_NAME}.d.ts)..."
# This overwrites the wasm-pack generated .d.ts to match our promise export
cat << EOF > pkg/"${OUT_NAME}.d.ts"
import * as wasm from './${OUT_NAME}_bg';
export * from './${OUT_NAME}_bg';
declare const wasmPromise: Promise<typeof wasm>;
export default wasmPromise;
EOF

# --- Entry Point for CDN Usage ---
# We use your original, excellent CDN loader script. It remains unchanged.
# It correctly assumes it will be in the same directory as the _bg.wasm file on the CDN.
echo "INFO: Creating custom CDN loader script (pkg/${CDN_LOADER_FILENAME})..."
cat << EOF > pkg/"${CDN_LOADER_FILENAME}"
// ${CDN_LOADER_FILENAME} - Custom loader for CDN usage
import * as bgModule from './${OUT_NAME}_bg.js';

let initialized = false;

// The init function is the default export for this file.
async function init(wasmPathOrModule) {
    if (initialized) {
        return { ...bgModule };
    }
    if (typeof bgModule.__wbg_set_wasm !== 'function') {
        throw new Error("Missing __wbg_set_wasm in ./${OUT_NAME}_bg.js");
    }

    let input = wasmPathOrModule;
    if (input === undefined) {
        input = new URL('./${OUT_NAME}_bg.wasm', import.meta.url);
    }

    const importsObject = { './${OUT_NAME}_bg.js': bgModule };
    const { instance } = await WebAssembly.instantiateStreaming(fetch(input), importsObject);

    bgModule.__wbg_set_wasm(instance.exports);
    initialized = true;

    // Return all the exports from the bg module
    return { ...bgModule };
}

// Re-export the named items so users can do:
// import init, { SchematicWrapper } from '.../loader.js';
// This works because after init() is called, these exports become "live".
export * from './${OUT_NAME}_bg.js';
export default init;
EOF

# --- Final package.json Wiring ---
echo "INFO: Modifying pkg/package.json to support both bundler and CDN use cases..."
node -e "
const fs = require('fs');
const pkgPath = './pkg/package.json';
const pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf8'));

pkg.files = [
    '${OUT_NAME}.js',
    '${OUT_NAME}_bg.wasm',
    '${OUT_NAME}_bg.js',
    '${OUT_NAME}.d.ts',
    '${OUT_NAME}_bg.d.ts',
    '${CDN_LOADER_FILENAME}',
    'README.md'
];

pkg.module = './${OUT_NAME}.js';
pkg.main = './${OUT_NAME}.js';
pkg.types = './${OUT_NAME}.d.ts';
pkg.name = '${CRATE_NAME}';

// Use the exports field to define clear paths for each use case
pkg.exports = {
    // For bundlers: 'import nucleation from \"nucleation\"'
    '.': {
        'import': './${OUT_NAME}.js',
        'types': './${OUT_NAME}.d.ts'
    },
    // For CDN: 'import init from \"nucleation/cdn-loader\"'
    './cdn-loader': {
        'import': './${CDN_LOADER_FILENAME}'
        // You could create a cdn-loader.d.ts here if you wanted, but it's complex.
        // For now, consumers of the CDN loader will likely not have strong typing on init.
    },
    './package.json': './package.json'
};

// ... your version sync logic ...

fs.writeFileSync(pkgPath, JSON.stringify(pkg, null, 2));
console.log('INFO: pkg/package.json has been successfully modified.');
"

if [ ! -f "pkg/README.md" ]; then
  if [ -f "../README.md" ]; then cp ../README.md pkg/README.md; fi
fi

echo ""
echo "--------------------------------------------------------------------"
echo " BUILD SCRIPT COMPLETE - Unified"
echo "--------------------------------------------------------------------"
echo "✅ For BUNDLERS (Vite, Webpack, etc.):"
echo "   import nucleation from 'nucleation';"
echo "   const { SchematicWrapper } = await nucleation;"
echo ""
echo "✅ For CDN (<script type=module>):"
echo "   import init, { SchematicWrapper } from 'https://cdn.jsdelivr.net/npm/${CRATE_NAME}@VERSION/${CDN_LOADER_FILENAME}';"
echo "   await init();"
echo "   const schematic = new SchematicWrapper();"
echo "--------------------------------------------------------------------"