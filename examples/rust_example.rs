/*!
 * Nucleation Rust API Example
 * 
 * This example demonstrates how to use the nucleation library from Rust
 * to create, modify, and save Minecraft schematics.
 * 
 * Run with:
 *   cargo run --example rust_example
 *   
 * Or compile and run:
 *   rustc --extern nucleation examples/rust_example.rs
 *   ./rust_example
 */

use nucleation::{
    UniversalSchematic, BlockState, BoundingBox, ChunkLoadingStrategy,
    litematic,
};
use std::collections::HashMap;
use std::fs;

fn print_separator(title: &str) {
    println!("=== {} ===", title);
}

fn basic_usage() -> Result<UniversalSchematic, Box<dyn std::error::Error>> {
    print_separator("Basic Usage");
    
    // Create a new empty schematic
    let mut sch = UniversalSchematic::new("Rust Demo".to_string());
    
    // Add some basic blocks
    sch.set_block(0, 0, 0, BlockState::new("minecraft:stone".to_string()));
    sch.set_block(1, 0, 0, BlockState::new("minecraft:oak_log".to_string()));
    sch.set_block(2, 0, 0, BlockState::new("minecraft:diamond_block".to_string()));
    
    // Add a block with properties
    let mut properties = HashMap::new();
    properties.insert("facing".to_string(), "north".to_string());
    properties.insert("half".to_string(), "bottom".to_string());
    let stairs_block = BlockState::new("minecraft:oak_stairs".to_string()).with_properties(properties);
    sch.set_block(3, 0, 0, stairs_block);
    
    // Print schematic info
    let dimensions = sch.get_dimensions();
    println!("Dimensions: {:?}", dimensions);
    let block_count = sch.get_blocks().len();
    println!("Block count: {}", block_count);
    let (width, height, length) = dimensions;
    let volume = width * height * length;
    println!("Volume: {}", volume);
    println!();
    
    Ok(sch)
}

fn block_state_demo() -> Result<(), Box<dyn std::error::Error>> {
    print_separator("BlockState Demo");
    
    // Create basic block state
    let stone = BlockState::new("minecraft:stone".to_string());
    println!("Stone: {}", stone.get_name());
    println!("Properties: {:?}", stone.properties);
    
    // Create block state with properties
    let mut stairs = BlockState::new("minecraft:oak_stairs".to_string());
    stairs.set_property("facing".to_string(), "north".to_string());
    stairs.set_property("half".to_string(), "top".to_string());
    
    println!("Stairs: {}", stairs.get_name());
    println!("Properties: {:?}", stairs.properties);
    
    // Using the fluent interface
    let complex_block = BlockState::new("minecraft:redstone_wire".to_string())
        .with_property("north".to_string(), "side".to_string())
        .with_property("south".to_string(), "up".to_string())
        .with_property("power".to_string(), "15".to_string());
        
    println!("Complex block: {}", complex_block.get_name());
    println!("Properties: {:?}", complex_block.properties);
    println!();
    
    Ok(())
}

fn file_operations() -> Result<UniversalSchematic, Box<dyn std::error::Error>> {
    print_separator("File Operations");
    
    // Create a sample schematic
    let mut sch = UniversalSchematic::new("Rust File Demo".to_string());
    
    // Build a small house
    // Floor
    for x in 0..5 {
        for z in 0..5 {
            sch.set_block(x, 0, z, BlockState::new("minecraft:oak_planks".to_string()));
        }
    }
    
    // Walls
    for i in 0..5 {
        // Front and back walls
        sch.set_block(i, 1, 0, BlockState::new("minecraft:oak_log".to_string()));
        sch.set_block(i, 1, 4, BlockState::new("minecraft:oak_log".to_string()));
        sch.set_block(i, 2, 0, BlockState::new("minecraft:oak_log".to_string()));
        sch.set_block(i, 2, 4, BlockState::new("minecraft:oak_log".to_string()));
        
        // Side walls
        sch.set_block(0, 1, i, BlockState::new("minecraft:oak_log".to_string()));
        sch.set_block(4, 1, i, BlockState::new("minecraft:oak_log".to_string()));
        sch.set_block(0, 2, i, BlockState::new("minecraft:oak_log".to_string()));
        sch.set_block(4, 2, i, BlockState::new("minecraft:oak_log".to_string()));
    }
    
    // Add a door
    let mut door_props_lower = HashMap::new();
    door_props_lower.insert("half".to_string(), "lower".to_string());
    let door_lower = BlockState::new("minecraft:oak_door".to_string()).with_properties(door_props_lower);
    sch.set_block(2, 1, 0, door_lower);
    
    let mut door_props_upper = HashMap::new();
    door_props_upper.insert("half".to_string(), "upper".to_string());
    let door_upper = BlockState::new("minecraft:oak_door".to_string()).with_properties(door_props_upper);
    sch.set_block(2, 2, 0, door_upper);
    
    // Roof
    for x in 0..5 {
        for z in 0..5 {
            sch.set_block(x, 3, z, BlockState::new("minecraft:oak_planks".to_string()));
        }
    }
    
    // Save as different formats
    let litematic_data = litematic::to_litematic(&sch)?;
    fs::write("rust_house.litematic", litematic_data)?;
    println!("Saved as rust_house.litematic");
    
    let schematic_data = sch.to_schematic()?;
    fs::write("rust_house.schematic", schematic_data)?;
    println!("Saved as rust_house.schematic");
    
    // Load it back
    let loaded_data = fs::read("rust_house.litematic")?;
    let loaded_sch = litematic::from_litematic(&loaded_data)?;
    
    println!("Loaded schematic dimensions: {:?}", loaded_sch.get_dimensions());
    let loaded_block_count = loaded_sch.get_blocks().len();
    println!("Loaded schematic block count: {}", loaded_block_count);
    println!();
    
    Ok(loaded_sch)
}

fn advanced_features(sch: &UniversalSchematic) -> Result<(), Box<dyn std::error::Error>> {
    print_separator("Advanced Features");
    
    // Query blocks
    if let Some(block_at_origin) = sch.get_block(0, 0, 0) {
        println!("Block at origin: {}", block_at_origin.get_name());
        println!("Properties: {:?}", block_at_origin.properties);
    }
    
    // Get all blocks
    let all_blocks = sch.get_blocks();
    println!("Total blocks in schematic: {}", all_blocks.len());
    
    // Check for block entities
    let entities = sch.get_block_entities_as_list();
    if !entities.is_empty() {
        println!("Found {} block entities", entities.len());
        for entity in &entities {
            println!("  Entity at {:?}: {}", entity.position, entity.id);
        }
    }
    
    // JSON debug info
    println!("\nJSON debug (first 200 chars):");
    let json_debug = sch.get_json_string()?;
    println!("{}...", &json_debug[..200.min(json_debug.len())]);
    println!();
    
    Ok(())
}

fn chunk_operations() -> Result<(), Box<dyn std::error::Error>> {
    print_separator("Chunk Operations");
    
    // Create a larger schematic
    let mut sch = UniversalSchematic::new("Rust Chunk Demo".to_string());
    
    // Fill a 10x10x10 area with different blocks
    for x in 0..10 {
        for y in 0..10 {
            for z in 0..10 {
                let block_type = match (x + y + z) % 3 {
                    0 => BlockState::new("minecraft:stone".to_string()),
                    1 => BlockState::new("minecraft:dirt".to_string()),
                    _ => BlockState::new("minecraft:grass_block".to_string()),
                };
                sch.set_block(x, y, z, block_type);
            }
        }
    }
    
    // Get chunks with different strategies
    let chunks: Vec<_> = sch.iter_chunks(5, 5, 5, Some(ChunkLoadingStrategy::BottomUp)).collect();
    println!("Generated {} chunks using BottomUp strategy", chunks.len());
    
    let chunks_random: Vec<_> = sch.iter_chunks(5, 5, 5, Some(ChunkLoadingStrategy::Random)).collect();
    println!("Generated {} chunks using Random strategy", chunks_random.len());
    
    // Get chunks with camera distance
    let chunks_camera: Vec<_> = sch.iter_chunks(
        5, 5, 5, 
        Some(ChunkLoadingStrategy::DistanceToCamera(5.0, 5.0, 5.0))
    ).collect();
    println!("Generated {} chunks ordered by camera distance", chunks_camera.len());
    
    // Process chunks
    for (i, chunk) in chunks.iter().take(3).enumerate() {
        println!("Chunk {}: ({}, {}, {}) with {} positions", 
                i, chunk.chunk_x, chunk.chunk_y, chunk.chunk_z, chunk.positions.len());
    }
    
    println!();
    
    Ok(())
}

fn copy_region_demo() -> Result<(), Box<dyn std::error::Error>> {
    print_separator("Region Copying Demo");
    
    // Create source schematic with a pattern
    let mut source = UniversalSchematic::new("Source".to_string());
    
    // Create a 3x3x3 pattern
    for x in 0..3 {
        for y in 0..3 {
            for z in 0..3 {
                let block_type = if x == 1 && y == 1 && z == 1 {
                    BlockState::new("minecraft:diamond_block".to_string())
                } else {
                    BlockState::new("minecraft:stone".to_string())
                };
                source.set_block(x, y, z, block_type);
            }
        }
    }
    
    // Create destination schematic
    let mut dest = UniversalSchematic::new("Destination".to_string());
    
    // Copy the pattern multiple times
    for i in 0..3 {
        let bounds = BoundingBox::new((0, 0, 0), (2, 2, 2));
        dest.copy_region(
            &source,
            &bounds,
            (i * 4, 0, 0),  // target position
            &[]             // no excluded blocks
        )?;
    }
    
    let dest_block_count = dest.get_blocks().len();
    println!("Copied pattern 3 times. Destination has {} blocks", dest_block_count);
    
    // Copy again but exclude diamond blocks
    let bounds = BoundingBox::new((0, 0, 0), (2, 2, 2));
    let diamond_block = BlockState::new("minecraft:diamond_block".to_string());
    dest.copy_region(
        &source,
        &bounds,
        (0, 4, 0),
        &[diamond_block]  // exclude diamonds
    )?;
    
    let final_block_count = dest.get_blocks().len();
    println!("Copied without diamonds. Destination now has {} blocks", final_block_count);
    println!();
    
    Ok(())
}

fn error_handling_demo() -> Result<(), Box<dyn std::error::Error>> {
    print_separator("Error Handling Demo");
    
    let mut sch = UniversalSchematic::new("Error Demo".to_string());
    
    // Test basic functionality (these operations normally succeed)
    let result = sch.set_block(-1, 0, 0, BlockState::new("minecraft:stone".to_string()));
    println!("Set block at (-1, 0, 0): {}", result);
    
    let result2 = sch.set_block(0, 0, 0, BlockState::new("invalid:block_name".to_string()));
    println!("Set block with invalid name: {}", result2);
    
    // Test file operations with invalid paths
    let data = litematic::to_litematic(&sch);
    match data {
        Ok(_) => println!("Litematic conversion succeeded"),
        Err(e) => println!("Litematic conversion failed: {}", e),
    }
    
    // Test invalid file writing
    if let Ok(data) = litematic::to_litematic(&sch) {
        match fs::write("/invalid/path/test.litematic", data) {
            Ok(_) => println!("Invalid path was accepted (unexpected)"),
            Err(e) => println!("Invalid path rejected: {}", e),
        }
    }
    
    println!();
    
    Ok(())
}

fn performance_demo() -> Result<(), Box<dyn std::error::Error>> {
    print_separator("Performance Demo");
    
    use std::time::Instant;
    
    let mut sch = UniversalSchematic::new("Performance Test".to_string());
    
    // Time block placement
    let start = Instant::now();
    for x in 0..100 {
        for y in 0..10 {
            for z in 0..100 {
                sch.set_block(x, y, z, BlockState::new("minecraft:stone".to_string()));
            }
        }
    }
    let duration = start.elapsed();
    
    let block_count = sch.get_blocks().len();
    println!("Placed {} blocks in {:?}", block_count, duration);
    println!("Rate: {:.0} blocks/second", 
             block_count as f64 / duration.as_secs_f64());
    
    // Time serialization
    let start = Instant::now();
    let _data = litematic::to_litematic(&sch)?;
    let serialize_duration = start.elapsed();
    
    println!("Serialized to Litematic in {:?}", serialize_duration);
    println!();
    
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Nucleation Rust API Example");
    println!("{}", "=".repeat(40));
    println!();
    
    // Run all examples
    let basic_sch = basic_usage()?;
    block_state_demo()?;
    let file_sch = file_operations()?;
    advanced_features(&file_sch)?;
    chunk_operations()?;
    copy_region_demo()?;
    error_handling_demo()?;
    performance_demo()?;
    
    println!("Example completed successfully!");
    
    // Clean up generated files
    let files_to_remove = [
        "rust_house.litematic",
        "rust_house.schematic"
    ];
    
    for file in &files_to_remove {
        if let Err(e) = fs::remove_file(file) {
            println!("Note: Could not remove {}: {}", file, e);
        }
    }
    
    println!("Cleaned up generated files");
    
    Ok(())
}
