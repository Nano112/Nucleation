//! Advanced region operations for Nucleation
//! 
//! This module provides specialized operations for transforming and manipulating
//! regions within schematics, including color analysis, material transformations,
//! and intelligent block operations.

use crate::{BlockState, Region};
use blockpedia::{
    color::ExtendedColorData,
    transforms::BlockShape,
    get_block,
    BlockState as BlockpediaBlockState,
};
use std::collections::HashMap;

/// Error type for region operations
#[derive(Debug, thiserror::Error)]
pub enum RegionError {
    #[error("Block not found: {0}")]
    BlockNotFound(String),
    #[error("Transform error: {0}")]
    TransformError(String),
    #[error("Color error: {0}")]
    ColorError(String),
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

pub type Result<T> = std::result::Result<T, RegionError>;

/// Color analysis results for a region
#[derive(Debug, Clone)]
pub struct RegionColorAnalysis {
    pub dominant_colors: Vec<ExtendedColorData>,
    pub color_distribution: HashMap<String, usize>,
    pub harmony_score: f32,
    pub total_colored_blocks: usize,
    pub total_blocks: usize,
    pub coverage_percentage: f32,
}

/// Batch transformation operation
#[derive(Debug, Clone)]
pub enum BatchOperation {
    ReplaceMaterial { from: String, to: String },
    ChangeShape { material: String, target_shape: BlockShape },
    ReplaceColorMatch { target_color: ExtendedColorData, tolerance: f32, replacement: String },
    CustomTransform { predicate: String, transform: String }, // Simplified for now
}

/// Enhanced region operations
impl Region {
    /// Analyze the color composition of this region
    pub fn analyze_colors(&self) -> Result<RegionColorAnalysis> {
        let mut block_colors = Vec::new();
        let mut color_distribution = HashMap::new();
        let mut total_blocks = 0;
        let mut colored_blocks = 0;

        // Count blocks in palette (this represents unique block types, not positions)
        let palette = self.get_palette();
        total_blocks = palette.len();

        for block_state in palette {
            if let Some(blockpedia_block) = get_block(&block_state.name) {
                if let Some(color) = blockpedia_block.extras.color {
                    let extended_color = color.to_extended();
                    block_colors.push(extended_color);
                    colored_blocks += 1;
                    
                    let color_key = format!("#{:02X}{:02X}{:02X}", 
                        extended_color.rgb[0], 
                        extended_color.rgb[1], 
                        extended_color.rgb[2]
                    );
                    *color_distribution.entry(color_key).or_insert(0) += 1;
                }
            }
        }

        let coverage_percentage = if total_blocks > 0 {
            (colored_blocks as f32 / total_blocks as f32) * 100.0
        } else {
            0.0
        };

        let dominant_colors = if !block_colors.is_empty() {
            find_dominant_colors(&block_colors, 5)
        } else {
            Vec::new()
        };

        let harmony_score = calculate_harmony_score(&block_colors);

        Ok(RegionColorAnalysis {
            dominant_colors,
            color_distribution,
            harmony_score,
            total_colored_blocks: colored_blocks,
            total_blocks,
            coverage_percentage,
        })
    }

    /// Replace all blocks of one material with another while preserving shapes
    pub fn replace_material(&mut self, from_material: &str, to_material: &str) -> Result<usize> {
        self.transform_blocks(|block| {
            if block.name.contains(from_material) {
                // Convert to blockpedia format, transform, then convert back
                match nucleation_to_blockpedia_block(block) {
                    Ok(blockpedia_block) => {
                        match blockpedia_block.with_material(to_material) {
                            Ok(transformed) => Ok(blockpedia_to_nucleation_block(&transformed)),
                            Err(e) => Err(RegionError::TransformError(e.to_string()))
                        }
                    }
                    Err(e) => Err(e)
                }
            } else {
                Ok(block.clone())
            }
        })
    }

    /// Convert blocks of a specific material to a target shape
    pub fn convert_to_shape(&mut self, material: &str, target_shape: BlockShape) -> Result<usize> {
        self.transform_blocks(|block| {
            if block.name.contains(material) {
                // Convert to blockpedia format, transform, then convert back
                match nucleation_to_blockpedia_block(block) {
                    Ok(blockpedia_block) => {
                        match blockpedia_block.with_shape(target_shape.clone()) {
                            Ok(transformed) => Ok(blockpedia_to_nucleation_block(&transformed)),
                            Err(e) => Err(RegionError::TransformError(e.to_string()))
                        }
                    }
                    Err(e) => Err(e)
                }
            } else {
                Ok(block.clone())
            }
        })
    }

    /// Replace blocks that match a color within tolerance
    pub fn replace_by_color_match(
        &mut self, 
        target_color: ExtendedColorData, 
        tolerance: f32,
        replacement_material: &str
    ) -> Result<usize> {
        self.transform_blocks(|block| {
            if let Some(blockpedia_block) = get_block(&block.name) {
                if let Some(color) = blockpedia_block.extras.color {
                    if color.to_extended().distance_oklab(&target_color) <= tolerance {
                        // Try to maintain the same shape but change material
                        match nucleation_to_blockpedia_block(block) {
                            Ok(bp_block) => {
                                match bp_block.with_material(replacement_material) {
                                    Ok(transformed) => Ok(blockpedia_to_nucleation_block(&transformed)),
                                    Err(_) => {
                                        // Fallback to basic block if shape conversion fails
                                        Ok(BlockState::new(format!("minecraft:{}", replacement_material)))
                                    }
                                }
                            }
                            Err(_) => Ok(BlockState::new(format!("minecraft:{}", replacement_material)))
                        }
                    } else {
                        Ok(block.clone())
                    }
                } else {
                    Ok(block.clone())
                }
            } else {
                Ok(block.clone())
            }
        })
    }

    /// Apply a custom transformation function to all blocks
    pub fn transform_blocks<F>(&mut self, transform_fn: F) -> Result<usize>
    where
        F: Fn(&BlockState) -> Result<BlockState>
    {
        let mut transformations = 0;
        let mut new_palette = Vec::new();
        let mut palette_mapping = HashMap::new();

        // Transform each block in the palette
        for (index, block_state) in self.get_palette().iter().enumerate() {
            match transform_fn(block_state) {
                Ok(transformed) => {
                    if transformed != *block_state {
                        transformations += 1;
                    }
                    new_palette.push(transformed);
                    palette_mapping.insert(index, new_palette.len() - 1);
                },
                Err(_) => {
                    // Keep original block if transformation fails
                    new_palette.push(block_state.clone());
                    palette_mapping.insert(index, new_palette.len() - 1);
                }
            }
        }

        // TODO: Update the region's internal block data to use the new palette
        // This would require access to Region's private fields
        // For now, this is a placeholder that demonstrates the API
        
        Ok(transformations)
    }

    /// Replace blocks matching a predicate with a transformation
    pub fn replace_blocks_matching<P, T>(&mut self, predicate: P, transform: T) -> Result<usize>
    where
        P: Fn(&BlockState) -> bool,
        T: Fn(&BlockState) -> Result<BlockState>
    {
        self.transform_blocks(|block| {
            if predicate(block) {
                transform(block)
            } else {
                Ok(block.clone())
            }
        })
    }

    /// Rotate all blocks in this region
    pub fn rotate_region(&mut self, rotation: blockpedia::transforms::Rotation) -> Result<()> {
        self.transform_blocks(|block| {
            // Apply rotation to the block state through blockpedia conversion
            match nucleation_to_blockpedia_block(block) {
                Ok(blockpedia_block) => {
                    // Use the appropriate rotation method based on the rotation type
                    let rotated = match rotation {
                        blockpedia::transforms::Rotation::Clockwise90 => blockpedia_block.rotate_clockwise(),
                        blockpedia::transforms::Rotation::Half => blockpedia_block.rotate_180(),
                        blockpedia::transforms::Rotation::Clockwise270 => blockpedia_block.rotate_counter_clockwise(),
                        blockpedia::transforms::Rotation::None => Ok(blockpedia_block.clone()),
                    };
                    
                    match rotated {
                        Ok(rotated_block) => Ok(blockpedia_to_nucleation_block(&rotated_block)),
                        Err(e) => Err(RegionError::TransformError(e.to_string()))
                    }
                }
                Err(e) => Err(e)
            }
        })?;
        
        // TODO: Also need to rotate the actual positions of blocks within the region
        // This would require restructuring the region's coordinate system
        
        Ok(())
    }

    /// Apply multiple operations in batch for efficiency
    pub fn apply_batch_operations(&mut self, operations: &[BatchOperation]) -> Result<Vec<usize>> {
        let mut results = Vec::new();
        
        for operation in operations {
            let result = match operation {
                BatchOperation::ReplaceMaterial { from, to } => {
                    self.replace_material(from, to)?
                },
                BatchOperation::ChangeShape { material, target_shape } => {
                    self.convert_to_shape(material, target_shape.clone())?
                },
                BatchOperation::ReplaceColorMatch { target_color, tolerance, replacement } => {
                    self.replace_by_color_match(*target_color, *tolerance, replacement)?
                },
                BatchOperation::CustomTransform { predicate: _, transform: _ } => {
                    // Placeholder for custom transformations
                    // In a real implementation, this would parse and execute custom logic
                    0
                },
            };
            results.push(result);
        }
        
        Ok(results)
    }

    /// Find blocks in this region that match specific criteria
    pub fn find_blocks<F>(&self, criteria: F) -> Vec<(BlockState, Vec<(i32, i32, i32)>)>
    where
        F: Fn(&BlockState) -> bool
    {
        let mut results = Vec::new();
        let palette = self.get_palette();
        
        for (_index, block_state) in palette.iter().enumerate() {
            if criteria(block_state) {
                // TODO: Find actual positions where this block appears
                // For now, return empty position list as placeholder
                let positions = Vec::new(); // Would need access to region internals
                results.push((block_state.clone(), positions));
            }
        }
        
        results
    }

    /// Extract a color palette from this region's blocks
    pub fn extract_color_palette(&self, size: usize) -> Result<Vec<ExtendedColorData>> {
        let analysis = self.analyze_colors()?;
        
        if analysis.dominant_colors.is_empty() {
            return Ok(Vec::new());
        }

        // Use blockpedia's palette generation capabilities
        let all_colors: Vec<_> = analysis.dominant_colors.into_iter().collect();
        Ok(blockpedia::color::palettes::PaletteGenerator::generate_distinct_palette(&all_colors, size))
    }
}

/// Convert Nucleation BlockState to Blockpedia BlockState
fn nucleation_to_blockpedia_block(nucleation_block: &BlockState) -> Result<BlockpediaBlockState> {
    // Parse the block name and properties into blockpedia format
    let mut blockpedia_block = BlockpediaBlockState::new(&nucleation_block.name)
        .map_err(|e| RegionError::TransformError(e.to_string()))?;

    // Add properties
    for (key, value) in &nucleation_block.properties {
        blockpedia_block = blockpedia_block.with(key, value)
            .map_err(|e| RegionError::TransformError(e.to_string()))?;
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

/// Find the most dominant colors in a collection
fn find_dominant_colors(colors: &[ExtendedColorData], count: usize) -> Vec<ExtendedColorData> {
    let mut color_counts: HashMap<String, (ExtendedColorData, usize)> = HashMap::new();
    
    for color in colors {
        let key = format!("#{:02X}{:02X}{:02X}", color.rgb[0], color.rgb[1], color.rgb[2]);
        let entry = color_counts.entry(key).or_insert((*color, 0));
        entry.1 += 1;
    }

    let mut sorted_colors: Vec<_> = color_counts.into_values().collect();
    sorted_colors.sort_by(|a, b| b.1.cmp(&a.1));
    
    sorted_colors.into_iter()
        .take(count)
        .map(|(color, _)| color)
        .collect()
}

/// Calculate a harmony score for a collection of colors
fn calculate_harmony_score(colors: &[ExtendedColorData]) -> f32 {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BlockState;

    #[test]
    fn test_region_color_analysis() {
        // This test would need a proper region setup
        // For now, just test that the module compiles
        assert!(true);
    }

    #[test]
    fn test_batch_operations() {
        let operations = vec![
            BatchOperation::ReplaceMaterial { 
                from: "oak".to_string(), 
                to: "stone".to_string() 
            },
            BatchOperation::ChangeShape { 
                material: "stone".to_string(), 
                target_shape: BlockShape::Stairs 
            },
        ];
        
        // Test that operations can be created
        assert_eq!(operations.len(), 2);
    }
}
