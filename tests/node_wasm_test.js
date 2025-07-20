// Node.js test script for WASM chunk iterator functionality
// This allows for more detailed testing and debugging outside the browser

const fs = require('fs');
const path = require('path');

// You'll need to adjust this path based on where your built WASM is located
const wasmPath = path.join(__dirname, '../pkg/nucleation.js');

async function runTests() {
    let nucleation;
    
    try {
        nucleation = require(wasmPath);
        await nucleation.default(); // Initialize WASM
        console.log('✅ WASM module loaded successfully');
    } catch (error) {
        console.error('❌ Failed to load WASM module:', error);
        console.log('Make sure to build the WASM package first with: ./build-wasm.sh');
        process.exit(1);
    }

    const { SchematicWrapper } = nucleation;

    // Helper function to create test schematic
    function createTestSchematic() {
        const schematic = new SchematicWrapper();
        
        // Create a 4x4x4 cube with some variety
        for (let x = 0; x < 4; x++) {
            for (let y = 0; y < 4; y++) {
                for (let z = 0; z < 4; z++) {
                    if (x === 0 || x === 3 || y === 0 || y === 3 || z === 0 || z === 3) {
                        // Walls are stone
                        schematic.set_block(x, y, z, "minecraft:stone");
                    } else {
                        // Interior has different blocks
                        schematic.set_block(x, y, z, "minecraft:air");
                    }
                }
            }
        }
        
        // Add some distinctive blocks
        schematic.set_block(1, 1, 1, "minecraft:diamond_block");
        schematic.set_block(2, 1, 1, "minecraft:emerald_block");
        schematic.set_block(1, 2, 1, "minecraft:gold_block");
        schematic.set_block(2, 2, 1, "minecraft:iron_block");
        schematic.set_block(1, 1, 2, "minecraft:redstone_block");
        
        return schematic;
    }

    // Helper function to load real test data
    function loadTestSchematic() {
        const testFiles = [
            '../tests/samples/1x1.litematic',
            '../tests/samples/3x3.litematic',
            '../simple_cube.litematic'
        ];
        
        for (const file of testFiles) {
            const filePath = path.join(__dirname, file);
            if (fs.existsSync(filePath)) {
                try {
                    const data = fs.readFileSync(filePath);
                    const schematic = new SchematicWrapper();
                    schematic.from_data(new Uint8Array(data));
                    console.log(`✅ Loaded test schematic from ${file}`);
                    return schematic;
                } catch (error) {
                    console.log(`⚠️  Failed to load ${file}: ${error.message}`);
                }
            }
        }
        
        console.log('📝 Using generated test schematic');
        return createTestSchematic();
    }

    console.log('\n=== Running WASM Chunk Iterator Tests ===\n');

    // Test 1: Basic chunk functionality
    console.log('🧪 Test 1: Basic chunk functionality');
    const schematic = createTestSchematic();
    
    const chunks = schematic.chunks(2, 2, 2);
    console.log(`   - Generated ${chunks.length} chunks with 2x2x2 size`);
    
    if (chunks.length > 0) {
        const firstChunk = chunks[0];
        console.log(`   - First chunk at (${firstChunk.chunk_x}, ${firstChunk.chunk_y}, ${firstChunk.chunk_z})`);
        console.log(`   - First chunk has ${firstChunk.blocks.length} blocks`);
        
        if (firstChunk.blocks.length > 0) {
            const firstBlock = firstChunk.blocks[0];
            console.log(`   - First block: (${firstBlock.x}, ${firstBlock.y}, ${firstBlock.z}) = ${firstBlock.name}`);
        }
    }

    // Test 2: Chunk indices optimization
    console.log('\n🧪 Test 2: Chunk indices optimization');
    const chunksIndices = schematic.chunks_indices(2, 2, 2);
    console.log(`   - Regular chunks: ${chunks.length}, Indexed chunks: ${chunksIndices.length}`);
    
    if (chunksIndices.length > 0) {
        const firstIndexChunk = chunksIndices[0];
        console.log(`   - First indexed chunk has ${firstIndexChunk.blocks.length} blocks`);
        
        if (firstIndexChunk.blocks.length > 0) {
            const firstIndexBlock = firstIndexChunk.blocks[0];
            console.log(`   - First indexed block: [${firstIndexBlock.join(', ')}] (x,y,z,palette_idx)`);
        }
        
        // Get palettes to understand the indices
        const allPalettes = schematic.get_all_palettes();
        console.log(`   - Default palette has ${allPalettes.default.length} entries`);
        
        // Show first few palette entries
        for (let i = 0; i < Math.min(5, allPalettes.default.length); i++) {
            console.log(`   - Palette[${i}]: ${allPalettes.default[i].name}`);
        }
    }

    // Test 3: Loading strategies
    console.log('\n🧪 Test 3: Loading strategies');
    const strategies = ['bottom_up', 'top_down', 'distance_to_camera', 'center_outward', 'random'];
    
    for (const strategy of strategies) {
        const strategyChunks = schematic.chunks_with_strategy(2, 2, 2, strategy, 0, 0, 0);
        console.log(`   - Strategy '${strategy}': ${strategyChunks.length} chunks`);
        
        if (strategyChunks.length > 0) {
            const positions = strategyChunks.map(chunk => `(${chunk.chunk_x},${chunk.chunk_y},${chunk.chunk_z})`);
            console.log(`     Order: ${positions.join(' -> ')}`);
        }
    }

    // Test 4: Lazy chunk iterator
    console.log('\n🧪 Test 4: Lazy chunk iterator');
    const iterator = schematic.create_lazy_chunk_iterator(2, 2, 2, 'bottom_up', 0, 0, 0);
    console.log(`   - Total chunks available: ${iterator.total_chunks()}`);
    
    const retrievedChunks = [];
    let iterations = 0;
    const maxIterations = 20; // Safety limit
    
    while (iterator.has_next() && iterations < maxIterations) {
        const chunk = iterator.next();
        if (chunk !== null) {
            retrievedChunks.push({
                position: `(${chunk.chunk_x},${chunk.chunk_y},${chunk.chunk_z})`,
                blocks: chunk.blocks.length,
                index: chunk.index,
                total: chunk.total
            });
        }
        iterations++;
    }
    
    console.log(`   - Retrieved ${retrievedChunks.length} chunks through lazy iterator`);
    retrievedChunks.forEach((chunk, i) => {
        console.log(`     ${i}: ${chunk.position} - ${chunk.blocks} blocks [${chunk.index}/${chunk.total}]`);
    });
    
    // Test iterator controls
    iterator.reset();
    console.log(`   - After reset, position: ${iterator.current_position()}, has_next: ${iterator.has_next()}`);
    
    if (iterator.total_chunks() > 2) {
        iterator.skip_to(Math.floor(iterator.total_chunks() / 2));
        console.log(`   - After skip to middle, position: ${iterator.current_position()}`);
    }

    // Test 5: Data integrity and false values detection
    console.log('\n🧪 Test 5: Data integrity and false values detection');
    
    // Reset for clean test
    iterator.reset();
    const allBlocks = [];
    const chunkData = [];
    
    while (iterator.has_next()) {
        const chunk = iterator.next();
        if (chunk === null) {
            console.log('   ❌ ERROR: Iterator returned null chunk!');
            break;
        }
        
        const chunkInfo = {
            position: [chunk.chunk_x, chunk.chunk_y, chunk.chunk_z],
            blockCount: chunk.blocks.length,
            blocks: []
        };
        
        // Analyze each block in the chunk
        for (let i = 0; i < chunk.blocks.length; i++) {
            const blockData = chunk.blocks[i];
            
            // Validate block data structure
            if (!Array.isArray(blockData) || blockData.length !== 4) {
                console.log(`   ❌ ERROR: Invalid block data structure at chunk ${chunk.chunk_x},${chunk.chunk_y},${chunk.chunk_z}, block ${i}`);
                console.log(`     Expected array of length 4, got:`, blockData);
                continue;
            }
            
            const [x, y, z, paletteIndex] = blockData;
            
            // Validate coordinate values
            if (typeof x !== 'number' || typeof y !== 'number' || typeof z !== 'number') {
                console.log(`   ❌ ERROR: Non-numeric coordinates: (${x}, ${y}, ${z})`);
                continue;
            }
            
            // Validate palette index
            if (typeof paletteIndex !== 'number' || paletteIndex < 0 || paletteIndex > 1000) {
                console.log(`   ❌ ERROR: Invalid palette index: ${paletteIndex} at (${x}, ${y}, ${z})`);
                continue;
            }
            
            // Check for obviously wrong values (this is where you might catch "false" values)
            if (paletteIndex !== Math.floor(paletteIndex)) {
                console.log(`   ⚠️  WARNING: Non-integer palette index: ${paletteIndex} at (${x}, ${y}, ${z})`);
            }
            
            const blockInfo = { x, y, z, paletteIndex };
            chunkInfo.blocks.push(blockInfo);
            allBlocks.push(blockInfo);
        }
        
        chunkData.push(chunkInfo);
    }
    
    console.log(`   - Analyzed ${chunkData.length} chunks with ${allBlocks.length} total blocks`);
    
    // Check for duplicates
    const positionMap = new Map();
    const duplicates = [];
    
    allBlocks.forEach((block, index) => {
        const key = `${block.x},${block.y},${block.z}`;
        if (positionMap.has(key)) {
            duplicates.push({
                position: key,
                firstIndex: positionMap.get(key),
                duplicateIndex: index,
                firstBlock: allBlocks[positionMap.get(key)],
                duplicateBlock: block
            });
        } else {
            positionMap.set(key, index);
        }
    });
    
    if (duplicates.length > 0) {
        console.log(`   ❌ ERROR: Found ${duplicates.length} duplicate blocks:`);
        duplicates.forEach(dup => {
            console.log(`     Position ${dup.position}: indices ${dup.firstIndex} and ${dup.duplicateIndex}`);
            console.log(`       First: palette ${dup.firstBlock.paletteIndex}, Duplicate: palette ${dup.duplicateBlock.paletteIndex}`);
        });
    } else {
        console.log('   ✅ No duplicate blocks found');
    }
    
    // Palette consistency check
    const allPalettes = schematic.get_all_palettes();
    const paletteSize = allPalettes.default.length;
    const invalidIndices = allBlocks.filter(block => block.paletteIndex >= paletteSize);
    
    if (invalidIndices.length > 0) {
        console.log(`   ❌ ERROR: Found ${invalidIndices.length} blocks with invalid palette indices:`);
        invalidIndices.slice(0, 5).forEach(block => {
            console.log(`     (${block.x}, ${block.y}, ${block.z}): index ${block.paletteIndex} >= palette size ${paletteSize}`);
        });
        if (invalidIndices.length > 5) {
            console.log(`     ... and ${invalidIndices.length - 5} more`);
        }
    } else {
        console.log('   ✅ All palette indices are valid');
    }
    
    // Test 6: Performance comparison
    console.log('\n🧪 Test 6: Performance comparison');
    
    const iterations_perf = 10;
    
    // Time regular chunks method
    const start1 = Date.now();
    for (let i = 0; i < iterations_perf; i++) {
        schematic.chunks(2, 2, 2);
    }
    const time1 = Date.now() - start1;
    
    // Time indexed chunks method
    const start2 = Date.now();
    for (let i = 0; i < iterations_perf; i++) {
        schematic.chunks_indices(2, 2, 2);
    }
    const time2 = Date.now() - start2;
    
    // Time lazy iterator
    const start3 = Date.now();
    for (let i = 0; i < iterations_perf; i++) {
        const iter = schematic.create_lazy_chunk_iterator(2, 2, 2, 'bottom_up', 0, 0, 0);
        while (iter.has_next()) {
            iter.next();
        }
    }
    const time3 = Date.now() - start3;
    
    console.log(`   - Regular chunks: ${time1}ms (${iterations_perf} iterations)`);
    console.log(`   - Indexed chunks: ${time2}ms (${iterations_perf} iterations)`);
    console.log(`   - Lazy iterator: ${time3}ms (${iterations_perf} iterations)`);
    console.log(`   - Indexed chunks are ${(time1/time2).toFixed(2)}x faster than regular`);
    console.log(`   - Lazy iterator vs indexed: ${(time3/time2).toFixed(2)}x ratio`);

    // Test 7: Real world scenario with larger schematic
    console.log('\n🧪 Test 7: Real world scenario');
    const realSchematic = loadTestSchematic();
    
    const dimensions = realSchematic.get_dimensions();
    const blockCount = realSchematic.get_block_count();
    console.log(`   - Schematic dimensions: ${dimensions[0]}x${dimensions[1]}x${dimensions[2]}`);
    console.log(`   - Total blocks: ${blockCount}`);
    
    if (blockCount > 0) {
        const realChunks = realSchematic.chunks_indices(8, 8, 8);
        console.log(`   - Divided into ${realChunks.length} chunks (8x8x8)`);
        
        let totalRealBlocks = 0;
        realChunks.forEach(chunk => {
            totalRealBlocks += chunk.blocks.length;
        });
        
        console.log(`   - Total blocks in chunks: ${totalRealBlocks}`);
        
        // Test lazy loading on real data
        const realIterator = realSchematic.create_lazy_chunk_iterator(4, 4, 4, 'distance_to_camera', 0, 0, 0);
        console.log(`   - Lazy iterator reports ${realIterator.total_chunks()} chunks (4x4x4)`);
        
        let realChunkCount = 0;
        let realBlockCount = 0;
        while (realIterator.has_next() && realChunkCount < 10) { // Limit for testing
            const chunk = realIterator.next();
            if (chunk && chunk.blocks) {
                realBlockCount += chunk.blocks.length;
            }
            realChunkCount++;
        }
        
        console.log(`   - First 10 lazy chunks contain ${realBlockCount} blocks`);
    }

    console.log('\n=== Test Summary ===');
    console.log('✅ All basic functionality tests completed');
    console.log('📊 Check the output above for any ❌ ERROR messages');
    console.log('🔍 Pay attention to palette index validation and duplicate detection');
    
    if (duplicates.length > 0 || invalidIndices.length > 0) {
        console.log('\n⚠️  ISSUES DETECTED:');
        if (duplicates.length > 0) console.log(`   - ${duplicates.length} duplicate blocks found`);
        if (invalidIndices.length > 0) console.log(`   - ${invalidIndices.length} invalid palette indices found`);
        console.log('   This suggests there may be issues with the chunk iterator implementation.');
        process.exit(1);
    } else {
        console.log('\n🎉 No major issues detected! The chunk iterator appears to be working correctly.');
    }
}

// Run the tests
runTests().catch(error => {
    console.error('❌ Test failed:', error);
    process.exit(1);
});
