use nucleation::{UniversalSchematic, BlockState};
use nucleation::universal_schematic::ChunkLoadingStrategy;

fn main() {
    // Create a test schematic with some blocks
    let mut schematic = UniversalSchematic::new("Test Schematic".to_string());
    
    // Add some non-air blocks
    for x in 0..10 {
        for y in 0..10 {
            for z in 0..10 {
                if (x + y + z) % 3 == 0 {  // Only set some blocks, not all
                    schematic.set_block(x, y, z, BlockState::new("minecraft:stone".to_string()));
                }
            }
        }
    }
    
    println!("Created test schematic with blocks");
    
    // Test chunk methods with same parameters
    let chunk_width = 16;
    let chunk_height = 16;
    let chunk_length = 16;
    
    // Count chunks using iter_chunks
    let chunks: Vec<_> = schematic.iter_chunks(
        chunk_width, 
        chunk_height, 
        chunk_length, 
        Some(ChunkLoadingStrategy::BottomUp)
    ).collect();
    
    // Count chunks using iter_chunks_indices  
    let chunks_indices: Vec<_> = schematic.iter_chunks_indices(
        chunk_width, 
        chunk_height, 
        chunk_length, 
        Some(ChunkLoadingStrategy::BottomUp)
    ).collect();
    
    println!("Regular chunks method returned: {} chunks", chunks.len());
    println!("Indices chunks method returned: {} chunks", chunks_indices.len());
    
    // They should now be equal!
    if chunks.len() == chunks_indices.len() {
        println!("✅ SUCCESS: Chunk counts are now consistent!");
    } else {
        println!("❌ FAILURE: Chunk counts are still inconsistent");
    }
    
    // Test total block counts
    let total_blocks: Vec<_> = schematic.iter_blocks().collect();
    let non_air_blocks: Vec<_> = schematic.iter_blocks_indices().collect();
    
    println!("Total blocks (including air): {}", total_blocks.len());
    println!("Non-air blocks: {}", non_air_blocks.len());
    
    // Count air blocks
    let air_count = total_blocks.iter().filter(|(_, block)| block.name == "minecraft:air").count();
    println!("Air blocks: {}", air_count);
    println!("Non-air blocks calculated: {}", total_blocks.len() - air_count);
    
    if non_air_blocks.len() == total_blocks.len() - air_count {
        println!("✅ SUCCESS: Block counts are consistent!");
    } else {
        println!("❌ FAILURE: Block counts are inconsistent");
    }
}
