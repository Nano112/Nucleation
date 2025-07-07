# Nucleation API Report

**Library:** nucleation v0.1.50
**Description:** A high-performance Minecraft schematic parser and utility library

## Classes

### BlockState
Represents a Minecraft block with its properties

**Properties:**
- `name`: String - The block's resource name (e.g., 'minecraft:stone')
- `properties`: HashMap<String, String> - The block's state properties

**Methods:**
- `new(name: String)` -> BlockState - Create a new BlockState with the given name
- `with_property(key: String, value: String)` -> BlockState - Create a new BlockState with an additional property

### Schematic
A Minecraft schematic containing blocks and metadata

**Properties:**
- `dimensions`: Vec<i32> - The schematic's dimensions [width, height, length]
- `block_count`: i32 - Total number of non-air blocks
- `volume`: i32 - Total volume of the schematic
- `region_names`: Vec<String> - Names of all regions in the schematic

**Methods:**
- `new(name: Option<String>)` -> Schematic - Create a new empty schematic
- `from_data(data: bytes)` -> Result<(), String> - Load schematic from byte data, auto-detecting format
- `from_litematic(data: bytes)` -> Result<(), String> - Load schematic from Litematic format data
- `to_litematic()` -> Result<bytes, String> - Convert schematic to Litematic format
- `from_schematic(data: bytes)` -> Result<(), String> - Load schematic from classic .schematic format data
- `to_schematic()` -> Result<bytes, String> - Convert schematic to classic .schematic format
- `set_block(x: i32, y: i32, z: i32, block_name: String)` -> () - Set a block at the specified position
- `set_block_with_properties(x: i32, y: i32, z: i32, block_name: String, properties: HashMap<String, String>)` -> () - Set a block at the specified position with properties
- `get_block(x: i32, y: i32, z: i32)` -> Option<BlockState> - Get block at the specified position

## Functions

- `load_schematic(path: String)` -> Result<Schematic, String> - Load a schematic from file path
- `save_schematic(schematic: Schematic, path: String, format: String)` -> Result<(), String> - Save a schematic to file path
- `debug_schematic(schematic: Schematic)` -> String - Get debug information for a schematic
- `debug_json_schematic(schematic: Schematic)` -> String - Get debug information for a schematic in JSON format
