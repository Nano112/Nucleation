//! benches/schematic.rs
use criterion::{
    black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion,
};
use minecraft_schematic_utils::{BlockState, ChunkLoadingStrategy, UniversalSchematic};

/// -------------------------------------------------------------------------
/// Helpers
/// -------------------------------------------------------------------------
fn generate_filled_schematic(edge: i32) -> UniversalSchematic {
    let mut sch = UniversalSchematic::new(format!("{}³-filled", edge));
    // FIX: Remove .into()
    let stone = BlockState::new("minecraft:stone");

    // (~ edge³ ) writes – enough to exercise palette & region expansion,
    // but still < ½ second in debug so benches stay quick.
    for x in 0..edge {
        for y in 0..edge {
            for z in 0..edge {
                sch.set_block(x, y, z, stone.clone());
            }
        }
    }
    sch
}

/// -------------------------------------------------------------------------
/// Individual benchmarks
/// -------------------------------------------------------------------------

/// 1. Just construct an *empty* schematic (baseline allocation cost).
fn bench_create_empty(c: &mut Criterion) {
    c.bench_function("create empty schematic", |b| {
        // Assuming UniversalSchematic::new also takes Into<Arc<str>> or similar
        // If it takes String, use "bench".to_string()
        // If it takes &str, use "bench"
        // If it takes Arc<str>, use "bench".into() or Arc::from("bench")
        // Based on the pattern, let's assume Into<Arc<str>> is likely
        b.iter(|| black_box(UniversalSchematic::new("bench".to_string())));
    });
}

/// 2. Write 10 000 blocks at once (exercises palette look-ups &
///    automatic region growth).
fn bench_place_10k_blocks(c: &mut Criterion) {
    c.bench_function("place 10 000 blocks", |b| {
        b.iter(|| {
            // Assuming UniversalSchematic::new takes Into<Arc<str>> or similar
            let mut sch = UniversalSchematic::new("bench".to_string());
            // FIX: Remove .into()
            let stone = BlockState::new("minecraft:stone");

            for i in 0..10_000 {
                let x = (i % 20) as i32;
                let y = ((i / 20) % 20) as i32;
                let z = (i / 400) as i32;
                sch.set_block(x, y, z, stone.clone());
            }
            black_box(sch);
        })
    });
}

/// 3. Region *expansion* cost: place a block farther and farther away so
///    every write forces a grow-and-copy inside the backing `Region`.
fn bench_region_expansion(c: &mut Criterion) {
    c.bench_function("progressive region expansion", |b| {
        b.iter(|| {
            // Assuming UniversalSchematic::new takes Into<Arc<str>> or similar
            let mut sch = UniversalSchematic::new("expand".to_string());
            // FIX: Remove .into()
            let stone = BlockState::new("minecraft:stone");

            for i in 0..1_000 {
                // (-i, -i, -i) walks the negative octant, stressing both growth
                // directions each time.
                let n = -(i as i32);
                sch.set_block(n, n, n, stone.clone());
            }
            black_box(sch);
        })
    });
}

/// 4. Iterating every block (hot path for analytics / conversions).
fn bench_iter_blocks(c: &mut Criterion) {
    let sch = generate_filled_schematic(32); // 32³ ≈ 32k blocks

    c.bench_function("iter_blocks (32³ filled)", |b| {
        b.iter(|| {
            // Sum the palette indices so the loop isn’t optimised away.
            let mut acc = 0usize;
            for (_, bstate) in sch.iter_blocks() {
                acc = acc.wrapping_add(bstate.get_name().len());
            }
            black_box(acc)
        })
    });
}

/// 5. Chunk splitting & iteration for several loading strategies.
///    Uses the same 32³ schematic to keep the numbers comparable.
fn bench_chunk_iteration(c: &mut Criterion) {
    let sch = generate_filled_schematic(64); // 64³ ≈ 262k blocks
    let mut group = c.benchmark_group("chunk iteration 64³");

    // Common closure so we don’t duplicate the actual timed body.
    let bench = |id: &str,
                 group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
                 strategy: Option<ChunkLoadingStrategy>| {
        group.bench_function(BenchmarkId::new("iterate_chunks", id), |b| {
            b.iter(|| {
                // Count chunks so optimisations can’t elide the work.
                let n = sch
                    .iter_chunks(16, 256, 16, strategy.clone())
                    .count();
                black_box(n);
            })
        });
    };

    bench("default", &mut group, None);
    bench(
        "center_outward",
        &mut group,
        Some(ChunkLoadingStrategy::CenterOutward),
    );
    bench(
        "top_down",
        &mut group,
        Some(ChunkLoadingStrategy::TopDown),
    );
    bench(
        "distance_to_camera",
        &mut group,
        Some(ChunkLoadingStrategy::DistanceToCamera(0.0, 80.0, 0.0)),
    );

    group.finish();
}

/// -------------------------------------------------------------------------
/// Criterion entry-points
/// -------------------------------------------------------------------------
criterion_group!(
    benches,
    bench_create_empty,
    bench_place_10k_blocks,
    bench_region_expansion,
    bench_iter_blocks,
    bench_chunk_iteration
);
criterion_main!(benches);
