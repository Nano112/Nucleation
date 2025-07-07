//! Integration tests for blockpedia features in Nucleation
//! 
//! These tests verify that the blockpedia integration works correctly
//! when the blockpedia feature is enabled.

#[cfg(feature = "blockpedia")]
mod blockpedia_tests {
    use nucleation::{UniversalSchematic, BlockState};
    use nucleation::blockpedia::{ColorAnalysis, BlockpediaError};
    
    #[test]
    fn test_basic_blockpedia_integration() {
        // Create a simple schematic with some blocks
        let mut schematic = UniversalSchematic::new("Test Schematic".to_string());
        
        // Add some blocks that should have color data in blockpedia
        let stone_block = BlockState::new("minecraft:stone".to_string());
        let dirt_block = BlockState::new("minecraft:dirt".to_string());
        let grass_block = BlockState::new("minecraft:grass_block".to_string());
        
        schematic.set_block(0, 0, 0, stone_block);
        schematic.set_block(1, 0, 0, dirt_block);
        schematic.set_block(2, 0, 0, grass_block);
        
        // Test color analysis
        let analysis_result = schematic.analyze_colors();
        assert!(analysis_result.is_ok(), "Color analysis should succeed");
        
        let analysis = analysis_result.unwrap();
        
        // Verify analysis results make sense
        assert!(analysis.coverage_percentage >= 0.0);
        assert!(analysis.coverage_percentage <= 100.0);
        assert!(analysis.harmony_score >= 0.0);
        assert!(analysis.harmony_score <= 1.0);
        
        println!("Color coverage: {:.1}%", analysis.coverage_percentage);
        println!("Harmony score: {:.2}", analysis.harmony_score);
        println!("Total colored blocks: {}", analysis.total_colored_blocks);
        println!("Dominant colors found: {}", analysis.dominant_colors.len());
    }
    
    #[test]
    fn test_material_replacement() {
        let mut schematic = UniversalSchematic::new("Material Test".to_string());
        
        // Add some oak stairs
        let oak_stairs = BlockState::new("minecraft:oak_stairs".to_string())
            .with_property("facing".to_string(), "north".to_string())
            .with_property("half".to_string(), "bottom".to_string());
        
        schematic.set_block(0, 0, 0, oak_stairs);
        
        // Replace oak with stone
        let replacement_result = schematic.replace_material("oak", "stone");
        assert!(replacement_result.is_ok(), "Material replacement should succeed");
        
        let replacements = replacement_result.unwrap();
        println!("Made {} material replacements", replacements);
    }
    
    #[test]
    fn test_palette_extraction() {
        let mut schematic = UniversalSchematic::new("Palette Test".to_string());
        
        // Add blocks with different colors
        let blocks = vec![
            "minecraft:stone",
            "minecraft:dirt", 
            "minecraft:grass_block",
            "minecraft:oak_planks",
            "minecraft:water",
        ];
        
        for (i, block_name) in blocks.iter().enumerate() {
            let block = BlockState::new(block_name.to_string());
            schematic.set_block(i as i32, 0, 0, block);
        }
        
        // Extract a color palette
        let palette_result = schematic.extract_palette(5);
        assert!(palette_result.is_ok(), "Palette extraction should succeed");
        
        let palette = palette_result.unwrap();
        println!("Extracted palette with {} colors", palette.len());
        
        // Print the colors for visual verification
        for (i, color) in palette.iter().enumerate() {
            println!("Color {}: #{:02X}{:02X}{:02X}", 
                i + 1, color.rgb[0], color.rgb[1], color.rgb[2]);
        }
    }
    
    #[test] 
    fn test_error_handling() {
        let schematic = UniversalSchematic::new("Error Test".to_string());
        
        // Test with an empty schematic - should still work but return empty results
        let analysis = schematic.analyze_colors().unwrap();
        assert_eq!(analysis.total_colored_blocks, 0);
        assert_eq!(analysis.coverage_percentage, 0.0);
        
        // Test palette extraction on empty schematic
        let palette = schematic.extract_palette(5).unwrap();
        assert!(palette.is_empty());
    }
    
    #[test]
    fn test_nucleation_blockpedia_block_conversion() {
        // Test that we can convert between Nucleation and Blockpedia block formats
        let nucleation_block = BlockState::new("minecraft:oak_stairs".to_string())
            .with_property("facing".to_string(), "north".to_string())
            .with_property("half".to_string(), "top".to_string())
            .with_property("shape".to_string(), "straight".to_string());
        
        // This conversion happens internally in material replacement
        let mut schematic = UniversalSchematic::new("Conversion Test".to_string());
        schematic.set_block(0, 0, 0, nucleation_block);
        
        // Try to replace material - this exercises the conversion functions
        let result = schematic.replace_material("oak", "stone");
        assert!(result.is_ok(), "Block conversion should work correctly");
    }
}

// Tests that work without the blockpedia feature
#[test] 
fn test_without_blockpedia_feature() {
    use nucleation::{UniversalSchematic, BlockState};
    
    // This test should pass regardless of whether blockpedia is enabled
    let schematic = UniversalSchematic::new("Basic Test".to_string());
    assert_eq!(schematic.regions.len(), 0);
    
    // Test basic block setting still works
    let mut schematic = UniversalSchematic::new("Basic Block Test".to_string());
    let stone = BlockState::new("minecraft:stone".to_string());
    let success = schematic.set_block(0, 0, 0, stone);
    assert!(success, "Setting blocks should work without blockpedia");
}

// NOTE: blockpedia is now a core dependency, so these tests are no longer needed
// The blockpedia functionality is always available.
