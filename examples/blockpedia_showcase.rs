//! Blockpedia Integration Showcase
//! 
//! This example demonstrates the key features of blockpedia integration in Nucleation:
//! - Color analysis of existing schematics
//! - Material replacement while preserving shapes
//! - Palette extraction and color-based block recommendations
//! - Intelligent block transformations

use nucleation::{UniversalSchematic, BlockState};

#[cfg(feature = "blockpedia")]
use nucleation::{ExtendedColorData, BlockShape};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Blockpedia Integration Showcase");
    println!("==================================\n");

    // Create a sample building schematic
    let mut castle = create_sample_castle();
    
    #[cfg(feature = "blockpedia")]
    {
        println!("📊 Original Castle Analysis:");
        analyze_schematic(&castle)?;
        
        println!("\n🔄 Converting Oak to Stone...");
        let oak_replacements = castle.replace_material("oak", "stone")?;
        println!("✅ Replaced {} oak blocks with stone", oak_replacements);
        
        println!("\n📊 Updated Castle Analysis:");
        analyze_schematic(&castle)?;
        
        println!("\n🎨 Extracting Color Palette...");
        let palette = castle.extract_palette(8)?;
        print_color_palette(&palette);
        
        println!("\n🔍 Finding Color-Similar Blocks...");
        demonstrate_color_matching(&mut castle)?;
        
        println!("\n🏗️  Material & Shape Transformations...");
        demonstrate_transformations(&mut castle)?;
    }
    
    #[cfg(not(feature = "blockpedia"))]
    {
        println!("ℹ️  Blockpedia features are not enabled.");
        println!("   To see the full showcase, build with: cargo run --example blockpedia_showcase --features blockpedia");
        println!("\n📦 Basic schematic operations still work:");
        println!("   Castle size: {} regions", castle.regions.len());
        if let Some(region) = castle.regions.values().next() {
            println!("   Palette size: {} unique blocks", region.get_palette().len());
        }
    }
    
    Ok(())
}

/// Create a sample castle schematic with various materials
fn create_sample_castle() -> UniversalSchematic {
    let mut castle = UniversalSchematic::new("Sample Castle".to_string());
    
    // Foundation - cobblestone
    for x in 0..10 {
        for z in 0..10 {
            let cobblestone = BlockState::new("minecraft:cobblestone".to_string());
            castle.set_block(x, 0, z, cobblestone);
        }
    }
    
    // Walls - stone bricks with oak accents
    for y in 1..5 {
        for x in 0..10 {
            for z in 0..10 {
                if x == 0 || x == 9 || z == 0 || z == 9 {
                    let wall_block = if (x + z + y) % 4 == 0 {
                        BlockState::new("minecraft:oak_planks".to_string())
                    } else {
                        BlockState::new("minecraft:stone_bricks".to_string())
                    };
                    castle.set_block(x, y, z, wall_block);
                }
            }
        }
    }
    
    // Roof - oak stairs and slabs
    for x in 1..9 {
        for z in 1..9 {
            let roof_block = if (x + z) % 2 == 0 {
                BlockState::new("minecraft:oak_stairs".to_string())
                    .with_property("facing".to_string(), "north".to_string())
                    .with_property("half".to_string(), "bottom".to_string())
            } else {
                BlockState::new("minecraft:oak_slab".to_string())
                    .with_property("type".to_string(), "bottom".to_string())
            };
            castle.set_block(x, 5, z, roof_block);
        }
    }
    
    // Add some decorative elements
    castle.set_block(5, 6, 5, BlockState::new("minecraft:glowstone".to_string()));
    castle.set_block(2, 3, 2, BlockState::new("minecraft:glass".to_string()));
    castle.set_block(7, 3, 7, BlockState::new("minecraft:glass".to_string()));
    
    println!("🏰 Created sample castle with {} regions", castle.regions.len());
    castle
}

#[cfg(feature = "blockpedia")]
fn analyze_schematic(schematic: &UniversalSchematic) -> Result<(), Box<dyn std::error::Error>> {
    let analysis = schematic.analyze_colors()?;
    
    println!("   🎨 Color Coverage: {:.1}%", analysis.coverage_percentage);
    println!("   🎵 Color Harmony Score: {:.2}/1.0", analysis.harmony_score);
    println!("   🧱 Total Colored Blocks: {}", analysis.total_colored_blocks);
    println!("   🌈 Dominant Colors: {}", analysis.dominant_colors.len());
    
    // Show dominant colors
    for (i, color) in analysis.dominant_colors.iter().take(3).enumerate() {
        println!("      {}. #{:02X}{:02X}{:02X}", 
            i + 1, color.rgb[0], color.rgb[1], color.rgb[2]);
    }
    
    Ok(())
}

#[cfg(feature = "blockpedia")]
fn print_color_palette(palette: &[ExtendedColorData]) {
    println!("   Extracted {} colors from the schematic:", palette.len());
    for (i, color) in palette.iter().enumerate() {
        println!("   {}. #{:02X}{:02X}{:02X} (RGB: {}, {}, {})", 
            i + 1, 
            color.rgb[0], color.rgb[1], color.rgb[2],
            color.rgb[0], color.rgb[1], color.rgb[2]
        );
    }
}

#[cfg(feature = "blockpedia")]
fn demonstrate_color_matching(castle: &mut UniversalSchematic) -> Result<(), Box<dyn std::error::Error>> {
    // Create a target color (stone gray)
    let stone_gray = ExtendedColorData::from_rgb(125, 125, 125);
    println!("   🎯 Target color: #{:02X}{:02X}{:02X} (stone gray)", 
        stone_gray.rgb[0], stone_gray.rgb[1], stone_gray.rgb[2]);
    
    // Find blocks with similar colors (with a tolerance)
    let matches = castle.replace_with_color_match(stone_gray, 25.0)?;
    println!("   ✅ Found and replaced {} blocks with similar colors", matches);
    
    Ok(())
}

#[cfg(feature = "blockpedia")]
fn demonstrate_transformations(castle: &mut UniversalSchematic) -> Result<(), Box<dyn std::error::Error>> {
    // Convert some stone blocks to stairs for architectural detail
    let shape_conversions = castle.convert_to_shape("cobblestone", BlockShape::Stairs)?;
    println!("   🔧 Converted {} cobblestone blocks to stairs", shape_conversions);
    
    // Show the variety of transformations possible
    println!("   💡 Available transformation types:");
    println!("      • Material replacement (oak → stone, brick → quartz, etc.)");
    println!("      • Shape conversion (full blocks → stairs, slabs, walls, etc.)");
    println!("      • Color-based matching (find blocks by target color)");
    println!("      • Rotation and orientation (coming soon)");
    
    Ok(())
}

#[cfg(feature = "blockpedia")]
fn demonstrate_advanced_features() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🚀 Advanced Features Preview:");
    
    // Show what's possible with the integration
    println!("   🎨 Color Analysis:");
    println!("      • Extract dominant colors from existing builds");
    println!("      • Calculate color harmony scores");
    println!("      • Generate complementary color palettes");
    
    println!("   🔄 Smart Transformations:");
    println!("      • Material swapping with property preservation");
    println!("      • Shape conversions maintaining orientation");
    println!("      • Bulk operations across entire schematics");
    
    println!("   🎯 Intelligent Selection:");
    println!("      • Find blocks by color similarity");
    println!("      • Discover available material variants");
    println!("      • Suggest alternative blocks for themes");
    
    Ok(())
}

// Example of what integration with real schematics might look like
#[cfg(feature = "blockpedia")]
fn example_real_world_usage() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🌍 Real-World Usage Examples:");
    
    println!("   1. 🏠 Converting house materials:");
    println!("      schematic.replace_material(\"oak\", \"dark_oak\")?;");
    
    println!("   2. 🎨 Theme-based building:");
    println!("      let medieval_colors = schematic.extract_palette(5)?;");
    println!("      // Use these colors for consistent theming");
    
    println!("   3. 🔍 Finding matching blocks:");
    println!("      let target = ExtendedColorData::from_rgb(139, 69, 19); // Brown");
    println!("      schematic.replace_with_color_match(target, 15.0)?;");
    
    println!("   4. 🏗️  Architectural details:");
    println!("      schematic.convert_to_shape(\"stone\", BlockShape::Stairs)?;");
    
    Ok(())
}
