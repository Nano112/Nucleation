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
    let size = 1000;

    // 1. Create a large schematic
    println!("Creating schematic of size {}x{}x{}", size, size, size);
    let start = Instant::now();
    let schematic = create_test_schematic(size);
    println!("Creation took {:?}", start.elapsed());

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