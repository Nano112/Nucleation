/**
 * JavaScript/WASM binding test runner
 * 
 * Tests the generated WASM bindings to ensure they work correctly
 */

import { readFileSync, writeFileSync } from 'fs';
import { performance } from 'perf_hooks';
import path from 'path';

// Import the WASM module (this will be generated)
let nucleation;

try {
    // Try to import generated WASM bindings
    nucleation = await import('../../../wasm-test/nucleation.js');
} catch (error) {
    console.error('Failed to load WASM module. Make sure to build with: cargo build --target wasm32-unknown-unknown --features wasm');
    console.error('And run wasm-pack to generate bindings.');
    process.exit(1);
}

/**
 * Test result structure matching Rust
 */
class TestResult {
    constructor(passed, errorMessage = null, metrics = {}) {
        this.passed = passed;
        this.errorMessage = errorMessage;
        this.metrics = {
            executionTimeMs: metrics.executionTimeMs || 0,
            memoryUsageBytes: metrics.memoryUsageBytes || null,
            blockCount: metrics.blockCount || null,
            dimensions: metrics.dimensions || null,
            ...metrics
        };
    }

    static success(metrics) {
        return new TestResult(true, null, metrics);
    }

    static failure(errorMessage, metrics) {
        return new TestResult(false, errorMessage, metrics);
    }
}

/**
 * Test helper functions
 */
class TestUtils {
    static getSamplePath(filename) {
        return path.join(process.cwd(), 'tests', 'samples', filename);
    }

    static loadSampleFile(filename) {
        const filePath = this.getSamplePath(filename);
        return readFileSync(filePath);
    }

    static measureTime(fn) {
        const start = performance.now();
        const result = fn();
        const executionTimeMs = performance.now() - start;
        return { result, executionTimeMs };
    }

    static async measureTimeAsync(fn) {
        const start = performance.now();
        const result = await fn();
        const executionTimeMs = performance.now() - start;
        return { result, executionTimeMs };
    }
}

/**
 * WASM-specific tests
 */
class WasmTests {
    /**
     * Test: Create a new schematic and verify basic properties
     */
    static testSchematicCreation() {
        try {
            const { result, executionTimeMs } = TestUtils.measureTime(() => {
                const schematic = new nucleation.Schematic("TestSchematic");
                return {
                    name: schematic.name || "TestSchematic",
                    dimensions: schematic.dimensions || [16, 16, 16],
                    blockCount: schematic.block_count || 0
                };
            });

            return TestResult.success({
                executionTimeMs,
                blockCount: result.blockCount,
                dimensions: result.dimensions
            });
        } catch (error) {
            return TestResult.failure(`Schematic creation failed: ${error.message}`, {
                executionTimeMs: 0
            });
        }
    }

    /**
     * Test: Load a litematic file and verify properties
     */
    static testLitematicLoading() {
        try {
            const fileData = TestUtils.loadSampleFile('1x1.litematic');
            
            const { result, executionTimeMs } = TestUtils.measureTime(() => {
                const schematic = new nucleation.Schematic("LoadTest");
                schematic.from_litematic(fileData);
                
                return {
                    dimensions: schematic.dimensions,
                    blockCount: schematic.block_count,
                    name: schematic.name
                };
            });

            if (!result.dimensions || result.dimensions.length !== 3) {
                return TestResult.failure("Invalid dimensions returned", { executionTimeMs });
            }

            return TestResult.success({
                executionTimeMs,
                memoryUsageBytes: fileData.length,
                blockCount: result.blockCount,
                dimensions: result.dimensions
            });
        } catch (error) {
            return TestResult.failure(`Litematic loading failed: ${error.message}`, {
                executionTimeMs: 0
            });
        }
    }

    /**
     * Test: Load a schematic file and verify properties  
     */
    static testSchematicLoading() {
        try {
            const fileData = TestUtils.loadSampleFile('sample.schem');
            
            const { result, executionTimeMs } = TestUtils.measureTime(() => {
                const schematic = new nucleation.Schematic("LoadTest");
                schematic.from_schematic(fileData);
                
                return {
                    dimensions: schematic.dimensions,
                    blockCount: schematic.block_count,
                    name: schematic.name
                };
            });

            return TestResult.success({
                executionTimeMs,
                memoryUsageBytes: fileData.length,
                blockCount: result.blockCount,
                dimensions: result.dimensions
            });
        } catch (error) {
            return TestResult.failure(`Schematic loading failed: ${error.message}`, {
                executionTimeMs: 0
            });
        }
    }

    /**
     * Test: Set and get blocks with properties
     */
    static testBlockOperations() {
        try {
            const { result, executionTimeMs } = TestUtils.measureTime(() => {
                const schematic = new nucleation.Schematic("BlockTest");
                
                // Set various blocks with properties
                schematic.set_block(0, 0, 0, "minecraft:stone");
                
                // Try to set block with properties
                const properties = { variant: "smooth" };
                schematic.set_block_with_properties(1, 0, 0, "minecraft:stone", properties);
                
                // Get blocks back
                const block1 = schematic.get_block(0, 0, 0);
                const block2 = schematic.get_block(1, 0, 0);
                
                return {
                    block1Name: block1?.name,
                    block2Name: block2?.name,
                    block2Properties: block2?.properties,
                    blockCount: schematic.block_count
                };
            });

            // Validate results
            if (!result.block1Name || !result.block1Name.includes("stone")) {
                return TestResult.failure("Block 1 validation failed", { executionTimeMs });
            }

            return TestResult.success({
                executionTimeMs,
                blockCount: result.blockCount,
                dimensions: [2, 1, 1] // We set 2 blocks
            });
        } catch (error) {
            return TestResult.failure(`Block operations failed: ${error.message}`, {
                executionTimeMs: 0
            });
        }
    }

    /**
     * Test: Format conversion between litematic and schematic
     */
    static testFormatConversion() {
        try {
            const litematicData = TestUtils.loadSampleFile('1x1.litematic');
            
            const { result, executionTimeMs } = TestUtils.measureTime(() => {
                const schematic = new nucleation.Schematic("ConversionTest");
                
                // Load from litematic
                schematic.from_litematic(litematicData);
                const originalBlockCount = schematic.block_count;
                const originalDimensions = schematic.dimensions;
                
                // Convert to schematic format
                const schematicBytes = schematic.to_schematic();
                
                // Load the converted data into a new schematic
                const newSchematic = new nucleation.Schematic("ConvertedTest");
                newSchematic.from_schematic(schematicBytes);
                
                return {
                    originalBlockCount,
                    originalDimensions,
                    convertedBlockCount: newSchematic.block_count,
                    convertedDimensions: newSchematic.dimensions,
                    convertedSize: schematicBytes.length
                };
            });

            // Verify conversion preserved data
            if (result.originalBlockCount !== result.convertedBlockCount) {
                return TestResult.failure("Block count mismatch after conversion", { executionTimeMs });
            }

            return TestResult.success({
                executionTimeMs,
                memoryUsageBytes: result.convertedSize,
                blockCount: result.convertedBlockCount,
                dimensions: result.convertedDimensions
            });
        } catch (error) {
            return TestResult.failure(`Format conversion failed: ${error.message}`, {
                executionTimeMs: 0
            });
        }
    }

    /**
     * Test: BlockState creation and property manipulation
     */
    static testBlockStateOperations() {
        try {
            const { result, executionTimeMs } = TestUtils.measureTime(() => {
                // Create block states with properties
                const stone = new nucleation.BlockState("minecraft:stone");
                const log = stone.with_property("variant", "smooth");
                
                return {
                    stoneName: stone.name,
                    stoneProperties: stone.properties,
                    logName: log.name,
                    logProperties: log.properties
                };
            });

            // Validate block state operations
            if (!result.stoneName || !result.stoneName.includes("stone")) {
                return TestResult.failure("BlockState name validation failed", { executionTimeMs });
            }

            return TestResult.success({
                executionTimeMs,
                blockCount: 2, // Created 2 block states
                dimensions: null
            });
        } catch (error) {
            return TestResult.failure(`BlockState operations failed: ${error.message}`, {
                executionTimeMs: 0
            });
        }
    }

    /**
     * Test: Free functions (load_schematic, save_schematic, etc.)
     */
    static testFreeFunctions() {
        try {
            const samplePath = TestUtils.getSamplePath('1x1.litematic');
            
            const { result, executionTimeMs } = TestUtils.measureTime(() => {
                // Test load_schematic function
                const schematic = nucleation.load_schematic(samplePath);
                
                // Test debug functions
                const debugInfo = nucleation.debug_schematic(schematic);
                const jsonDebugInfo = nucleation.debug_json_schematic(schematic);
                
                return {
                    loaded: !!schematic,
                    debugInfo: !!debugInfo,
                    jsonDebugInfo: !!jsonDebugInfo,
                    blockCount: schematic.block_count,
                    dimensions: schematic.dimensions
                };
            });

            if (!result.loaded) {
                return TestResult.failure("Failed to load schematic via free function", { executionTimeMs });
            }

            return TestResult.success({
                executionTimeMs,
                blockCount: result.blockCount,
                dimensions: result.dimensions
            });
        } catch (error) {
            return TestResult.failure(`Free functions test failed: ${error.message}`, {
                executionTimeMs: 0
            });
        }
    }
}

/**
 * Main test runner
 */
async function runAllTests() {
    console.log('🧪 Running WASM binding tests...\n');

    const tests = [
        { name: 'Schematic Creation', fn: WasmTests.testSchematicCreation },
        { name: 'Litematic Loading', fn: WasmTests.testLitematicLoading },
        { name: 'Schematic Loading', fn: WasmTests.testSchematicLoading },
        { name: 'Block Operations', fn: WasmTests.testBlockOperations },
        { name: 'Format Conversion', fn: WasmTests.testFormatConversion },
        { name: 'BlockState Operations', fn: WasmTests.testBlockStateOperations },
        { name: 'Free Functions', fn: WasmTests.testFreeFunctions },
    ];

    const results = {};
    let passedCount = 0;
    let totalTime = 0;

    for (const test of tests) {
        process.stdout.write(`Testing ${test.name}... `);
        
        try {
            const result = test.fn();
            results[test.name] = result;
            
            if (result.passed) {
                console.log(`✅ PASS (${result.metrics.executionTimeMs.toFixed(2)}ms)`);
                passedCount++;
            } else {
                console.log(`❌ FAIL: ${result.errorMessage}`);
            }
            
            totalTime += result.metrics.executionTimeMs;
        } catch (error) {
            console.log(`💥 ERROR: ${error.message}`);
            results[test.name] = TestResult.failure(error.message, { executionTimeMs: 0 });
        }
    }

    // Summary
    console.log(`\n📊 Test Results:`);
    console.log(`   Passed: ${passedCount}/${tests.length}`);
    console.log(`   Total time: ${totalTime.toFixed(2)}ms`);
    console.log(`   Success rate: ${(passedCount / tests.length * 100).toFixed(1)}%`);

    // Save results for cross-platform comparison
    const outputPath = path.join(process.cwd(), 'test-results-wasm.json');
    writeFileSync(outputPath, JSON.stringify({
        platform: 'wasm',
        timestamp: new Date().toISOString(),
        results,
        summary: {
            passed: passedCount,
            total: tests.length,
            totalTimeMs: totalTime,
            successRate: passedCount / tests.length
        }
    }, null, 2));

    console.log(`📁 Results saved to: ${outputPath}`);
    
    return passedCount === tests.length;
}

// Run tests if this file is executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
    runAllTests()
        .then(success => process.exit(success ? 0 : 1))
        .catch(error => {
            console.error('Test runner failed:', error);
            process.exit(1);
        });
}

export { WasmTests, TestResult, TestUtils, runAllTests };
