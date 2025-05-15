# Nucleation

Nucleation is a high-performance Minecraft schematic parser and utility library written in Rust, with WebAssembly and FFI bindings for multiple environments.

[//]: # ([![Crates.io]&#40;https://img.shields.io/crates/v/nucleation.svg&#41;]&#40;https://crates.io/crates/nucleation&#41;)

[//]: # ([![NPM Version]&#40;https://img.shields.io/npm/v/nucleation.svg&#41;]&#40;https://www.npmjs.com/package/nucleation&#41;)

[//]: # ([![MIT/Apache-2.0]&#40;https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg&#41;]&#40;LICENSE&#41;)

## Features

- Parse and manipulate multiple schematic formats (.schematic, .litematic, etc.)
- High-performance Rust core with WebAssembly bindings
- Chunk-based loading for progressive rendering
- Redstone circuit analysis and simulation using MCHPRS
- Block entity support (chests, signs, etc.)
- Designed for integration with [Cubane](https://github.com/Nano112/cubane) for 3D visualization

## Installation

### Rust

```bash
cargo add nucleation
```

### JavaScript/TypeScript (via npm)

```bash
npm install nucleation
# or
yarn add nucleation
```

## Usage Examples

### JavaScript (WebAssembly)

```javascript
import { SchematicParser } from 'nucleation';

// Load a schematic file
const response = await fetch('example.litematic');
const fileData = new Uint8Array(await response.arrayBuffer());

// Parse the schematic
const parser = new SchematicParser();
await parser.fromData(fileData);

// Get schematic dimensions
const [width, height, depth] = parser.getDimensions();
console.log(`Dimensions: ${width}x${height}x${depth}`);

// Load blocks progressively in chunks
const chunks = parser.chunksWithStrategy(
  16, 16, 16,           // chunk size
  "distance_to_camera", // loading strategy
  camera.x, camera.y, camera.z
);

// With Cubane integration
const renderer = new Cubane.Cubane();
for (const chunk of chunks) {
  for (const block of chunk.blocks) {
    const mesh = await renderer.getBlockMesh(block.name);
    mesh.position.set(block.x, block.y, block.z);
    scene.add(mesh);
  }
}
```

### Rust

```rust
use nucleation::SchematicParser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load a schematic file
    let data = std::fs::read("example.litematic")?;
    
    // Parse the schematic
    let parser = SchematicParser::new();
    let schematic = parser.from_data(&data)?;
    
    // Get block information
    let dimensions = schematic.get_dimensions();
    println!("Dimensions: {}x{}x{}", dimensions[0], dimensions[1], dimensions[2]);
    
    // Iterate through blocks
    for (pos, block) in schematic.iter_blocks() {
        println!("Block at {},{},{}: {}", pos.x, pos.y, pos.z, block.name);
    }
    
    Ok(())
}
```

## Integration with Cubane

Nucleation works seamlessly with [Cubane](https://github.com/Nano112/cubane) for 3D visualization, forming a complete solution for Minecraft schematic processing and rendering.

```javascript
import { SchematicParser } from 'nucleation';
import { Cubane } from 'cubane';

// Parse schematic with Nucleation
const parser = new SchematicParser();
await parser.fromData(fileData);

// Render with Cubane
const cubane = new Cubane.Cubane();
// ... set up Three.js scene ...

// Load and render blocks
for (const block of parser.blocks()) {
  const mesh = await cubane.getBlockMesh(block.name);
  mesh.position.set(block.x, block.y, block.z);
  scene.add(mesh);
}
```

## License

This project is available under the MIT or Apache-2.0 license.

### Acknowledgements

Nucleation incorporates components from:
- [MCHPRS](https://github.com/MCHPRS/MCHPRS) (MIT License)
- [hematite_nbt](https://github.com/StackDoubleFlow/hematite_nbt) (MIT License)

## Development

```bash
# Build the Rust library
cargo build --release

# Build the WebAssembly package
./build-wasm.sh

# Run tests
cargo test
```

---

Created and maintained by [Nano](https://github.com/Nano112)