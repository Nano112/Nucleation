use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use minecraft_schematic_utils::{BlockState, UniversalSchematic, litematic, schematic};
use minecraft_schematic_utils::ChunkLoadingStrategy;
use std::fs;
use std::path::Path;
use std::time::Instant;

// Helper functions
fn create_test_schematic(size: usize) -> UniversalSchematic {
    let mut schematic = UniversalSchematic::new(format!("Benchmark_{}x{}x{}", size, size, size));

    // Create a variety of blocks to test palette performance too
    let block_types = [
        BlockState::new("minecraft:stone"),
        BlockState::new("minecraft:dirt"),
        BlockState::new("minecraft:grass_block"),
        BlockState::new("minecraft:cobblestone"),
        BlockState::new("minecraft:oak_planks"),
    ];

    // Set blocks in the schematic with some patterns to avoid all being the same block
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

fn save_benchmark_schematic(size: usize) -> Vec<u8> {
    let schematic = create_test_schematic(size);
    schematic.to_schematic().expect("Failed to convert to schematic")
}

fn ensure_bench_directory() {
    let path = Path::new("./benches/output");
    if !path.exists() {
        fs::create_dir_all(path).expect("Failed to create benchmark output directory");
    }
}

// Benchmark functions
fn bench_create_schematic(c: &mut Criterion) {
    let mut group = c.benchmark_group("create_schematic");

    for size in [10, 25, 50].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| create_test_schematic(size));
        });
    }

    group.finish();
}

fn bench_save_schematic(c: &mut Criterion) {
    let mut group = c.benchmark_group("save_schematic");
    ensure_bench_directory();

    for size in [10, 25, 50].iter() {
        let schematic = create_test_schematic(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let data = schematic.to_schematic().expect("Failed to convert to schematic");
                black_box(data);
            });
        });
    }

    group.finish();
}

fn bench_save_litematic(c: &mut Criterion) {
    let mut group = c.benchmark_group("save_litematic");
    ensure_bench_directory();

    for size in [10, 25, 50].iter() {
        let schematic = create_test_schematic(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let data = litematic::to_litematic(&schematic).expect("Failed to convert to litematic");
                black_box(data);
            });
        });
    }

    group.finish();
}

fn bench_load_schematic(c: &mut Criterion) {
    let mut group = c.benchmark_group("load_schematic");
    ensure_bench_directory();

    for size in [10, 25, 50].iter() {
        // Create and save a schematic file for each size
        let schematic = create_test_schematic(*size);
        let schem_data = schematic.to_schematic().expect("Failed to convert to schematic");
        let path = format!("./benches/output/bench_{}x{}x{}.schem", size, size, size);
        fs::write(&path, &schem_data).expect("Failed to write benchmark schematic");

        // Read the file for benchmarking
        let schem_data = fs::read(&path).expect("Failed to read benchmark schematic");

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &_size| {
            b.iter(|| {
                let loaded_schematic = schematic::from_schematic(&schem_data).expect("Failed to parse schematic");
                black_box(loaded_schematic);
            });
        });
    }

    group.finish();
}

fn bench_load_litematic(c: &mut Criterion) {
    let mut group = c.benchmark_group("load_litematic");
    ensure_bench_directory();

    for size in [10, 25, 50].iter() {
        // Create and save a litematic file for each size
        let schematic = create_test_schematic(*size);
        let litematic_data = litematic::to_litematic(&schematic).expect("Failed to convert to litematic");
        let path = format!("./benches/output/bench_{}x{}x{}.litematic", size, size, size);
        fs::write(&path, &litematic_data).expect("Failed to write benchmark litematic");

        // Read the file for benchmarking
        let litematic_data = fs::read(&path).expect("Failed to read benchmark litematic");

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &_size| {
            b.iter(|| {
                let loaded_schematic = litematic::from_litematic(&litematic_data).expect("Failed to parse litematic");
                black_box(loaded_schematic);
            });
        });
    }

    group.finish();
}

fn bench_iter_blocks(c: &mut Criterion) {
    let mut group = c.benchmark_group("iter_blocks");

    for size in [10, 25, 50].iter() {
        let schematic = create_test_schematic(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &_size| {
            b.iter(|| {
                // Count blocks to ensure the iterator is fully consumed
                let count = schematic.iter_blocks().count();
                black_box(count);
            });
        });
    }

    group.finish();
}

fn bench_get_block(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_block");

    for size in [10, 25, 50].iter() {
        let schematic = create_test_schematic(*size);
        let size_i32 = *size as i32;

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &_size| {
            b.iter(|| {
                // Access blocks in a pattern that touches various parts of the schematic
                let mut count = 0;
                for x in (0..size_i32).step_by(5) {
                    for y in (0..size_i32).step_by(5) {
                        for z in (0..size_i32).step_by(5) {
                            if let Some(_block) = schematic.get_block(x, y, z) {
                                count += 1;
                            }
                        }
                    }
                }
                black_box(count);
            });
        });
    }

    group.finish();
}

fn bench_iter_chunks(c: &mut Criterion) {
    let mut group = c.benchmark_group("iter_chunks");

    for size in [10, 25, 50].iter() {
        let schematic = create_test_schematic(*size);

        // Test different chunk sizes
        for chunk_size in [4, 8, 16].iter() {
            group.bench_with_input(
                BenchmarkId::new(format!("size_{}", size), format!("chunk_{}", chunk_size)),
                &(*size, *chunk_size),
                |b, &(_, chunk_size)| {
                    b.iter(|| {
                        let mut chunks = Vec::new();
                        for chunk in schematic.iter_chunks(
                            chunk_size as i32,
                            chunk_size as i32,
                            chunk_size as i32,
                            None
                        ) {
                            chunks.push(chunk);
                        }
                        black_box(chunks.len());
                    });
                }
            );
        }

        // Test different loading strategies with a fixed chunk size
        let strategies = [
            ("Default", None),
            ("TopDown", Some(ChunkLoadingStrategy::TopDown)),
            ("BottomUp", Some(ChunkLoadingStrategy::BottomUp)),
            ("CenterOutward", Some(ChunkLoadingStrategy::CenterOutward)),
            ("Random", Some(ChunkLoadingStrategy::Random)),
            ("DistanceToCamera", Some(ChunkLoadingStrategy::DistanceToCamera(25.0, 25.0, 25.0))),
        ];

        for (name, strategy) in strategies.iter() {
            group.bench_with_input(
                BenchmarkId::new(format!("size_{}", size), name),
                &(*size, name, strategy),
                |b, &(_, _, strategy)| {
                    b.iter(|| {
                        let mut chunks = Vec::new();
                        for chunk in schematic.iter_chunks(
                            8, 8, 8,
                            strategy.clone()
                        ) {
                            chunks.push(chunk);
                        }
                        black_box(chunks.len());
                    });
                }
            );
        }
    }

    group.finish();
}

fn bench_count_block_types(c: &mut Criterion) {
    let mut group = c.benchmark_group("count_block_types");

    for size in [10, 25, 50].iter() {
        let schematic = create_test_schematic(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &_size| {
            b.iter(|| {
                let counts = schematic.count_block_types();
                black_box(counts.len());
            });
        });
    }

    group.finish();
}

fn bench_set_block(c: &mut Criterion) {
    let mut group = c.benchmark_group("set_block");

    for size in [10, 25, 50].iter() {
        let size_i32 = *size as i32;

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &_size| {
            b.iter_with_setup(
                || UniversalSchematic::new(format!("Benchmark_Set_{}x{}x{}", size, size, size)),
                |mut schematic| {
                    // Set blocks in a dispersed pattern to test different areas
                    for x in (0..size_i32).step_by(3) {
                        for y in (0..size_i32).step_by(3) {
                            for z in (0..size_i32).step_by(3) {
                                schematic.set_block(x, y, z, BlockState::new("minecraft:stone"));
                            }
                        }
                    }
                }
            );
        });
    }

    group.finish();
}

fn bench_region_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("region_operations");

    for size in [10, 25, 50].iter() {
        let schematic = create_test_schematic(*size);
        let region_name = "Main";

        // Benchmark getting a region
        group.bench_with_input(
            BenchmarkId::new("get_region", size),
            size,
            |b, &_size| {
                b.iter(|| {
                    let region = schematic.get_region(region_name);
                    black_box(region);
                });
            }
        );

        // Benchmark get_merged_region
        group.bench_with_input(
            BenchmarkId::new("get_merged_region", size),
            size,
            |b, &_size| {
                b.iter(|| {
                    let merged_region = schematic.get_merged_region();
                    black_box(merged_region);
                });
            }
        );
    }

    group.finish();
}

fn bench_create_schematic_from_region(c: &mut Criterion) {
    let mut group = c.benchmark_group("create_schematic_from_region");

    for size in [10, 25].iter() {  // Use smaller sizes for this more expensive operation
        let schematic = create_test_schematic(*size);
        let bbox = schematic.get_bounding_box();

        // Benchmark creating a new schematic from a region
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            size,
            |b, &_size| {
                b.iter(|| {
                    let new_schematic = schematic.create_schematic_from_region(&bbox);
                    black_box(new_schematic);
                });
            }
        );
    }

    group.finish();
}

fn bench_memory_usage() {
    // This function demonstrates how to track memory usage,
    // but we'll use an external crate for actual memory profiling

    // Create increasingly large schematics and measure memory
    for size in [25, 50, 100].iter() {
        println!("Creating {}x{}x{} schematic...", size, size, size);

        // Record memory before
        let before = std::process::Command::new("ps")
            .args(&["-o", "rss=", &format!("{}", std::process::id())])
            .output()
            .expect("Failed to get memory usage");
        let before_kb: i32 = String::from_utf8_lossy(&before.stdout)
            .trim()
            .parse()
            .expect("Failed to parse memory usage");

        // Create schematic
        let start = Instant::now();
        let schematic = create_test_schematic(*size);
        let duration = start.elapsed();

        // Record memory after
        let after = std::process::Command::new("ps")
            .args(&["-o", "rss=", &format!("{}", std::process::id())])
            .output()
            .expect("Failed to get memory usage");
        let after_kb: i32 = String::from_utf8_lossy(&after.stdout)
            .trim()
            .parse()
            .expect("Failed to parse memory usage");

        // Report results
        let memory_delta_mb = (after_kb - before_kb) as f64 / 1024.0;
        println!(
            "Size: {}x{}x{}, Time: {:?}, Memory: {:.2} MB",
            size, size, size, duration, memory_delta_mb
        );
    }
}

criterion_group!(
    benches,
    bench_create_schematic,
    bench_save_schematic,
    bench_save_litematic,
    bench_load_schematic,
    bench_load_litematic,
    bench_iter_blocks,
    bench_get_block,
    bench_iter_chunks,
    bench_count_block_types,
    bench_set_block,
    bench_region_operations,
    bench_create_schematic_from_region,
);
criterion_main!(benches);

// Add this to your Cargo.toml:
// [dev-dependencies]
// criterion = { version = "0.5", features = ["html_reports"] }
// 
// [dependencies]
// # For memory profiling
// jemallocator = "0.5"
// jemalloc-ctl = "0.5"
// 
// [[bench]]
// name = "schematic_benchmark"
// harness = false