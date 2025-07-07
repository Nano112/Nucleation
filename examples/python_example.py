#!/usr/bin/env python3
"""
Nucleation Python Binding Example

This example demonstrates how to use the nucleation library from Python
to create, modify, and save Minecraft schematics.

Prerequisites:
    pip install nucleation
    
or build from source:
    cd nucleation
    maturin develop --features python
"""

import nucleation as nuc


def basic_usage():
    """Basic schematic creation and manipulation"""
    print("=== Basic Usage ===")
    
    # Create a new empty schematic
    sch = nuc.Schematic("Python Demo")
    
    # Add some basic blocks
    sch.set_block(0, 0, 0, "minecraft:stone")
    sch.set_block(1, 0, 0, "minecraft:oak_log")
    sch.set_block(2, 0, 0, "minecraft:diamond_block")
    
    # Add a block with properties
    sch.set_block_with_properties(
        3, 0, 0, 
        "minecraft:oak_stairs", 
        {"facing": "north", "half": "bottom"}
    )
    
    # Add a block with NBT data (barrel with redstone signal)
    sch.set_block_from_string(
        4, 0, 0,
        'minecraft:barrel[facing=up]{signal=13}'
    )
    
    # Print schematic info
    print(f"Dimensions: {sch.dimensions}")
    print(f"Block count: {sch.block_count}")
    print(f"Volume: {sch.volume}")
    print()
    
    return sch


def block_state_demo():
    """Demonstrate BlockState usage"""
    print("=== BlockState Demo ===")
    
    # Create basic block state
    stone = nuc.BlockState("minecraft:stone")
    print(f"Stone: {stone.name}")
    print(f"Properties: {stone.properties}")
    
    # Create block state with properties
    stairs = nuc.BlockState("minecraft:oak_stairs")
    stairs_north = stairs.with_property("facing", "north")
    stairs_complete = stairs_north.with_property("half", "top")
    
    print(f"Stairs: {stairs_complete.name}")
    print(f"Properties: {stairs_complete.properties}")
    print()


def file_operations():
    """Demonstrate file I/O operations"""
    print("=== File Operations ===")
    
    # Create a sample schematic
    sch = nuc.Schematic("File Demo")
    
    # Build a small house
    # Floor
    for x in range(5):
        for z in range(5):
            sch.set_block(x, 0, z, "minecraft:oak_planks")
    
    # Walls
    for i in range(5):
        # Front and back walls
        sch.set_block(i, 1, 0, "minecraft:oak_log")
        sch.set_block(i, 1, 4, "minecraft:oak_log")
        sch.set_block(i, 2, 0, "minecraft:oak_log")
        sch.set_block(i, 2, 4, "minecraft:oak_log")
        
        # Side walls
        sch.set_block(0, 1, i, "minecraft:oak_log")
        sch.set_block(4, 1, i, "minecraft:oak_log")
        sch.set_block(0, 2, i, "minecraft:oak_log")
        sch.set_block(4, 2, i, "minecraft:oak_log")
    
    # Add a door
    sch.set_block_with_properties(2, 1, 0, "minecraft:oak_door", {"half": "lower"})
    sch.set_block_with_properties(2, 2, 0, "minecraft:oak_door", {"half": "upper"})
    
    # Roof
    for x in range(5):
        for z in range(5):
            sch.set_block(x, 3, z, "minecraft:oak_planks")
    
    # Save as Litematic
    litematic_data = sch.to_litematic()
    with open("python_house.litematic", "wb") as f:
        f.write(litematic_data)
    print("Saved as python_house.litematic")
    
    # Save as classic schematic
    schematic_data = sch.to_schematic()
    with open("python_house.schematic", "wb") as f:
        f.write(schematic_data)
    print("Saved as python_house.schematic")
    
    # Load it back
    with open("python_house.litematic", "rb") as f:
        data = f.read()
    
    loaded_sch = nuc.Schematic("Loaded House")
    loaded_sch.from_data(data)  # Auto-detects format
    
    print(f"Loaded schematic dimensions: {loaded_sch.dimensions}")
    print(f"Loaded schematic block count: {loaded_sch.block_count}")
    print()
    
    return loaded_sch


def advanced_features(sch):
    """Demonstrate advanced features"""
    print("=== Advanced Features ===")
    
    # Query blocks
    block_at_origin = sch.get_block(0, 0, 0)
    if block_at_origin:
        print(f"Block at origin: {block_at_origin.name}")
        print(f"Properties: {block_at_origin.properties}")
    
    # Get all blocks
    all_blocks = sch.get_all_blocks()
    print(f"Total blocks in schematic: {len(all_blocks)}")
    
    # Check for block entities
    entities = sch.get_all_block_entities()
    if entities:
        print(f"Found {len(entities)} block entities")
        for entity in entities:
            print(f"  Entity at {entity['position']}: {entity['id']}")
    
    # Print ASCII representation
    print("\nASCII representation:")
    print(sch.print_schematic())
    
    # Debug info
    print("\nDebug info:")
    print(nuc.debug_schematic(sch))
    print()


def chunk_operations():
    """Demonstrate chunk-based operations"""
    print("=== Chunk Operations ===")
    
    # Create a larger schematic
    sch = nuc.Schematic("Chunk Demo")
    
    # Fill a 10x10x10 area with different blocks
    for x in range(10):
        for y in range(10):
            for z in range(10):
                if (x + y + z) % 3 == 0:
                    sch.set_block(x, y, z, "minecraft:stone")
                elif (x + y + z) % 3 == 1:
                    sch.set_block(x, y, z, "minecraft:dirt")
                else:
                    sch.set_block(x, y, z, "minecraft:grass_block")
    
    # Get chunks with different strategies
    chunks = sch.get_chunks(5, 5, 5, strategy="bottom_up")
    print(f"Generated {len(chunks)} chunks using bottom_up strategy")
    
    chunks_random = sch.get_chunks(5, 5, 5, strategy="random")
    print(f"Generated {len(chunks_random)} chunks using random strategy")
    
    # Get chunks with camera distance
    chunks_camera = sch.get_chunks(
        5, 5, 5, 
        strategy="distance_to_camera", 
        camera_x=5.0, camera_y=5.0, camera_z=5.0
    )
    print(f"Generated {len(chunks_camera)} chunks ordered by camera distance")
    
    print()


def copy_region_demo():
    """Demonstrate region copying"""
    print("=== Region Copying Demo ===")
    
    # Create source schematic with a pattern
    source = nuc.Schematic("Source")
    
    # Create a 3x3x3 pattern
    for x in range(3):
        for y in range(3):
            for z in range(3):
                if x == 1 and y == 1 and z == 1:
                    source.set_block(x, y, z, "minecraft:diamond_block")
                else:
                    source.set_block(x, y, z, "minecraft:stone")
    
    # Create destination schematic
    dest = nuc.Schematic("Destination")
    
    # Copy the pattern multiple times
    for i in range(3):
        dest.copy_region(
            source,
            0, 0, 0,  # min coordinates
            2, 2, 2,  # max coordinates  
            i * 4, 0, 0,  # target position
            []  # no excluded blocks
        )
    
    print(f"Copied pattern 3 times. Destination has {dest.block_count} blocks")
    
    # Copy again but exclude diamond blocks
    dest.copy_region(
        source,
        0, 0, 0,
        2, 2, 2,
        0, 4, 0,
        ["minecraft:diamond_block"]  # exclude diamonds
    )
    
    print(f"Copied without diamonds. Destination now has {dest.block_count} blocks")
    print()


def main():
    """Main example function"""
    print("Nucleation Python Binding Example")
    print("=" * 40)
    
    # Run all examples
    basic_sch = basic_usage()
    block_state_demo()
    file_sch = file_operations()
    advanced_features(file_sch)
    chunk_operations()
    copy_region_demo()
    
    print("Example completed successfully!")
    
    # Clean up generated files (optional)
    import os
    try:
        os.remove("python_house.litematic")
        os.remove("python_house.schematic")
        print("Cleaned up generated files")
    except FileNotFoundError:
        pass


if __name__ == "__main__":
    main()
