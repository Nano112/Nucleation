use std::os::raw::{c_char, c_uchar, c_int, c_uint};
use std::ffi::{CStr, CString};
use std::collections::HashMap;
use std::ptr;
use crate::{
    UniversalSchematic, 
    BlockState,
    formats::{litematic, schematic},
    print_utils::{format_schematic, format_json_schematic},
    block_position::BlockPosition,
    mchprs_world::MchprsWorld,
    bounding_box::BoundingBox,
};
use mchprs_blocks::BlockPos;
use crate::mchprs_world::generate_truth_table;

// C-compatible data structures
#[repr(C)]
pub struct ByteArray {
    data: *mut c_uchar,
    len: usize,
}

#[repr(C)]
pub struct StringArray {
    data: *mut *mut c_char,
    len: usize,
}

#[repr(C)]
pub struct IntArray {
    data: *mut c_int,
    len: usize,
}

#[repr(C)]
pub struct Position {
    x: c_int,
    y: c_int,
    z: c_int,
}

// Wrapper structs with opaque pointers
pub struct SchematicWrapper(*mut UniversalSchematic);
pub struct MchprsWorldWrapper(*mut MchprsWorld);
pub struct BlockStateWrapper(*mut BlockState);

// Helper functions for memory management
#[no_mangle]
pub extern "C" fn free_byte_array(array: ByteArray) {
    unsafe {
        let _ = Box::from_raw(std::slice::from_raw_parts_mut(array.data, array.len));
    }
}

#[no_mangle]
pub extern "C" fn free_string_array(array: StringArray) {
    unsafe {
        for i in 0..array.len {
            let _ = CString::from_raw(*array.data.add(i));
        }
        let _ = Box::from_raw(std::slice::from_raw_parts_mut(array.data, array.len));
    }
}

#[no_mangle]
pub extern "C" fn free_int_array(array: IntArray) {
    unsafe {
        let _ = Box::from_raw(std::slice::from_raw_parts_mut(array.data, array.len));
    }
}

#[no_mangle]
pub extern "C" fn free_string(string: *mut c_char) {
    unsafe {
        if !string.is_null() {
            let _ = CString::from_raw(string);
        }
    }
}

// Schematic creation and manipulation
#[no_mangle]
pub extern "C" fn schematic_new(name: *const c_char) -> *mut SchematicWrapper {
    let name_str = if name.is_null() {
        "Default".to_string()
    } else {
        unsafe {
            CStr::from_ptr(name).to_string_lossy().into_owned()
        }
    };
    
    let schematic = UniversalSchematic::new(name_str);
    let wrapper = SchematicWrapper(Box::into_raw(Box::new(schematic)));
    Box::into_raw(Box::new(wrapper))
}

#[no_mangle]
pub extern "C" fn schematic_free(schematic: *mut SchematicWrapper) {
    if !schematic.is_null() {
        unsafe {
            let wrapper = Box::from_raw(schematic);
            let _ = Box::from_raw(wrapper.0);
        }
    }
}

// Original format conversion function
#[no_mangle]
pub extern "C" fn convert_schematic(
    input_data: *const c_char,
    input_len: usize,
    output_format: *const c_char,
) -> ByteArray {
    let input_slice = unsafe {
        std::slice::from_raw_parts(input_data as *const u8, input_len)
    };

    let format = unsafe {
        CStr::from_ptr(output_format)
            .to_str()
            .unwrap_or("litematic")
    };

    let result = match format {
        "litematic" => {
            if schematic::is_schematic(input_slice) {
                let schematic = schematic::from_schematic(input_slice).unwrap();
                litematic::to_litematic(&schematic).unwrap()
            } else {
                Vec::new()
            }
        },
        "schem" => {
            if litematic::is_litematic(input_slice) {
                let schematic = litematic::from_litematic(input_slice).unwrap();
                schematic::to_schematic(&schematic).unwrap()
            } else {
                Vec::new()
            }
        },
        _ => Vec::new()
    };

    let mut boxed_slice = result.into_boxed_slice();
    let len = boxed_slice.len();
    let data = Box::into_raw(boxed_slice) as *mut c_uchar;

    ByteArray { data, len }
}

// Format-specific loading and saving
#[no_mangle]
pub extern "C" fn schematic_from_data(
    schematic: *mut SchematicWrapper,
    data: *const c_uchar,
    data_len: usize,
) -> c_int {
    if schematic.is_null() || data.is_null() {
        return -1;
    }

    let data_slice = unsafe {
        std::slice::from_raw_parts(data, data_len)
    };

    unsafe {
        let wrapper = &mut *schematic;
        let schematic = &mut *wrapper.0;

        if litematic::is_litematic(data_slice) {
            match litematic::from_litematic(data_slice) {
                Ok(result) => {
                    *schematic = result;
                    0 // Success
                },
                Err(_) => -2, // Parsing error
            }
        } else if schematic::is_schematic(data_slice) {
            match schematic::from_schematic(data_slice) {
                Ok(result) => {
                    *schematic = result;
                    0 // Success
                },
                Err(_) => -2, // Parsing error
            }
        } else {
            -3 // Unknown format
        }
    }
}

#[no_mangle]
pub extern "C" fn schematic_from_litematic(
    schematic: *mut SchematicWrapper,
    data: *const c_uchar,
    data_len: usize,
) -> c_int {
    if schematic.is_null() || data.is_null() {
        return -1;
    }

    let data_slice = unsafe {
        std::slice::from_raw_parts(data, data_len)
    };

    unsafe {
        let wrapper = &mut *schematic;
        let schematic_ref = &mut *wrapper.0;

        match litematic::from_litematic(data_slice) {
            Ok(result) => {
                *schematic_ref = result;
                0 // Success
            },
            Err(_) => -2, // Parsing error
        }
    }
}

#[no_mangle]
pub extern "C" fn schematic_to_litematic(
    schematic: *const SchematicWrapper,
) -> ByteArray {
    if schematic.is_null() {
        return ByteArray { data: ptr::null_mut(), len: 0 };
    }

    unsafe {
        let wrapper = &*schematic;
        let schematic_ref = &*wrapper.0;

        match litematic::to_litematic(schematic_ref) {
            Ok(result) => {
                let mut boxed_slice = result.into_boxed_slice();
                let len = boxed_slice.len();
                let data = Box::into_raw(boxed_slice) as *mut c_uchar;
                ByteArray { data, len }
            },
            Err(_) => ByteArray { data: ptr::null_mut(), len: 0 },
        }
    }
}

#[no_mangle]
pub extern "C" fn schematic_from_schematic(
    schematic: *mut SchematicWrapper,
    data: *const c_uchar,
    data_len: usize,
) -> c_int {
    if schematic.is_null() || data.is_null() {
        return -1;
    }

    let data_slice = unsafe {
        std::slice::from_raw_parts(data, data_len)
    };

    unsafe {
        let wrapper = &mut *schematic;
        let schematic_ref = &mut *wrapper.0;

        match schematic::from_schematic(data_slice) {
            Ok(result) => {
                *schematic_ref = result;
                0 // Success
            },
            Err(_) => -2, // Parsing error
        }
    }
}

#[no_mangle]
pub extern "C" fn schematic_to_schematic(
    schematic: *const SchematicWrapper,
) -> ByteArray {
    if schematic.is_null() {
        return ByteArray { data: ptr::null_mut(), len: 0 };
    }

    unsafe {
        let wrapper = &*schematic;
        let schematic_ref = &*wrapper.0;

        match schematic::to_schematic(schematic_ref) {
            Ok(result) => {
                let mut boxed_slice = result.into_boxed_slice();
                let len = boxed_slice.len();
                let data = Box::into_raw(boxed_slice) as *mut c_uchar;
                ByteArray { data, len }
            },
            Err(_) => ByteArray { data: ptr::null_mut(), len: 0 },
        }
    }
}

// Block manipulation
#[no_mangle]
pub extern "C" fn schematic_set_block(
    schematic: *mut SchematicWrapper,
    x: c_int,
    y: c_int,
    z: c_int,
    block_name: *const c_char,
) -> c_int {
    if schematic.is_null() || block_name.is_null() {
        return -1;
    }

    unsafe {
        let wrapper = &mut *schematic;
        let schematic_ref = &mut *wrapper.0;
        
        let block_name_str = CStr::from_ptr(block_name)
            .to_str()
            .unwrap_or("minecraft:stone")
            .to_string();
            
        schematic_ref.set_block(x, y, z, BlockState::new(block_name_str));
        0 // Success
    }
}

#[no_mangle]
pub extern "C" fn schematic_set_block_from_string(
    schematic: *mut SchematicWrapper,
    x: c_int,
    y: c_int,
    z: c_int,
    block_string: *const c_char,
) -> c_int {
    if schematic.is_null() || block_string.is_null() {
        return -1;
    }

    unsafe {
        let wrapper = &mut *schematic;
        let schematic_ref = &mut *wrapper.0;
        
        let block_string_str = CStr::from_ptr(block_string)
            .to_str()
            .unwrap_or("minecraft:stone")
            .to_string();
            
        match schematic_ref.set_block_from_string(x, y, z, &block_string_str) {
            Ok(_) => 0, // Success
            Err(_) => -2, // Parse error
        }
    }
}

// Simple properties container for the C API
#[repr(C)]
pub struct Property {
    key: *const c_char,
    value: *const c_char,
}

#[no_mangle]
pub extern "C" fn schematic_set_block_with_properties(
    schematic: *mut SchematicWrapper,
    x: c_int,
    y: c_int,
    z: c_int,
    block_name: *const c_char,
    properties: *const Property,
    properties_len: usize,
) -> c_int {
    if schematic.is_null() || block_name.is_null() {
        return -1;
    }

    unsafe {
        let wrapper = &mut *schematic;
        let schematic_ref = &mut *wrapper.0;
        
        let block_name_str = CStr::from_ptr(block_name)
            .to_str()
            .unwrap_or("minecraft:stone")
            .to_string();
        
        let mut props = HashMap::new();
        
        if !properties.is_null() && properties_len > 0 {
            let props_slice = std::slice::from_raw_parts(properties, properties_len);
            
            for prop in props_slice {
                if !prop.key.is_null() && !prop.value.is_null() {
                    let key = CStr::from_ptr(prop.key).to_string_lossy().into_owned();
                    let value = CStr::from_ptr(prop.value).to_string_lossy().into_owned();
                    props.insert(key, value);
                }
            }
        }
        
        let block_state = BlockState {
            name: block_name_str,
            properties: props,
        };
        
        schematic_ref.set_block(x, y, z, block_state);
        0 // Success
    }
}

#[no_mangle]
pub extern "C" fn schematic_get_block(
    schematic: *const SchematicWrapper,
    x: c_int,
    y: c_int,
    z: c_int,
) -> *mut c_char {
    if schematic.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        let wrapper = &*schematic;
        let schematic_ref = &*wrapper.0;
        
        match schematic_ref.get_block(x, y, z) {
            Some(block_state) => {
                CString::new(block_state.name.clone())
                    .unwrap_or(CString::new("").unwrap())
                    .into_raw()
            },
            None => ptr::null_mut(),
        }
    }
}

#[no_mangle]
pub extern "C" fn schematic_get_block_with_properties(
    schematic: *const SchematicWrapper,
    x: c_int,
    y: c_int,
    z: c_int,
) -> *mut BlockStateWrapper {
    if schematic.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        let wrapper = &*schematic;
        let schematic_ref = &*wrapper.0;
        
        match schematic_ref.get_block(x, y, z).cloned() {
            Some(block_state) => {
                let block_wrapper = BlockStateWrapper(Box::into_raw(Box::new(block_state)));
                Box::into_raw(Box::new(block_wrapper))
            },
            None => ptr::null_mut(),
        }
    }
}

// Region copying
#[no_mangle]
pub extern "C" fn schematic_copy_region(
    target: *mut SchematicWrapper,
    source: *const SchematicWrapper,
    min_x: c_int,
    min_y: c_int,
    min_z: c_int,
    max_x: c_int,
    max_y: c_int,
    max_z: c_int,
    target_x: c_int,
    target_y: c_int,
    target_z: c_int,
    excluded_blocks: *const *const c_char,
    excluded_blocks_len: usize,
) -> c_int {
    if target.is_null() || source.is_null() {
        return -1;
    }

    unsafe {
        let target_wrapper = &mut *target;
        let target_schematic = &mut *target_wrapper.0;
        
        let source_wrapper = &*source;
        let source_schematic = &*source_wrapper.0;
        
        let bounds = BoundingBox::new(
            (min_x, min_y, min_z),
            (max_x, max_y, max_z)
        );
        
        let mut excluded = Vec::new();
        
        if !excluded_blocks.is_null() && excluded_blocks_len > 0 {
            let excluded_slice = std::slice::from_raw_parts(excluded_blocks, excluded_blocks_len);
            
            for block_ptr in excluded_slice {
                if !block_ptr.is_null() {
                    let block_string = CStr::from_ptr(*block_ptr).to_string_lossy().into_owned();
                    
                    match UniversalSchematic::parse_block_string(&block_string) {
                        Ok((block_state, _)) => excluded.push(block_state),
                        Err(_) => return -3, // Invalid block string
                    }
                }
            }
        }
        
        match target_schematic.copy_region(
            source_schematic,
            &bounds,
            (target_x, target_y, target_z),
            &excluded
        ) {
            Ok(_) => 0, // Success
            Err(_) => -2, // Copy error
        }
    }
}

// Dimension information
#[no_mangle]
pub extern "C" fn schematic_get_dimensions(
    schematic: *const SchematicWrapper,
) -> IntArray {
    if schematic.is_null() {
        return IntArray { data: ptr::null_mut(), len: 0 };
    }

    unsafe {
        let wrapper = &*schematic;
        let schematic_ref = &*wrapper.0;
        
        let (x, y, z) = schematic_ref.get_dimensions();
        let dims = vec![x, y, z];
        
        let mut boxed_slice = dims.into_boxed_slice();
        let len = boxed_slice.len();
        let data = Box::into_raw(boxed_slice) as *mut c_int;
        
        IntArray { data, len }
    }
}

#[no_mangle]
pub extern "C" fn schematic_get_block_count(
    schematic: *const SchematicWrapper,
) -> c_int {
    if schematic.is_null() {
        return 0;
    }

    unsafe {
        let wrapper = &*schematic;
        let schematic_ref = &*wrapper.0;
        
        schematic_ref.total_blocks()
    }
}

#[no_mangle]
pub extern "C" fn schematic_get_volume(
    schematic: *const SchematicWrapper,
) -> c_int {
    if schematic.is_null() {
        return 0;
    }

    unsafe {
        let wrapper = &*schematic;
        let schematic_ref = &*wrapper.0;
        
        schematic_ref.total_volume()
    }
}

#[no_mangle]
pub extern "C" fn schematic_get_region_names(
    schematic: *const SchematicWrapper,
) -> StringArray {
    if schematic.is_null() {
        return StringArray { data: ptr::null_mut(), len: 0 };
    }

    unsafe {
        let wrapper = &*schematic;
        let schematic_ref = &*wrapper.0;
        
        let names = schematic_ref.get_region_names();
        let mut string_ptrs = Vec::with_capacity(names.len());
        
        for name in names {
            let c_string = CString::new(name).unwrap();
            string_ptrs.push(c_string.into_raw());
        }
        
        let mut boxed_slice = string_ptrs.into_boxed_slice();
        let len = boxed_slice.len();
        let data = Box::into_raw(boxed_slice) as *mut *mut c_char;
        
        StringArray { data, len }
    }
}

// Block entity support
#[repr(C)]
pub struct BlockEntity {
    id: *mut c_char,
    x: c_int,
    y: c_int,
    z: c_int,
    // For simplicity, the NBT data is provided as a JSON string
    nbt_json: *mut c_char,
}

#[no_mangle]
pub extern "C" fn free_block_entity(entity: *mut BlockEntity) {
    if !entity.is_null() {
        unsafe {
            let entity_ref = &mut *entity;
            
            if !entity_ref.id.is_null() {
                let _ = CString::from_raw(entity_ref.id);
            }
            
            if !entity_ref.nbt_json.is_null() {
                let _ = CString::from_raw(entity_ref.nbt_json);
            }
            
            let _ = Box::from_raw(entity);
        }
    }
}

#[no_mangle]
pub extern "C" fn schematic_get_block_entity(
    schematic: *const SchematicWrapper,
    x: c_int,
    y: c_int,
    z: c_int,
) -> *mut BlockEntity {
    if schematic.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        let wrapper = &*schematic;
        let schematic_ref = &*wrapper.0;
        
        let block_position = BlockPosition { x, y, z };
        
        match schematic_ref.get_block_entity(block_position) {
            Some(block_entity) => {
                let id_cstring = CString::new(block_entity.id.clone()).unwrap();
                let nbt_json = serde_json::to_string(&block_entity.nbt).unwrap_or_default();
                let nbt_cstring = CString::new(nbt_json).unwrap();
                
                let entity = BlockEntity {
                    id: id_cstring.into_raw(),
                    x: block_entity.position.0,
                    y: block_entity.position.1,
                    z: block_entity.position.2,
                    nbt_json: nbt_cstring.into_raw(),
                };
                
                Box::into_raw(Box::new(entity))
            },
            None => ptr::null_mut(),
        }
    }
}

// Simulation support
#[no_mangle]
pub extern "C" fn mchprs_world_new(
    schematic: *const SchematicWrapper,
) -> *mut MchprsWorldWrapper {
    if schematic.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        let wrapper = &*schematic;
        let schematic_ref = &*wrapper.0;
        
        match MchprsWorld::new(schematic_ref.clone()) {
            Ok(world) => {
                let world_wrapper = MchprsWorldWrapper(Box::into_raw(Box::new(world)));
                Box::into_raw(Box::new(world_wrapper))
            },
            Err(_) => ptr::null_mut(),
        }
    }
}

#[no_mangle]
pub extern "C" fn mchprs_world_free(world: *mut MchprsWorldWrapper) {
    if !world.is_null() {
        unsafe {
            let wrapper = Box::from_raw(world);
            let _ = Box::from_raw(wrapper.0);
        }
    }
}

#[no_mangle]
pub extern "C" fn mchprs_world_on_use_block(
    world: *mut MchprsWorldWrapper,
    x: c_int,
    y: c_int,
    z: c_int,
) {
    if world.is_null() {
        return;
    }

    unsafe {
        let wrapper = &mut *world;
        let world_ref = &mut *wrapper.0;
        
        world_ref.on_use_block(BlockPos::new(x, y, z));
    }
}

#[no_mangle]
pub extern "C" fn mchprs_world_tick(
    world: *mut MchprsWorldWrapper,
    number_of_ticks: c_uint,
) {
    if world.is_null() {
        return;
    }

    unsafe {
        let wrapper = &mut *world;
        let world_ref = &mut *wrapper.0;
        
        world_ref.tick(number_of_ticks);
    }
}

#[no_mangle]
pub extern "C" fn mchprs_world_flush(
    world: *mut MchprsWorldWrapper,
) {
    if world.is_null() {
        return;
    }

    unsafe {
        let wrapper = &mut *world;
        let world_ref = &mut *wrapper.0;
        
        world_ref.flush();
    }
}

#[no_mangle]
pub extern "C" fn mchprs_world_is_lit(
    world: *const MchprsWorldWrapper,
    x: c_int,
    y: c_int,
    z: c_int,
) -> c_int {
    if world.is_null() {
        return 0;
    }

    unsafe {
        let wrapper = &*world;
        let world_ref = &*wrapper.0;
        
        if world_ref.is_lit(BlockPos::new(x, y, z)) {
            1
        } else {
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn mchprs_world_get_lever_power(
    world: *const MchprsWorldWrapper,
    x: c_int,
    y: c_int,
    z: c_int,
) -> c_int {
    if world.is_null() {
        return 0;
    }

    unsafe {
        let wrapper = &*world;
        let world_ref = &*wrapper.0;
        
        if world_ref.get_lever_power(BlockPos::new(x, y, z)) {
            1
        } else {
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn mchprs_world_get_redstone_power(
    world: *const MchprsWorldWrapper,
    x: c_int,
    y: c_int,
    z: c_int,
) -> c_uchar {
    if world.is_null() {
        return 0;
    }

    unsafe {
        let wrapper = &*world;
        let world_ref = &*wrapper.0;
        
        world_ref.get_redstone_power(BlockPos::new(x, y, z))
    }
}

// BlockState handling
#[no_mangle]
pub extern "C" fn blockstate_new(
    name: *const c_char,
) -> *mut BlockStateWrapper {
    if name.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        let name_str = CStr::from_ptr(name).to_string_lossy().into_owned();
        let block_state = BlockState::new(name_str);
        let wrapper = BlockStateWrapper(Box::into_raw(Box::new(block_state)));
        Box::into_raw(Box::new(wrapper))
    }
}

#[no_mangle]
pub extern "C" fn blockstate_free(block_state: *mut BlockStateWrapper) {
    if !block_state.is_null() {
        unsafe {
            let wrapper = Box::from_raw(block_state);
            let _ = Box::from_raw(wrapper.0);
        }
    }
}

#[no_mangle]
pub extern "C" fn blockstate_with_property(
    block_state: *mut BlockStateWrapper,
    key: *const c_char,
    value: *const c_char,
) -> *mut BlockStateWrapper {
    if block_state.is_null() || key.is_null() || value.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        let wrapper = &*block_state;
        let state = &*wrapper.0;
        
        let key_str = CStr::from_ptr(key).to_string_lossy().into_owned();
        let value_str = CStr::from_ptr(value).to_string_lossy().into_owned();
        
        let new_state = state.clone().with_property(key_str, value_str);
        let new_wrapper = BlockStateWrapper(Box::into_raw(Box::new(new_state)));
        Box::into_raw(Box::new(new_wrapper))
    }
}

#[no_mangle]
pub extern "C" fn blockstate_get_name(
    block_state: *const BlockStateWrapper,
) -> *mut c_char {
    if block_state.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        let wrapper = &*block_state;
        let state = &*wrapper.0;
        
        CString::new(state.name.clone())
            .unwrap_or(CString::new("").unwrap())
            .into_raw()
    }
}

// Utility functions
#[no_mangle]
pub extern "C" fn schematic_print(
    schematic: *const SchematicWrapper,
) -> *mut c_char {
    if schematic.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        let wrapper = &*schematic;
        let schematic_ref = &*wrapper.0;
        
        let formatted = format_schematic(schematic_ref);
        
        CString::new(formatted)
            .unwrap_or(CString::new("").unwrap())
            .into_raw()
    }
}

#[no_mangle]
pub extern "C" fn schematic_debug_info(
    schematic: *const SchematicWrapper,
) -> *mut c_char {
    if schematic.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        let wrapper = &*schematic;
        let schematic_ref = &*wrapper.0;
        
        // Create a local String to ensure it lives long enough
        let name_string: String;
        let name = match &schematic_ref.metadata.name {
            Some(n) => n.as_str(),
            None => {
                name_string = "Unnamed".to_string();
                &name_string
            }
        };
        let region_count = schematic_ref.regions.len();
        
        let info = format!("Schematic name: {}, Regions: {}", name, region_count);
        
        CString::new(info)
            .unwrap_or(CString::new("").unwrap())
            .into_raw()
    }
}