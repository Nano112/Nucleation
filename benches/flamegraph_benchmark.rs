// Run with:
// cargo install flamegraph
// cargo flamegraph --bin flamegraph_benchmark

use minecraft_schematic_utils::{BlockState, UniversalSchematic, litematic, schematic};
use minecraft_schematic_utils::ChunkLoadingStrategy;
use std::fs;
use std::path::Path;
use std::time::Instant;

fn create_test_schematic(size: usize) -> UniversalSchematic {
    let mut schematic = UniversalSchematic::new(format!("Benchmark_{}x{}x{}", size, size, size));

    // Create a variety of blocks for a more realistic test
    let block_types = [
        BlockState::new("minecraft:stone"),
        BlockState::new("minecraft:dirt"),
        BlockState::new("minecraft:grass_block"),
        BlockState::new("minecraft:cobblestone"),
        BlockState::new("minecraft:oak_planks"),
    ];

    // Set blocks in the schematic with some patterns
    for x in 0..size {
        for y in 0..size {
            for z in 0..size {
                let block_idx = (x + y + z) % block_types.len();
                schematic.set_block(x as i32, y as i32, z as i32, block_types[block_idx].clone());
            }
        }
    }

    schematic
}

fn benchmark_block_access_patterns(schematic: &UniversalSchematic, size: i32) {
    use rand::prelude::*;

    println!("\n===== BLOCK ACCESS PATTERN BENCHMARKS =====");

    // 1. Random Sampling Test
    println!("\n1. Random Sampling Test");
    {
        // We'll use a fixed seed for reproducibility
        let seed = 42;
        let mut rng = StdRng::seed_from_u64(seed);

        // Calculate number of samples (scaled based on size)
        let samples = std::cmp::min(1_000_000, (size as usize).pow(3) / 10);

        let start = Instant::now();
        let mut count = 0;

        for _ in 0..samples {
            // Fixed: Use the correct gen_range syntax with separate arguments
            let x = rng.gen_range(0, size);
            let y = rng.gen_range(0, size);
            let z = rng.gen_range(0, size);

            if schematic.get_block(x, y, z).is_some() {
                count += 1;
            }
        }

        let duration = start.elapsed();
        let blocks_per_second = (count as f64 / duration.as_secs_f64()) as u64;

        println!("  - Randomly sampled {} blocks in {:?}", count, duration);
        println!("  - Performance: {} blocks per second", blocks_per_second);
    }

    // 2. Sequential Iteration Test
    println!("\n2. Sequential Iteration Test");
    {
        // For large schematics, use stepping to avoid excessive runtime
        let step = if size > 100 { std::cmp::max(1, size / 100) } else { 1 };

        let start = Instant::now();
        let mut count = 0;

        for x in (0..size).step_by(step as usize) {
            for y in (0..size).step_by(step as usize) {
                for z in (0..size).step_by(step as usize) {
                    if schematic.get_block(x, y, z).is_some() {
                        count += 1;
                    }
                }
            }
        }

        let duration = start.elapsed();
        let blocks_per_second = (count as f64 / duration.as_secs_f64()) as u64;

        println!("  - Sequentially accessed {} blocks (step size = {}) in {:?}",
                 count, step, duration);

        // Calculate extrapolated full-volume rate
        if step > 1 {
            let extrapolated_rate = blocks_per_second * (step as u64).pow(3);
            println!("  - Extrapolated performance: {} blocks per second",
                     extrapolated_rate);
        } else {
            println!("  - Performance: {} blocks per second", blocks_per_second);
        }
    }

    // 3. Chunk-then-Block Test
    println!("\n3. Chunk-then-Block Test");
    {
        let chunk_size = 16; // Standard Minecraft chunk size

        // Time how long it takes to split into chunks
        let start = Instant::now();
        let chunks = schematic.split_into_chunks(chunk_size, chunk_size, chunk_size);
        let split_duration = start.elapsed();

        println!("  - Split into {} chunks in {:?}", chunks.len(), split_duration);

        // Time the actual block access
        let start = Instant::now();
        let mut count = 0;

        for chunk in &chunks {
            // Calculate chunk bounds
            let chunk_min_x = chunk.chunk_x * chunk_size;
            let chunk_min_y = chunk.chunk_y * chunk_size;
            let chunk_min_z = chunk.chunk_z * chunk_size;

            let chunk_max_x = chunk_min_x + chunk_size - 1;
            let chunk_max_y = chunk_min_y + chunk_size - 1;
            let chunk_max_z = chunk_min_z + chunk_size - 1;

            // Process all blocks in the chunk
            for x in chunk_min_x..=chunk_max_x {
                for y in chunk_min_y..=chunk_max_y {
                    for z in chunk_min_z..=chunk_max_z {
                        if schematic.get_block(x, y, z).is_some() {
                            count += 1;
                        }
                    }
                }
            }
        }

        let iteration_duration = start.elapsed();
        let total_duration = split_duration + iteration_duration;
        let blocks_per_second = (count as f64 / iteration_duration.as_secs_f64()) as u64;

        println!("  - Accessed {} blocks via chunks in {:?}", count, iteration_duration);
        println!("  - Total time (split + access): {:?}", total_duration);
        println!("  - Performance: {} blocks per second", blocks_per_second);
    }

    // Additional test for built-in iterator (for comparison)
    println!("\n4. Built-in Iterator Test");
    {
        let start = Instant::now();
        let count = schematic.iter_blocks().count();
        let duration = start.elapsed();
        let blocks_per_second = (count as f64 / duration.as_secs_f64()) as u64;

        println!("  - Iterated through {} blocks using iter_blocks() in {:?}",
                 count, duration);
        println!("  - Performance: {} blocks per second", blocks_per_second);
    }

    println!("\n==========================================\n");
}
fn ensure_bench_directory() {
    let path = Path::new("./benches/output");
    if !path.exists() {
        fs::create_dir_all(path).expect("Failed to create benchmark output directory");
    }
}

fn main() {
    println!("Starting flamegraph benchmark - this will profile all operations");
    ensure_bench_directory();

    // Define a larger schematic size to make hotspots more visible
    let size = 500;

    // 1. Create a large schematic
    println!("Creating schematic of size {}x{}x{}", size, size, size);
    let start = Instant::now();
    let schematic = create_test_schematic(size);
    println!("Creation took {:?}", start.elapsed());

    println!("\nRunning block access pattern benchmarks...");
    benchmark_block_access_patterns(&schematic, size as i32);

    // 2. Export as .schem
    println!("Exporting to .schem format");
    let start = Instant::now();
    let schem_data = schematic.to_schematic().expect("Failed to convert to schematic");
    println!("Export to .schem took {:?}", start.elapsed());

    // Save for loading test
    let path = format!("./benches/output/flame_{}x{}x{}.schem", size, size, size);
    fs::write(&path, &schem_data).expect("Failed to write benchmark schematic");

    // 3. Export as .litematic
    println!("Exporting to .litematic format");
    let start = Instant::now();
    let litematic_data = litematic::to_litematic(&schematic).expect("Failed to convert to litematic");
    println!("Export to .litematic took {:?}", start.elapsed());

    // Save for loading test
    let path = format!("./benches/output/flame_{}x{}x{}.litematic", size, size, size);
    fs::write(&path, &litematic_data).expect("Failed to write benchmark litematic");

    // 4. Count block types (likely HashMap bottlenecks)
    println!("Counting block types");
    let start = Instant::now();
    let block_counts = schematic.count_block_types();
    println!("Counting took {:?}, found {} unique blocks", start.elapsed(), block_counts.len());

    // 5. Iterate over all blocks
    println!("Iterating over all blocks");
    let start = Instant::now();
    let block_count = schematic.iter_blocks().count();
    println!("Iteration took {:?}, found {} blocks", start.elapsed(), block_count);

    // 6. Loading from files
    println!("Loading from .schem file");
    let schem_path = format!("./benches/output/flame_{}x{}x{}.schem", size, size, size);
    let schem_data = fs::read(&schem_path).expect("Failed to read benchmark schematic");

    let start = Instant::now();
    let loaded_schematic = schematic::from_schematic(&schem_data).expect("Failed to parse schematic");
    println!("Loading .schem took {:?}", start.elapsed());

    println!("Loading from .litematic file");
    let litematic_path = format!("./benches/output/flame_{}x{}x{}.litematic", size, size, size);
    let litematic_data = fs::read(&litematic_path).expect("Failed to read benchmark litematic");

    let start = Instant::now();
    let loaded_litematic = litematic::from_litematic(&litematic_data).expect("Failed to parse litematic");
    println!("Loading .litematic took {:?}", start.elapsed());

    // 7. Test different chunk iterators with different strategies
    println!("Testing chunk operations");

    let start = Instant::now();
    let chunks = schematic.split_into_chunks(8, 8, 8);
    println!("Splitting into chunks took {:?}, found {} chunks", start.elapsed(), chunks.len());

    // Test different loading strategies
    println!("Testing different chunk loading strategies");
    let strategies = [
        ("Default", None),
        ("TopDown", Some(ChunkLoadingStrategy::TopDown)),
        ("BottomUp", Some(ChunkLoadingStrategy::BottomUp)),
        ("CenterOutward", Some(ChunkLoadingStrategy::CenterOutward)),
        ("Random", Some(ChunkLoadingStrategy::Random)),
        ("DistanceToCamera", Some(ChunkLoadingStrategy::DistanceToCamera(
            size as f32 / 2.0,
            size as f32 / 2.0,
            size as f32 / 2.0
        ))),
    ];

    for (name, strategy) in &strategies {
        let start = Instant::now();
        let mut chunks = Vec::new();
        for chunk in schematic.iter_chunks(8, 8, 8, strategy.clone()) {
            chunks.push(chunk);
        }
        println!("Strategy {} took {:?}, found {} chunks", name, start.elapsed(), chunks.len());
    }

    // 8. Get merged region (helps identify region merge bottlenecks)
    println!("Getting merged region");
    let start = Instant::now();
    let merged_region = schematic.get_merged_region();
    println!("Merging regions took {:?}", start.elapsed());

    // 9. Test create_schematic_from_region
    println!("Creating schematic from region");
    let bbox = schematic.get_bounding_box();
    let start = Instant::now();
    let new_schematic = schematic.create_schematic_from_region(&bbox);
    println!("Creating from region took {:?}", start.elapsed());

    // 10. Copy region test (likely a common operation)
    println!("Testing copy_region operation");
    let mut target_schematic = UniversalSchematic::new("Target".to_string());
    let small_bbox = {
        let full_bbox = schematic.get_bounding_box();
        let size_i32 = size as i32;
        let mid = size_i32 / 2;
        let quarter = size_i32 / 4;
        let min = (mid - quarter, mid - quarter, mid - quarter);
        let max = (mid + quarter, mid + quarter, mid + quarter);
        minecraft_schematic_utils::bounding_box::BoundingBox::new(min, max)
    };

    let start = Instant::now();
    let result = target_schematic.copy_region(&schematic, &small_bbox, (0, 0, 0), &[]);
    println!("Copy region took {:?}, result: {:?}", start.elapsed(), result);

    // // 11. Test to_nbt serialization
    // println!("Testing to_nbt serialization");
    // let start = Instant::now();
    // let nbt = schematic.to_nbt();
    // println!("to_nbt took {:?}", start.elapsed());

    // 12. Test from_layers - pattern creation (if available)
    if size <= 20 { // Keep this manageable
        println!("Testing from_layers with a simple pattern");
        let layers = [
            "XXX",
            "XXX",
            "XXX"
        ].join("\n\n");

        let block_mappings = [
            (&'X', ("stone", vec![]))
        ];

        let start = Instant::now();
        let layer_schematic = UniversalSchematic::from_layers(
            "Layer Test".to_string(),
            &block_mappings,
            &layers
        );
        println!("from_layers took {:?}", start.elapsed());
    }

    // 13. Test get_json_string
    println!("Testing get_json_string serialization");
    let start = Instant::now();
    let json_result = schematic.get_json_string();
    match json_result {
        Ok(json) => println!("get_json_string took {:?}, JSON length: {} bytes",
                             start.elapsed(), json.len()),
        Err(e) => println!("get_json_string failed: {:?}", e),
    }

    // 14. Measure block access performance
    println!("Testing get_block performance");
    let start = Instant::now();
    let size_i32 = size as i32;
    let mut count = 0;
    for x in (0..size_i32).step_by(2) {
        for y in (0..size_i32).step_by(2) {
            for z in (0..size_i32).step_by(2) {
                if schematic.get_block(x, y, z).is_some() {
                    count += 1;
                }
            }
        }
    }
    println!("Accessed {} blocks in {:?}", count, start.elapsed());

    println!("Flamegraph benchmark complete");
}