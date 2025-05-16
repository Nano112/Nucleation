#!/bin/bash
set -e

CRATE_NAME="nucleation"
OUT_NAME="${CRATE_NAME}" # For bundler output files like nucleation_bg.js, nucleation_bg.wasm
CDN_LOADER_FILENAME="${CRATE_NAME}-cdn-loader.js" # Name of our custom loader

echo "INFO: Cleaning previous build artifacts (pkg/)..."
rm -rf pkg/

echo "INFO: Building WASM package (target: bundler, feature: wasm)..."
wasm-pack build --target bundler --out-dir pkg --out-name "${OUT_NAME}" --features wasm
# This creates: pkg/nucleation.js, pkg/nucleation_bg.wasm, pkg/nucleation_bg.js, pkg/nucleation.d.ts

echo "INFO: Bundler build complete. Verifying essential files in pkg/..."
ls -l pkg/
if [ ! -f pkg/"${OUT_NAME}_bg.js" ]; then
    echo "ERROR: pkg/${OUT_NAME}_bg.js (JS helper with class wrappers) is MISSING."
    exit 1
fi
if ! grep -q "class SchematicWrapper" pkg/"${OUT_NAME}_bg.js"; then
    echo "ERROR: SchematicWrapper class not found in pkg/${OUT_NAME}_bg.js."
    exit 1
fi
echo "INFO: Essential JS helper file and SchematicWrapper class found."

echo "INFO: Creating custom CDN loader script (pkg/${CDN_LOADER_FILENAME})..."
cat << EOF > pkg/"${CDN_LOADER_FILENAME}"
// ${CDN_LOADER_FILENAME} - Custom loader for CDN usage
// This file will be imported by the user's HTML script.

// These imports are relative to this loader file within the 'pkg' directory structure on the CDN
import * as bgModule from './${OUT_NAME}_bg.js'; // Imports SchematicWrapper, __wbg_set_wasm, etc.

let initialized = false;
let wasmExports = null;

/**
 * Initializes the WebAssembly module.
 * @param {string | URL | Request | Response | BufferSource} [wasmPathOrModule]
 *   Optional. Path/URL to the '${OUT_NAME}_bg.wasm' file, or a pre-fetched Response/BufferSource.
 *   If not provided, it will attempt to fetch '${OUT_NAME}_bg.wasm' relative to this script.
 * @returns {Promise<object>} A promise that resolves to an object containing the WASM exports
 *                            (e.g., SchematicWrapper, BlockStateWrapper).
 */
async function init(wasmPathOrModule) {
    if (initialized) {
        // console.debug("WASM already initialized. Returning existing exports.");
        return {
            SchematicWrapper: bgModule.SchematicWrapper,
            BlockStateWrapper: bgModule.BlockStateWrapper,
            // Add other direct exports from bgModule you want to expose
            debug_schematic: bgModule.debug_schematic,
            debug_json_schematic: bgModule.debug_json_schematic,
            start: bgModule.start,
            BlockPosition: bgModule.BlockPosition
        };
    }

    if (typeof bgModule.__wbg_set_wasm !== 'function') {
        throw new Error("__wbg_set_wasm function not found in ./${OUT_NAME}_bg.js. The WASM bindings seem incomplete.");
    }

    let input = wasmPathOrModule;
    if (input === undefined) {
        // Default to fetching WASM relative to this loader script's location
        // On CDN, this means same directory, e.g., .../pkg/nucleation_bg.wasm
        input = new URL('./${OUT_NAME}_bg.wasm', import.meta.url);
        // console.debug("No WASM path provided, attempting to load from default relative path:", input.href);
    }

    const importsObject = { './${OUT_NAME}_bg.js': bgModule };
    let instanceExports;

    if (typeof input === 'string' || input instanceof URL || input instanceof Request) {
        // console.debug("Fetching WASM from URL/Request:", input);
        const response = await fetch(input);
        if (!response.ok) {
            throw new Error(\`Failed to fetch WASM (\${input}): \${response.status} \${response.statusText}\`);
        }
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            const { instance } = await WebAssembly.instantiateStreaming(response, importsObject);
            instanceExports = instance.exports;
        } else {
            const buffer = await response.arrayBuffer();
            const { instance } = await WebAssembly.instantiate(buffer, importsObject);
            instanceExports = instance.exports;
        }
    } else if (input instanceof Response) { // Pre-fetched Response object
        // console.debug("Using pre-fetched Response object for WASM.");
         if (typeof WebAssembly.instantiateStreaming === 'function') {
            const { instance } = await WebAssembly.instantiateStreaming(input, importsObject);
            instanceExports = instance.exports;
        } else {
            const buffer = await input.arrayBuffer();
            const { instance } = await WebAssembly.instantiate(buffer, importsObject);
            instanceExports = instance.exports;
        }
    } else if (input instanceof ArrayBuffer || input instanceof Uint8Array) { // Pre-loaded BufferSource
        // console.debug("Using pre-loaded BufferSource for WASM.");
        const { instance } = await WebAssembly.instantiate(input, importsObject);
        instanceExports = instance.exports;
    } else if (input instanceof WebAssembly.Module) { // Pre-compiled Module
        // console.debug("Using pre-compiled WebAssembly.Module.");
         const instance = await WebAssembly.instantiate(input, importsObject);
         instanceExports = instance.exports;
    }
     else {
        throw new TypeError('Invalid input type for WASM module/path to init().');
    }

    bgModule.__wbg_set_wasm(instanceExports);
    // console.debug("WASM instance linked via __wbg_set_wasm.");

    initialized = true;
    wasmExports = {
        SchematicWrapper: bgModule.SchematicWrapper,
        BlockStateWrapper: bgModule.BlockStateWrapper,
        debug_schematic: bgModule.debug_schematic,
        debug_json_schematic: bgModule.debug_json_schematic,
        start: bgModule.start,
        BlockPosition: bgModule.BlockPosition
        // Add any other exports from bgModule that you want to make available
    };
    return wasmExports;
}

// Re-export the named items so users can do:
// import init, { SchematicWrapper } from '.../${CDN_LOADER_FILENAME}';
export {
    SchematicWrapper,
    BlockStateWrapper,
    debug_schematic,
    debug_json_schematic,
    start,
    BlockPosition
} from './${OUT_NAME}_bg.js'; // Re-export directly from _bg.js
                                // These will become "live" after init() is called and __wbg_set_wasm populates the WASM instance.

export default init; // Default export is the init function
EOF

echo "INFO: Custom CDN loader pkg/${CDN_LOADER_FILENAME} created."


echo "INFO: Modifying pkg/package.json to include CDN loader and set up exports..."
node -e "
const fs = require('fs');
const pkgPath = './pkg/package.json';
if (!fs.existsSync(pkgPath)) { console.error('ERROR: pkg/package.json does not exist.'); process.exit(1); }
const pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf8'));

// Files for npm publish
pkg.files = [
    '${OUT_NAME}.js',         // Main bundler entry (e.g., nucleation.js)
    '${OUT_NAME}_bg.wasm',    // Main WASM binary
    '${OUT_NAME}_bg.js',      // JS helper for bundler (contains class defs)
    '${OUT_NAME}.d.ts',       // Main type definitions
    '${CDN_LOADER_FILENAME}', // Our new CDN loader script
    'README.md'
];
pkg.files = [...new Set(pkg.files)];

// Main entries for bundlers
pkg.module = './${OUT_NAME}.js';
pkg.main = './${OUT_NAME}.js';
pkg.types = './${OUT_NAME}.d.ts';
pkg.name = '${CRATE_NAME}';

// Modern 'exports' field for explicit resolution
// This allows 'import ... from \"nucleation/cdn-loader\"' or similar if desired,
// but for CDN, users will typically use the full URL to ${CDN_LOADER_FILENAME}.
pkg.exports = pkg.exports || {}; // Initialize if not present
pkg.exports['.'] = {
    'import': './${OUT_NAME}.js',
    'types': './${OUT_NAME}.d.ts'
};
pkg.exports['./cdn-loader'] = { // Allows 'import init from \"nucleation/cdn-loader\"'
    'import': './${CDN_LOADER_FILENAME}',
    // Add types for cdn-loader if you create a specific .d.ts for it
};
pkg.exports['./package.json'] = './package.json';


// Synchronize version from root Cargo.toml
const cargoTomlPath = require('path').join('..', 'Cargo.toml');
if (!pkg.version && fs.existsSync(cargoTomlPath)) {
    try {
        const cargoTomlContent = fs.readFileSync(cargoTomlPath, 'utf8');
        const versionMatch = cargoTomlContent.match(/^version\s*=\s*\"([^\"]+)\"/m);
        if (versionMatch && versionMatch[1]) { pkg.version = versionMatch[1]; }
    } catch (e) { console.warn('WARN: Could not read version from root Cargo.toml'); }
}

fs.writeFileSync(pkgPath, JSON.stringify(pkg, null, 2));
console.log('INFO: pkg/package.json has been successfully modified.');
"

if [ ! -f "pkg/README.md" ]; then
  if [ -f "../README.md" ]; then cp ../README.md pkg/README.md;
  else echo \"# ${CRATE_NAME}\" > pkg/README.md; fi
fi

echo ""
echo "--------------------------------------------------------------------"
echo " BUILD SCRIPT COMPLETE"
echo "--------------------------------------------------------------------"
echo "The 'pkg/' directory is prepared."
echo "Key files:"
echo "  - pkg/${OUT_NAME}.js, pkg/${OUT_NAME}_bg.wasm, pkg/${OUT_NAME}_bg.js (For bundlers)"
echo "  - pkg/${CDN_LOADER_FILENAME} (New loader for simple CDN usage)"
echo "--------------------------------------------------------------------"
echo "To use from CDN in index.html:"
echo "1. Import from './${CDN_LOADER_FILENAME}' on the CDN:"
echo "   \`import init, { SchematicWrapper, ... } from 'https://cdn.jsdelivr.net/npm/${NPM_PACKAGE_NAME}@VERSION/${CDN_LOADER_FILENAME}';\`"
echo "2. Call \`await init();\` (it can auto-detect its sibling .wasm file)"
echo "   OR \`await init('path/to/${OUT_NAME}_bg.wasm');\` if wasm is elsewhere."
echo "3. Then use \`new SchematicWrapper();\` etc."
echo "--------------------------------------------------------------------"