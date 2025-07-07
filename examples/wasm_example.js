/**
 * Nucleation WASM Binding Example
 * 
 * This example demonstrates how to use the nucleation library from JavaScript
 * to create, modify, and save Minecraft schematics in the browser or Node.js.
 * 
 * Prerequisites:
 *   npm install nucleation
 *   
 * or build from source:
 *   ./scripts/build-wasm.sh
 */

import init, { 
    SchematicWrapper, 
    BlockStateWrapper, 
    debug_schematic, 
    debug_json_schematic 
} from 'nucleation';

// Initialize the WASM module
await init();

function basicUsage() {
    console.log("=== Basic Usage ===");
    
    // Create a new empty schematic
    const sch = new SchematicWrapper();
    
    // Add some basic blocks
    sch.set_block(0, 0, 0, "minecraft:stone");
    sch.set_block(1, 0, 0, "minecraft:oak_log");
    sch.set_block(2, 0, 0, "minecraft:diamond_block");
    
    // Add a block with properties
    sch.set_block_with_properties(
        3, 0, 0,
        "minecraft:oak_stairs",
        { facing: "north", half: "bottom" }
    );
    
    // Add a block with NBT data (barrel with redstone signal)
    sch.set_block_from_string(
        4, 0, 0,
        'minecraft:barrel[facing=up]{signal=13}'
    );
    
    // Print schematic info
    const dimensions = sch.get_dimensions();
    console.log(`Dimensions: [${dimensions[0]}, ${dimensions[1]}, ${dimensions[2]}]`);
    console.log(`Block count: ${sch.get_block_count()}`);
    console.log(`Volume: ${sch.get_volume()}`);
    console.log();
    
    return sch;
}

function blockStateDemo() {
    console.log("=== BlockState Demo ===");
    
    // Create basic block state
    const stone = new BlockStateWrapper("minecraft:stone");
    console.log(`Stone: ${stone.name()}`);
    console.log(`Properties:`, stone.properties());
    
    // Create block state with properties
    const stairs = new BlockStateWrapper("minecraft:oak_stairs");
    stairs.with_property("facing", "north");
    stairs.with_property("half", "top");
    
    console.log(`Stairs: ${stairs.name()}`);
    console.log(`Properties:`, stairs.properties());
    console.log();
    
    // Clean up memory
    stone.free();
    stairs.free();
}

function fileOperations() {
    console.log("=== File Operations ===");
    
    // Create a sample schematic
    const sch = new SchematicWrapper();
    
    // Build a small house
    // Floor
    for (let x = 0; x < 5; x++) {
        for (let z = 0; z < 5; z++) {
            sch.set_block(x, 0, z, "minecraft:oak_planks");
        }
    }
    
    // Walls
    for (let i = 0; i < 5; i++) {
        // Front and back walls
        sch.set_block(i, 1, 0, "minecraft:oak_log");
        sch.set_block(i, 1, 4, "minecraft:oak_log");
        sch.set_block(i, 2, 0, "minecraft:oak_log");
        sch.set_block(i, 2, 4, "minecraft:oak_log");
        
        // Side walls
        sch.set_block(0, 1, i, "minecraft:oak_log");
        sch.set_block(4, 1, i, "minecraft:oak_log");
        sch.set_block(0, 2, i, "minecraft:oak_log");
        sch.set_block(4, 2, i, "minecraft:oak_log");
    }
    
    // Add a door
    sch.set_block_with_properties(2, 1, 0, "minecraft:oak_door", { half: "lower" });
    sch.set_block_with_properties(2, 2, 0, "minecraft:oak_door", { half: "upper" });
    
    // Roof
    for (let x = 0; x < 5; x++) {
        for (let z = 0; z < 5; z++) {
            sch.set_block(x, 3, z, "minecraft:oak_planks");
        }
    }
    
    // Get data for saving
    const litematicData = sch.to_litematic();
    const schematicData = sch.to_schematic();
    
    console.log(`Litematic data size: ${litematicData.length} bytes`);
    console.log(`Schematic data size: ${schematicData.length} bytes`);
    
    // In browser environment, you can download files:
    if (typeof window !== 'undefined') {
        downloadFile(litematicData, "js_house.litematic");
        downloadFile(schematicData, "js_house.schematic");
        console.log("Files downloaded");
    } else {
        // In Node.js environment, write to filesystem:
        try {
            const fs = require('fs');
            fs.writeFileSync("js_house.litematic", litematicData);
            fs.writeFileSync("js_house.schematic", schematicData);
            console.log("Files saved to filesystem");
        } catch (e) {
            console.log("Filesystem writing not available");
        }
    }
    
    // Load it back
    const loadedSch = new SchematicWrapper();
    loadedSch.from_data(litematicData); // Auto-detects format
    
    const loadedDims = loadedSch.get_dimensions();
    console.log(`Loaded schematic dimensions: [${loadedDims[0]}, ${loadedDims[1]}, ${loadedDims[2]}]`);
    console.log(`Loaded schematic block count: ${loadedSch.get_block_count()}`);
    console.log();
    
    return loadedSch;
}

function advancedFeatures(sch) {
    console.log("=== Advanced Features ===");
    
    // Query blocks
    const blockAtOrigin = sch.get_block(0, 0, 0);
    if (blockAtOrigin) {
        console.log(`Block at origin: ${blockAtOrigin}`);
    }
    
    // Get block with properties
    const blockWithProps = sch.get_block_with_properties(0, 0, 0);
    if (blockWithProps) {
        console.log(`Block name: ${blockWithProps.name()}`);
        console.log(`Block properties:`, blockWithProps.properties());
        blockWithProps.free(); // Clean up memory
    }
    
    // Get all blocks
    const allBlocks = sch.blocks();
    console.log(`Total blocks in schematic: ${allBlocks.length}`);
    
    // Check for block entities
    const entities = sch.get_all_block_entities();
    if (entities && entities.length > 0) {
        console.log(`Found ${entities.length} block entities`);
        entities.forEach(entity => {
            console.log(`  Entity at [${entity.position}]: ${entity.id}`);
        });
    }
    
    // Print ASCII representation
    console.log("\nASCII representation:");
    console.log(sch.print_schematic());
    
    // Debug info
    console.log("\nDebug info:");
    console.log(debug_schematic(sch));
    console.log();
}

function chunkOperations() {
    console.log("=== Chunk Operations ===");
    
    // Create a larger schematic
    const sch = new SchematicWrapper();
    
    // Fill a 10x10x10 area with different blocks
    for (let x = 0; x < 10; x++) {
        for (let y = 0; y < 10; y++) {
            for (let z = 0; z < 10; z++) {
                if ((x + y + z) % 3 === 0) {
                    sch.set_block(x, y, z, "minecraft:stone");
                } else if ((x + y + z) % 3 === 1) {
                    sch.set_block(x, y, z, "minecraft:dirt");
                } else {
                    sch.set_block(x, y, z, "minecraft:grass_block");
                }
            }
        }
    }
    
    // Get chunks with different strategies
    const chunks = sch.chunks(5, 5, 5);
    console.log(`Generated ${chunks.length} chunks using default strategy`);
    
    const chunksRandom = sch.chunks_with_strategy(5, 5, 5, "random", 0, 0, 0);
    console.log(`Generated ${chunksRandom.length} chunks using random strategy`);
    
    // Get chunks with camera distance
    const chunksCamera = sch.chunks_with_strategy(
        5, 5, 5,
        "distance_to_camera",
        5.0, 5.0, 5.0
    );
    console.log(`Generated ${chunksCamera.length} chunks ordered by camera distance`);
    
    // Get specific chunk blocks
    const chunkBlocks = sch.get_chunk_blocks(0, 0, 0, 5, 5, 5);
    console.log(`Chunk at origin contains ${chunkBlocks.length} blocks`);
    
    console.log();
    
    // Clean up
    sch.free();
}

function copyRegionDemo() {
    console.log("=== Region Copying Demo ===");
    
    // Create source schematic with a pattern
    const source = new SchematicWrapper();
    
    // Create a 3x3x3 pattern
    for (let x = 0; x < 3; x++) {
        for (let y = 0; y < 3; y++) {
            for (let z = 0; z < 3; z++) {
                if (x === 1 && y === 1 && z === 1) {
                    source.set_block(x, y, z, "minecraft:diamond_block");
                } else {
                    source.set_block(x, y, z, "minecraft:stone");
                }
            }
        }
    }
    
    // Create destination schematic
    const dest = new SchematicWrapper();
    
    // Copy the pattern multiple times
    for (let i = 0; i < 3; i++) {
        dest.copy_region(
            source,
            0, 0, 0,  // min coordinates
            2, 2, 2,  // max coordinates
            i * 4, 0, 0,  // target position
            []  // no excluded blocks
        );
    }
    
    console.log(`Copied pattern 3 times. Destination has ${dest.get_block_count()} blocks`);
    
    // Copy again but exclude diamond blocks
    dest.copy_region(
        source,
        0, 0, 0,
        2, 2, 2,
        0, 4, 0,
        ["minecraft:diamond_block"]  // exclude diamonds
    );
    
    console.log(`Copied without diamonds. Destination now has ${dest.get_block_count()} blocks`);
    console.log();
    
    // Clean up
    source.free();
    dest.free();
}

// Utility function for downloading files in browser
function downloadFile(data, filename) {
    if (typeof window === 'undefined') return;
    
    const blob = new Blob([data], { type: 'application/octet-stream' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
}

async function main() {
    console.log("Nucleation WASM Binding Example");
    console.log("=" + "=".repeat(39));
    
    // Run all examples
    const basicSch = basicUsage();
    blockStateDemo();
    const fileSch = fileOperations();
    advancedFeatures(fileSch);
    chunkOperations();
    copyRegionDemo();
    
    console.log("Example completed successfully!");
    
    // Clean up memory
    basicSch.free();
    fileSch.free();
    
    // Clean up generated files in Node.js
    if (typeof window === 'undefined') {
        try {
            const fs = require('fs');
            fs.unlinkSync("js_house.litematic");
            fs.unlinkSync("js_house.schematic");
            console.log("Cleaned up generated files");
        } catch (e) {
            // Files might not exist
        }
    }
}

// Run the example
main().catch(console.error);

// For browser usage, you can also export the functions
if (typeof window !== 'undefined') {
    window.nucleationExample = {
        basicUsage,
        blockStateDemo,
        fileOperations,
        advancedFeatures,
        chunkOperations,
        copyRegionDemo
    };
}
