/**
 * Nucleation FFI (C) Binding Example
 * 
 * This example demonstrates how to use the nucleation library from C
 * to create, modify, and save Minecraft schematics.
 * 
 * Build prerequisites:
 *   cargo build --release --features ffi
 *   
 * Compile this example:
 *   gcc -o ffi_example ffi_example.c -L./target/release -lnucleation -lpthread -ldl -lm
 *   
 * Run:
 *   LD_LIBRARY_PATH=./target/release ./ffi_example
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include "nucleation.h"  // Generated header from binding generator

void print_separator(const char* title) {
    printf("=== %s ===\n", title);
}

void basic_usage() {
    print_separator("Basic Usage");
    
    // Create a new empty schematic
    SchematicHandle* sch = schematic_new("C Demo");
    if (!sch) {
        printf("Failed to create schematic\n");
        return;
    }
    
    // Add some basic blocks
    schematic_set_block(sch, 0, 0, 0, "minecraft:stone");
    schematic_set_block(sch, 1, 0, 0, "minecraft:oak_log");
    schematic_set_block(sch, 2, 0, 0, "minecraft:diamond_block");
    
    // Add a block with properties
    CProperty props[] = {
        {"facing", "north"},
        {"half", "bottom"}
    };
    CPropertyArray prop_array = {props, 2};
    schematic_set_block_with_properties(sch, 3, 0, 0, "minecraft:oak_stairs", props);
    
    // Print schematic info
    IntArray dimensions = schematic_get_dimensions(sch);
    printf("Dimensions: [%d, %d, %d]\n", 
           dimensions.data[0], dimensions.data[1], dimensions.data[2]);
    printf("Block count: %d\n", schematic_get_block_count(sch));
    printf("Volume: %d\n", schematic_get_volume(sch));
    printf("\n");
    
    // Clean up
    free_string_array((StringArray){NULL, 0}); // No strings to free in this example
    schematic_free(sch);
}

void blockstate_demo() {
    print_separator("BlockState Demo");
    
    // Create basic block state
    BlockStateHandle* stone = blockstate_new("minecraft:stone");
    if (stone) {
        printf("Stone: %s\n", blockstate_get_name(stone));
        const CProperty* props = blockstate_get_properties(stone);
        printf("Properties: (none for stone)\n");
        blockstate_free(stone);
    }
    
    // Create block state with properties
    BlockStateHandle* stairs = blockstate_new("minecraft:oak_stairs");
    if (stairs) {
        BlockStateHandle* stairs_north = blockstate_with_property(stairs, "facing", "north");
        BlockStateHandle* stairs_complete = blockstate_with_property(stairs_north, "half", "top");
        
        printf("Stairs: %s\n", blockstate_get_name(stairs_complete));
        printf("Properties: facing=north, half=top\n");
        
        blockstate_free(stairs);
        blockstate_free(stairs_north);
        blockstate_free(stairs_complete);
    }
    printf("\n");
}

void file_operations() {
    print_separator("File Operations");
    
    // Create a sample schematic
    SchematicHandle* sch = schematic_new("C File Demo");
    if (!sch) {
        printf("Failed to create schematic\n");
        return;
    }
    
    // Build a small house
    // Floor
    for (int x = 0; x < 5; x++) {
        for (int z = 0; z < 5; z++) {
            schematic_set_block(sch, x, 0, z, "minecraft:oak_planks");
        }
    }
    
    // Walls
    for (int i = 0; i < 5; i++) {
        // Front and back walls
        schematic_set_block(sch, i, 1, 0, "minecraft:oak_log");
        schematic_set_block(sch, i, 1, 4, "minecraft:oak_log");
        schematic_set_block(sch, i, 2, 0, "minecraft:oak_log");
        schematic_set_block(sch, i, 2, 4, "minecraft:oak_log");
        
        // Side walls
        schematic_set_block(sch, 0, 1, i, "minecraft:oak_log");
        schematic_set_block(sch, 4, 1, i, "minecraft:oak_log");
        schematic_set_block(sch, 0, 2, i, "minecraft:oak_log");
        schematic_set_block(sch, 4, 2, i, "minecraft:oak_log");
    }
    
    // Add a door
    CProperty door_props_lower[] = {{"half", "lower"}};
    CProperty door_props_upper[] = {{"half", "upper"}};
    schematic_set_block_with_properties(sch, 2, 1, 0, "minecraft:oak_door", door_props_lower);
    schematic_set_block_with_properties(sch, 2, 2, 0, "minecraft:oak_door", door_props_upper);
    
    // Roof
    for (int x = 0; x < 5; x++) {
        for (int z = 0; z < 5; z++) {
            schematic_set_block(sch, x, 3, z, "minecraft:oak_planks");
        }
    }
    
    // Convert to different formats
    int litematic_result = schematic_to_litematic(sch);
    int schematic_result = schematic_to_schematic(sch);
    
    if (litematic_result == 0) {
        printf("Successfully converted to Litematic format\n");
    } else {
        printf("Failed to convert to Litematic format\n");
    }
    
    if (schematic_result == 0) {
        printf("Successfully converted to classic schematic format\n");
    } else {
        printf("Failed to convert to classic schematic format\n");
    }
    
    // Save using the utility function
    if (save_schematic(sch, "c_house.litematic", "litematic") == 0) {
        printf("Saved as c_house.litematic\n");
    } else {
        printf("Failed to save Litematic file\n");
    }
    
    if (save_schematic(sch, "c_house.schematic", "schematic") == 0) {
        printf("Saved as c_house.schematic\n");
    } else {
        printf("Failed to save schematic file\n");
    }
    
    printf("Schematic dimensions: ");
    IntArray dims = schematic_get_dimensions(sch);
    printf("[%d, %d, %d]\n", dims.data[0], dims.data[1], dims.data[2]);
    printf("Schematic block count: %d\n", schematic_get_block_count(sch));
    printf("\n");
    
    schematic_free(sch);
}

void advanced_features() {
    print_separator("Advanced Features");
    
    // Create a schematic with some blocks
    SchematicHandle* sch = schematic_new("Advanced Demo");
    schematic_set_block(sch, 0, 0, 0, "minecraft:stone");
    schematic_set_block(sch, 1, 0, 0, "minecraft:dirt");
    schematic_set_block(sch, 2, 0, 0, "minecraft:grass_block");
    
    // Query blocks
    BlockStateHandle* block_at_origin = schematic_get_block(sch, 0, 0, 0);
    if (block_at_origin) {
        printf("Block at origin: %s\n", blockstate_get_name(block_at_origin));
        blockstate_free(block_at_origin);
    } else {
        printf("No block at origin\n");
    }
    
    // Get schematic info
    printf("Total volume: %d\n", schematic_get_volume(sch));
    printf("Block count: %d\n", schematic_get_block_count(sch));
    
    // Get region names
    IntArray region_names = schematic_get_region_names(sch);
    printf("Number of regions: %zu\n", region_names.len);
    
    // Debug info
    const char* debug_info = debug_schematic(sch);
    if (debug_info) {
        printf("Debug info: %s\n", debug_info);
        free_string((char*)debug_info);
    }
    
    const char* debug_json = debug_json_schematic(sch);
    if (debug_json) {
        printf("Debug JSON (first 100 chars): %.100s...\n", debug_json);
        free_string((char*)debug_json);
    }
    
    printf("\n");
    
    schematic_free(sch);
}

void memory_management_demo() {
    print_separator("Memory Management Demo");
    
    // Demonstrate proper memory management
    SchematicHandle* sch = schematic_new("Memory Demo");
    
    // Create multiple block states
    BlockStateHandle* blocks[3];
    blocks[0] = blockstate_new("minecraft:stone");
    blocks[1] = blockstate_new("minecraft:dirt");
    blocks[2] = blockstate_new("minecraft:grass_block");
    
    // Use them
    for (int i = 0; i < 3; i++) {
        if (blocks[i]) {
            printf("Block %d: %s\n", i, blockstate_get_name(blocks[i]));
        }
    }
    
    // Properly free all resources
    for (int i = 0; i < 3; i++) {
        if (blocks[i]) {
            blockstate_free(blocks[i]);
        }
    }
    
    schematic_free(sch);
    
    printf("All memory properly freed\n");
    printf("\n");
}

void error_handling_demo() {
    print_separator("Error Handling Demo");
    
    // Test error conditions
    SchematicHandle* null_sch = NULL;
    
    // These should handle NULL gracefully
    printf("Testing NULL schematic handling:\n");
    printf("Block count with NULL: %d\n", schematic_get_block_count(null_sch));
    printf("Volume with NULL: %d\n", schematic_get_volume(null_sch));
    
    const char* debug_null = debug_schematic(null_sch);
    if (debug_null) {
        printf("Debug info for NULL: %s\n", debug_null);
        free_string((char*)debug_null);
    }
    
    // Test with invalid file operations
    SchematicHandle* sch = schematic_new("Error Test");
    int result = save_schematic(sch, "/invalid/path/test.litematic", "litematic");
    printf("Save to invalid path result: %d (should be non-zero)\n", result);
    
    schematic_free(sch);
    printf("\n");
}

int main() {
    printf("Nucleation FFI (C) Binding Example\n");
    printf("=====================================\n\n");
    
    // Run all examples
    basic_usage();
    blockstate_demo();
    file_operations();
    advanced_features();
    memory_management_demo();
    error_handling_demo();
    
    printf("Example completed successfully!\n");
    
    // Clean up generated files
    printf("Cleaning up generated files...\n");
    remove("c_house.litematic");
    remove("c_house.schematic");
    
    return 0;
}
