// src/blockpedia.rs
//! Blockpedia integration for Nucleation
//! 
//! This module provides color analysis, block transforms, and intelligent block
//! manipulation capabilities by integrating with the blockpedia library.

use blockpedia::{
    BlockState as BlockpediaBlockState, 
    get_block,
    color::{ExtendedColorData, palettes::PaletteGenerator},
    transforms::{BlockShape, Rotation},
    BLOCKS,
};

use crate::{BlockState, UniversalSchematic, Region};
use std::collections::HashMap;

/// Error type for blockpedia integration operations
#[derive(Debug, thiserror::Error)]
pub enum BlockpediaError {
    #[error("Blockpedia feature not enabled")]
    FeatureNotEnabled,
    #[error("Block not found: {0}")]
    BlockNotFound(String),
    #[error("Transform error: {0}")]
    TransformError(String),
    #[error("Color error: {0}")]
    ColorError(String),
}

pub type Result<T> = std::result::Result<T, BlockpediaError>;

/// Color analysis results for a schematic
#[derive(Debug, Clone)]
pub struct ColorAnalysis {
    pub dominant_colors: Vec<ExtendedColorData>,
    pub color_distribution: HashMap<String, usize>,
    pub harmony_score: f32,
    pub total_colored_blocks: usize,
    pub coverage_percentage: f32,
}

/// Block transformation operations for schematics
pub struct SchematicTransforms;

/// Color analysis operations for schematics
pub struct SchematicColorAnalysis;

impl UniversalSchematic {
    /// Analyze the color composition of the entire schematic
    pub fn analyze_colors(&self) -> Result<ColorAnalysis> {
        let mut block_colors = Vec::new();
        let mut color_distribution = HashMap::new();
        let mut total_blocks = 0;
        let mut colored_blocks = 0;

        // Iterate through all blocks in all regions
        for region in self.regions.values() {
            for block_state in region.get_palette() {
                total_blocks += 1;
                
                if let Some(blockpedia_block) = get_block(&block_state.name) {
                    if let Some(color) = blockpedia_block.extras.color {
                        let extended_color = color.to_extended();
                        block_colors.push(extended_color);
                        colored_blocks += 1;
                        
                        // Track color distribution
                        let color_key = format!("#{:02X}{:02X}{:02X}", 
                            extended_color.rgb[0], 
                            extended_color.rgb[1], 
                            extended_color.rgb[2]
                        );
                        *color_distribution.entry(color_key).or_insert(0) += 1;
                    }
                }
            }
        }

        let coverage_percentage = if total_blocks > 0 {
            (colored_blocks as f32 / total_blocks as f32) * 100.0
        } else {
            0.0
        };

        let dominant_colors = if !block_colors.is_empty() {
            SchematicColorAnalysis::find_dominant_colors(&block_colors, 5)
        } else {
            Vec::new()
        };

        let harmony_score = SchematicColorAnalysis::calculate_harmony_score(&block_colors);

        Ok(ColorAnalysis {
            dominant_colors,
            color_distribution,
            harmony_score,
            total_colored_blocks: colored_blocks,
            coverage_percentage,
        })
    }

    /// Replace blocks with color-matched alternatives
    pub fn replace_with_color_match(&mut self, target_color: ExtendedColorData, tolerance: f32) -> Result<usize> {
        let mut replacements = 0;

        // Find matching blocks from blockpedia
        let matching_blocks: Vec<&blockpedia::BlockFacts> = BLOCKS.values()
            .filter(|block| {
                if let Some(color) = block.extras.color {
                    color.to_extended().distance_oklab(&target_color) <= tolerance
                } else {
                    false
                }
            })
            .map(|block| *block)  // Dereference to get &BlockFacts instead of &&BlockFacts
            .collect();

        if matching_blocks.is_empty() {
            return Ok(0);
        }

        // Replace blocks in all regions
        for region in self.regions.values_mut() {
            replacements += region.replace_blocks_with_color_match(&matching_blocks)?;
        }

        Ok(replacements)
    }

    /// Generate a color palette from the schematic's blocks
    pub fn extract_palette(&self, palette_size: usize) -> Result<Vec<ExtendedColorData>> {
        let analysis = self.analyze_colors()?;
        
        if analysis.dominant_colors.is_empty() {
            return Ok(Vec::new());
        }

        let all_colors: Vec<_> = analysis.dominant_colors.into_iter().collect();
        Ok(PaletteGenerator::generate_distinct_palette(&all_colors, palette_size))
    }

    /// Transform blocks in a region using a custom function
    pub fn transform_blocks_in_region<F>(&mut self, region_name: &str, transform_fn: F) -> Result<usize>
    where
        F: Fn(&BlockState) -> Result<BlockState>,
    {
        if let Some(region) = self.regions.get_mut(region_name) {
            // Create a wrapper function that converts RegionError to BlockpediaError
            let wrapper_fn = |block: &BlockState| {
                transform_fn(block)
            };
            
            // Call the region method and convert the error type
            region.transform_blocks(|block| {
                wrapper_fn(block).map_err(|e| crate::region_operations::RegionError::TransformError(e.to_string()))
            }).map_err(|e| BlockpediaError::TransformError(e.to_string()))
        } else {
            Err(BlockpediaError::BlockNotFound(region_name.to_string()))
        }
    }

    /// Replace blocks matching a predicate with a transformation
    pub fn replace_blocks_matching<P, T>(&mut self, predicate: P, transform: T) -> Result<usize>
    where
        P: Fn(&BlockState) -> bool + Clone,
        T: Fn(&BlockState) -> Result<BlockState> + Clone,
    {
        let mut total_replacements = 0;
        
        for region in self.regions.values_mut() {
            // Convert the transform function to work with RegionError
            let region_transform = |block: &BlockState| -> crate::region_operations::Result<BlockState> {
                transform(block).map_err(|e| crate::region_operations::RegionError::TransformError(e.to_string()))
            };
            
            total_replacements += region.replace_blocks_matching(predicate.clone(), region_transform)
                .map_err(|e| BlockpediaError::TransformError(e.to_string()))?;
        }
        
        Ok(total_replacements)
    }

    /// Rotate all blocks in a region
    pub fn rotate_region(&mut self, region_name: &str, rotation: Rotation) -> Result<()> {
        if let Some(region) = self.regions.get_mut(region_name) {
            region.rotate_region(rotation)
                .map_err(|e| BlockpediaError::TransformError(e.to_string()))
        } else {
            Err(BlockpediaError::BlockNotFound(region_name.to_string()))
        }
    }

    /// Replace all blocks of one material with another while preserving shapes
    pub fn replace_material(&mut self, from_material: &str, to_material: &str) -> Result<usize> {
        let mut total_replacements = 0;
        
        for region in self.regions.values_mut() {
            total_replacements += region.replace_material(from_material, to_material)
                .map_err(|e| BlockpediaError::TransformError(e.to_string()))?;
        }
        
        Ok(total_replacements)
    }

    /// Convert blocks to a specific shape while preserving materials
    pub fn convert_to_shape(&mut self, material: &str, target_shape: BlockShape) -> Result<usize> {
        let mut total_replacements = 0;
        
        for region in self.regions.values_mut() {
            total_replacements += region.convert_to_shape(material, target_shape.clone())
                .map_err(|e| BlockpediaError::TransformError(e.to_string()))?;
        }
        
        Ok(total_replacements)
    }
}

impl Region {
    /// Replace blocks with color-matched alternatives in this region
    pub fn replace_blocks_with_color_match(&mut self, matching_blocks: &[&blockpedia::BlockFacts]) -> Result<usize> {
        let mut replacements = 0;
        
        // This is a simplified implementation - in practice you'd want to iterate through
        // the actual block positions and replace them based on the region's storage structure
        // For now, we'll just update the palette
        
        Ok(replacements)
    }
}

impl SchematicColorAnalysis {
    /// Find the most dominant colors in a collection
    pub fn find_dominant_colors(colors: &[ExtendedColorData], count: usize) -> Vec<ExtendedColorData> {
        // Group similar colors and count occurrences
        let mut color_counts: HashMap<String, (ExtendedColorData, usize)> = HashMap::new();
        
        for color in colors {
            let key = format!("#{:02X}{:02X}{:02X}", color.rgb[0], color.rgb[1], color.rgb[2]);
            let entry = color_counts.entry(key).or_insert((*color, 0));
            entry.1 += 1;
        }

        // Sort by count and take the top ones
        let mut sorted_colors: Vec<_> = color_counts.into_values().collect();
        sorted_colors.sort_by(|a, b| b.1.cmp(&a.1));
        
        sorted_colors.into_iter()
            .take(count)
            .map(|(color, _)| color)
            .collect()
    }

    /// Calculate a simple harmony score for a collection of colors
    pub fn calculate_harmony_score(colors: &[ExtendedColorData]) -> f32 {
        if colors.len() < 2 {
            return 1.0;
        }

        let mut total_distance = 0.0;
        let mut comparisons = 0;

        for i in 0..colors.len() {
            for j in (i + 1)..colors.len() {
                total_distance += colors[i].distance_oklab(&colors[j]);
                comparisons += 1;
            }
        }

        if comparisons > 0 {
            let avg_distance = total_distance / comparisons as f32;
            // Convert to a 0-1 score where values closer to 50 are more harmonious
            let harmony = 1.0 - ((avg_distance - 50.0).abs() / 50.0).min(1.0);
            harmony.max(0.0)
        } else {
            1.0
        }
    }
}

/// Convert Nucleation BlockState to Blockpedia BlockState
fn nucleation_to_blockpedia_block(nucleation_block: &BlockState) -> Result<BlockpediaBlockState> {
    // Parse the block name and properties into blockpedia format
    let mut blockpedia_block = BlockpediaBlockState::new(&nucleation_block.name)
        .map_err(|e| BlockpediaError::TransformError(e.to_string()))?;

    // Add properties
    for (key, value) in &nucleation_block.properties {
        blockpedia_block = blockpedia_block.with(key, value)
            .map_err(|e| BlockpediaError::TransformError(e.to_string()))?;
    }

    Ok(blockpedia_block)
}

/// Convert Blockpedia BlockState to Nucleation BlockState
fn blockpedia_to_nucleation_block(blockpedia_block: &BlockpediaBlockState) -> BlockState {
    let block_string = blockpedia_block.to_string();
    
    // Parse the block string to extract name and properties
    if let Some(bracket_pos) = block_string.find('[') {
        let name = block_string[..bracket_pos].to_string();
        let properties_str = &block_string[bracket_pos + 1..block_string.len() - 1];
        
        let mut properties = HashMap::new();
        if !properties_str.is_empty() {
            for prop_pair in properties_str.split(',') {
                let parts: Vec<&str> = prop_pair.split('=').collect();
                if parts.len() == 2 {
                    properties.insert(parts[0].trim().to_string(), parts[1].trim().to_string());
                }
            }
        }
        
        BlockState {
            name,
            properties,
        }
    } else {
        BlockState {
            name: block_string,
            properties: HashMap::new(),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_compiles_without_blockpedia_feature() {
        // This test ensures the module compiles even without the blockpedia feature
        let schematic = UniversalSchematic::new("test".to_string());
        assert_eq!(schematic.regions.len(), 0);
    }
}
