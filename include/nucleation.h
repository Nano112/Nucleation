#ifndef NUCLEATION_H
#define NUCLEATION_H

// Auto-generated C header for nucleation.
// A high-performance Minecraft schematic parser and utility library.
// This file is generated from api_definition.rs - DO NOT EDIT MANUALLY!

#ifdef __cplusplus
extern "C" {
#endif

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

// C data structures

typedef struct {
    unsigned char* data;
    size_t len;
} ByteArray;

typedef struct {
    char** data;
    size_t len;
} StringArray;

typedef struct {
    int* data;
    size_t len;
} IntArray;

typedef struct {
    char* key;
    char* value;
} CProperty;

typedef struct {
    CProperty* data;
    size_t len;
} CPropertyArray;

// Opaque handles
typedef struct BlockStateHandle BlockStateHandle;
typedef struct SchematicHandle SchematicHandle;

// Memory management
void free_byte_array(ByteArray array);
void free_string_array(StringArray array);
void free_string(char* string);

// BlockState functions
BlockStateHandle* blockstate_new(const char* name);
void blockstate_free(BlockStateHandle* handle);
BlockStateHandle* blockstate_with_property(BlockStateHandle* handle, const char* key, const char* value);
const char* blockstate_get_name(const BlockStateHandle* handle);
const CProperty* blockstate_get_properties(const BlockStateHandle* handle);

// Schematic functions
SchematicHandle* schematic_new(const char* name);
void schematic_free(SchematicHandle* handle);
int schematic_from_data(SchematicHandle* handle, const unsigned char* data);
int schematic_from_litematic(SchematicHandle* handle, const unsigned char* data);
int schematic_to_litematic(SchematicHandle* handle);
int schematic_from_schematic(SchematicHandle* handle, const unsigned char* data);
int schematic_to_schematic(SchematicHandle* handle);
void schematic_set_block(SchematicHandle* handle, int x, int y, int z, const char* block_name);
void schematic_set_block_with_properties(SchematicHandle* handle, int x, int y, int z, const char* block_name, const CProperty* properties);
BlockStateHandle* schematic_get_block(SchematicHandle* handle, int x, int y, int z);
IntArray schematic_get_dimensions(const SchematicHandle* handle);
int schematic_get_block_count(const SchematicHandle* handle);
int schematic_get_volume(const SchematicHandle* handle);
IntArray schematic_get_region_names(const SchematicHandle* handle);

int load_schematic(const char* path);
int save_schematic(SchematicHandle* schematic, const char* path, const char* format);
const char* debug_schematic(SchematicHandle* schematic);
const char* debug_json_schematic(SchematicHandle* schematic);
#ifdef __cplusplus
}
#endif

#endif // NUCLEATION_H
