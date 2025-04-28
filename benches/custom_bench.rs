use clap::Parser;
use std::time::Instant;
use minecraft_schematic_utils::{BlockState, UniversalSchematic};


#[derive(Parser)]
#[command(name = "custom_bench", version = "1.0", author = "Your Name")]
struct Args {
    /// Edge size of the region to fill
    #[arg(short, long = "edge", default_value_t = 10)]
    edge: i32,

    /// Starting size of the schematic
    #[arg(short, long = "start-size", default_value_t = 0)]
    start_size: i32,

    /// Offset position for block placement
    #[arg(short, long = "offset", default_value_t = 0)]
    offset: i32,
}



fn main() {
    let args = Args::parse();

    let mut sch = UniversalSchematic::new("bench".to_string());

    // Optional: Pre-expand schematic to starting size
    if args.start_size > 0 {
        let air = BlockState::new("minecraft:air");
        for x in 0..args.start_size {
            for y in 0..args.start_size {
                for z in 0..args.start_size {
                    sch.set_block(x, y, z, air.clone());
                }
            }
        }
    }

    let stone = BlockState::new("minecraft:stone");

    let start = Instant::now();

    for i in 0..args.edge * args.edge {
        let x = (i % args.edge) as i32 + args.offset;
        let y = (i / args.edge) as i32 + args.offset;
        let z = 0 + args.offset;
        sch.set_block(x, y, z, stone.clone());
    }
    
    // save the schematic to a file
    //convert to a .schem
    let schem_data = sch.to_schematic().expect("Failed to convert to schematic");
    let output_path = format!("output/filled_schematic.schem");
    let schem_path = std::path::Path::new(&output_path);
    std::fs::write(schem_path, &schem_data).expect("Failed to write schematic file");
    println!("Schematic saved to {}", output_path);

    let duration = start.elapsed();
    println!("Time taken: {:?}", duration);
}
