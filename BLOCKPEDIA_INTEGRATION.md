# Blockpedia Integration

Nucleation now features powerful integration with [blockpedia](https://github.com/Nano112/blockpedia), bringing advanced color analysis and intelligent block transformations to schematic manipulation.

## Features

### 🎨 Color Analysis
- **Color Coverage Analysis**: Determine what percentage of your schematic has color data
- **Dominant Color Extraction**: Find the most prominent colors in your builds
- **Color Harmony Scoring**: Evaluate how well colors work together
- **Palette Generation**: Extract color palettes from existing schematics

### 🔄 Block Transformations
- **Material Replacement**: Convert between materials while preserving shapes (oak_stairs → stone_stairs)
- **Shape Conversion**: Transform block shapes while maintaining materials (stone → stone_stairs)
- **Color-Based Matching**: Find and replace blocks based on color similarity
- **Property Preservation**: Maintain directional and state properties during transformations

### 🎯 Intelligent Selection
- **Variant Discovery**: Find all available materials and shapes for any block type
- **Smart Recommendations**: Get suggestions for alternative blocks
- **Theme-Based Operations**: Apply consistent material themes across builds

## Installation

Add blockpedia integration to your `Cargo.toml`:

```toml
[dependencies]
nucleation = { version = "0.1.48", features = ["blockpedia"] }
```

## Quick Start

```rust
use nucleation::{UniversalSchematic, BlockState, ExtendedColorData, BlockShape};

// Load or create a schematic
let mut schematic = UniversalSchematic::new("My Build".to_string());

// Add some blocks
schematic.set_block(0, 0, 0, BlockState::new("minecraft:oak_stairs".to_string()));
schematic.set_block(1, 0, 0, BlockState::new("minecraft:stone".to_string()));

// Analyze colors
let analysis = schematic.analyze_colors()?;
println!("Color coverage: {:.1}%", analysis.coverage_percentage);
println!("Harmony score: {:.2}", analysis.harmony_score);

// Replace materials while preserving shapes
let replacements = schematic.replace_material("oak", "stone")?;
println!("Replaced {} blocks", replacements);

// Convert shapes while preserving materials
let conversions = schematic.convert_to_shape("stone", BlockShape::Stairs)?;
println!("Converted {} blocks to stairs", conversions);

// Extract a color palette
let palette = schematic.extract_palette(8)?;
for (i, color) in palette.iter().enumerate() {
    println!("Color {}: #{:02X}{:02X}{:02X}", 
        i + 1, color.rgb[0], color.rgb[1], color.rgb[2]);
}
```

## Detailed Usage

### Color Analysis

```rust
// Analyze the color composition of a schematic
let analysis = schematic.analyze_colors()?;

println!("📊 Schematic Analysis:");
println!("   Color Coverage: {:.1}%", analysis.coverage_percentage);
println!("   Total Colored Blocks: {}", analysis.total_colored_blocks);
println!("   Harmony Score: {:.2}/1.0", analysis.harmony_score);
println!("   Dominant Colors: {}", analysis.dominant_colors.len());

// Show the dominant colors
for (i, color) in analysis.dominant_colors.iter().enumerate() {
    println!("   {}. #{:02X}{:02X}{:02X}", 
        i + 1, color.rgb[0], color.rgb[1], color.rgb[2]);
}
```

### Material Transformations

```rust
// Replace all oak blocks with stone equivalents
let oak_to_stone = schematic.replace_material("oak", "stone")?;
println!("Converted {} oak blocks to stone", oak_to_stone);

// Convert all cobblestone to cobblestone stairs
let stone_to_stairs = schematic.convert_to_shape("cobblestone", BlockShape::Stairs)?;
println!("Converted {} blocks to stairs", stone_to_stairs);

// Apply custom transformations
let custom_replacements = schematic.replace_blocks_matching(
    |block| block.name.contains("planks"),
    |block| {
        // Convert all planks to their brick equivalents
        let material = block.name.replace("_planks", "");
        Ok(BlockState::new(format!("{}_bricks", material)))
    }
)?;
```

### Color-Based Operations

```rust
// Find blocks similar to a target color
let target_color = ExtendedColorData::from_rgb(139, 69, 19); // Brown
let matches = schematic.replace_with_color_match(target_color, 15.0)?;
println!("Found {} blocks with similar colors", matches);

// Generate themed palettes
let medieval_palette = schematic.extract_palette(6)?;
let modern_palette = generate_complementary_palette(&medieval_palette[0]);
```

### Advanced Transformations

```rust
// Custom region-specific transformations
schematic.transform_blocks_in_region("castle_walls", |block| {
    if block.name.contains("stone") {
        // Upgrade stone to stone bricks
        Ok(BlockState::new(block.name.replace("stone", "stone_brick")))
    } else {
        Ok(block.clone())
    }
})?;

// Batch operations
let transformations = vec![
    RegionTransformation::ReplaceMaterial {
        region: foundation_region,
        from: "dirt".to_string(),
        to: "stone".to_string(),
    },
    RegionTransformation::ChangeShape {
        region: detail_region,
        target_shape: BlockShape::Stairs,
    },
];

schematic.apply_transformations(&transformations)?;
```

## Real-World Examples

### Theme Conversion

```rust
// Convert a wooden house to a stone castle
async fn wooden_to_stone_castle(mut house: UniversalSchematic) -> Result<UniversalSchematic> {
    // Replace structural elements
    house.replace_material("oak", "stone_brick")?;
    house.replace_material("birch", "stone")?;
    
    // Convert flat roofs to peaked roofs with stairs
    house.convert_to_shape("stone_brick", BlockShape::Stairs)?;
    
    // Add architectural details
    house.replace_blocks_matching(
        |block| block.name.contains("glass"),
        |_| Ok(BlockState::new("minecraft:iron_bars".to_string()))
    )?;
    
    Ok(house)
}
```

### Color Palette Building

```rust
// Build using a specific color palette
async fn build_with_palette(palette: &[ExtendedColorData]) -> Result<UniversalSchematic> {
    let mut build = UniversalSchematic::new("Palette Build".to_string());
    
    // Find blocks that match each color in the palette
    for (layer, target_color) in palette.iter().enumerate() {
        let matching_blocks = find_blocks_by_color(*target_color, 10.0)?;
        
        // Use the best match for this layer
        if let Some(block) = matching_blocks.first() {
            for x in 0..16 {
                for z in 0..16 {
                    build.set_block(x, layer as i32, z, 
                        BlockState::new(block.id().to_string()));
                }
            }
        }
    }
    
    Ok(build)
}
```

### Progressive Material Upgrades

```rust
// Upgrade a build progressively by height
async fn progressive_upgrade(mut castle: UniversalSchematic) -> Result<UniversalSchematic> {
    let layers = [
        (0..3, "cobblestone", "stone"),           // Foundation
        (3..8, "stone", "stone_brick"),           // Main walls  
        (8..12, "stone_brick", "polished_stone"), // Upper levels
        (12..16, "wood", "dark_oak"),             // Roof structure
    ];
    
    for (y_range, from_material, to_material) in layers {
        for y in y_range {
            let layer_region = Region::horizontal_slice(&castle_bounds, y);
            castle.replace_material_in_region(&layer_region, from_material, to_material)?;
        }
    }
    
    Ok(castle)
}
```

## Integration with Nucleation Features

The blockpedia integration works seamlessly with all existing Nucleation features:

### Language Bindings

```python
# Python
import nucleation

schematic = nucleation.UniversalSchematic("test")
# Color analysis and transformations work in Python too!
analysis = schematic.analyze_colors()
print(f"Coverage: {analysis.coverage_percentage}%")
```

```javascript
// WASM/JavaScript
import { UniversalSchematic } from './nucleation_wasm';

const schematic = new UniversalSchematic("test");
const analysis = schematic.analyze_colors();
console.log(`Coverage: ${analysis.coverage_percentage}%`);
```

### File Format Support

```rust
// Works with all supported formats
let litematic_data = std::fs::read("castle.litematic")?;
let mut schematic = UniversalSchematic::from_litematic(&litematic_data)?;

// Apply blockpedia transformations
schematic.replace_material("oak", "dark_oak")?;
let analysis = schematic.analyze_colors()?;

// Save back to any format
let output_data = schematic.to_litematic()?;
std::fs::write("dark_oak_castle.litematic", output_data)?;
```

## Error Handling

```rust
use nucleation::blockpedia::{BlockpediaError, Result};

match schematic.replace_material("invalid", "stone") {
    Ok(count) => println!("Replaced {} blocks", count),
    Err(BlockpediaError::BlockNotFound(name)) => {
        println!("Block not found: {}", name);
    },
    Err(BlockpediaError::TransformError(msg)) => {
        println!("Transform failed: {}", msg);
    },
    Err(BlockpediaError::FeatureNotEnabled) => {
        println!("Build with --features blockpedia to enable this feature");
    },
    Err(e) => println!("Error: {}", e),
}
```

## Performance Notes

- Color analysis is cached and reused when possible
- Transformations operate on block palettes for efficiency
- Large schematics are processed in chunks to manage memory
- All operations are designed to be non-destructive by default

## Feature Flags

- `blockpedia` - Core integration (required)
- `color-analysis` - Color analysis features (included with blockpedia)
- `transforms` - Block transformation features (included with blockpedia)

## Examples

Run the included examples to see the integration in action:

```bash
# Basic showcase
cargo run --example blockpedia_showcase --features blockpedia

# Run integration tests
cargo test --features blockpedia --test blockpedia_integration_test
```

## Requirements

- Rust 1.82+
- Internet connection for initial blockpedia data download
- Optional: `blockpedia` CLI for advanced operations

## See Also

- [Blockpedia Documentation](https://github.com/Nano112/blockpedia)
- [Nucleation Core Documentation](README.md)
- [Color Theory Guide](docs/color-theory.md)
- [Transformation Patterns](docs/transformation-patterns.md)
