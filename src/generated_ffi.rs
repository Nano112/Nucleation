// Auto-generated FFI bindings for nucleation.
// This file is generated from api_definition.rs - DO NOT EDIT MANUALLY!

#![cfg(feature = "ffi")]

use std::os::raw::{c_char, c_uchar, c_int, c_float};
use std::ffi::{CStr, CString};
use std::collections::HashMap;
use std::ptr;

use crate::{
    UniversalSchematic,
    BlockState,
    formats::{litematic, schematic},
    print_utils::{format_schematic, format_json_schematic},
};

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
pub struct CProperty {
    key: *mut c_char,
    value: *mut c_char,
}

#[repr(C)]
pub struct CPropertyArray {
    data: *mut CProperty,
    len: usize,
}

// Wrapper types
pub struct BlockStateHandle(*mut BlockState);
pub struct SchematicHandle(*mut UniversalSchematic);

// Memory management functions

#[no_mangle]
pub extern "C" fn free_byte_array(array: ByteArray) {
    if !array.data.is_null() {
        unsafe {
            let _ = Vec::from_raw_parts(array.data, array.len, array.len);
        }
    }
}

#[no_mangle]
pub extern "C" fn free_string_array(array: StringArray) {
    if !array.data.is_null() {
        unsafe {
            let strings = Vec::from_raw_parts(array.data, array.len, array.len);
            for s in strings {
                let _ = CString::from_raw(s);
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn free_string(string: *mut c_char) {
    if !string.is_null() {
        unsafe {
            let _ = CString::from_raw(string);
        }
    }
}

// BlockState functions

#[no_mangle]
pub extern "C" fn blockstate_new(name: *const c_char) -> *mut BlockStateHandle {
    let name_str = unsafe { CStr::from_ptr(name).to_string_lossy().into_owned() };
    let block_state = BlockState::new(name_str);
    let handle = BlockStateHandle(Box::into_raw(Box::new(block_state)));
    Box::into_raw(Box::new(handle))
}

#[no_mangle]
pub extern "C" fn blockstate_free(handle: *mut BlockStateHandle) {
    if !handle.is_null() {
        unsafe {
            let wrapper = Box::from_raw(handle);
            let _ = Box::from_raw(wrapper.0);
        }
    }
}

#[no_mangle]
pub extern "C" fn blockstate_with_property(handle: *mut BlockStateHandle, key: *const c_char, value: *const c_char) -> *mut BlockStateHandle {
    if handle.is_null() { return ptr::null_mut(); }
    let obj = unsafe { &mut *(*handle).0 };
    let key_str = unsafe { CStr::from_ptr(key).to_string_lossy().into_owned() };
    let value_str = unsafe { CStr::from_ptr(value).to_string_lossy().into_owned() };
    let new_state = obj.clone().with_property(key_str, value_str);
    Box::into_raw(Box::new(BlockStateHandle(Box::into_raw(Box::new(new_state)))))
}

#[no_mangle]
pub extern "C" fn blockstate_get_name(handle: *const BlockStateHandle) -> *const c_char {
    if handle.is_null() { return ptr::null_mut(); }
    let obj = unsafe { &*(*handle).0 };
    CString::new(obj.name.clone()).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn blockstate_get_properties(handle: *const BlockStateHandle) -> *const CProperty {
    if handle.is_null() { return ptr::null_mut(); }
    let obj = unsafe { &*(*handle).0 };
    ptr::null_mut()
}

// Schematic functions

#[no_mangle]
pub extern "C" fn schematic_new(name: *const c_char) -> *mut SchematicHandle {
    let name_str = if name.is_null() {
        "Default".to_string()
    } else {
        unsafe { CStr::from_ptr(name).to_string_lossy().into_owned() }
    };
    let schematic = UniversalSchematic::new(name_str);
    let handle = SchematicHandle(Box::into_raw(Box::new(schematic)));
    Box::into_raw(Box::new(handle))
}

#[no_mangle]
pub extern "C" fn schematic_free(handle: *mut SchematicHandle) {
    if !handle.is_null() {
        unsafe {
            let wrapper = Box::from_raw(handle);
            let _ = Box::from_raw(wrapper.0);
        }
    }
}

#[no_mangle]
pub extern "C" fn schematic_from_data(handle: *mut SchematicHandle, data: *const c_uchar) -> c_int {
    if handle.is_null() { return -1; }
    let obj = unsafe { &mut *(*handle).0 };
    let data_slice = data;
    if crate::formats::litematic::is_litematic(data_slice) {
        match crate::formats::litematic::from_litematic(data_slice) {
            Ok(res) => { *obj = res; 0 }
            Err(_) => -2,
        }
    } else if crate::formats::schematic::is_schematic(data_slice) {
        match crate::formats::schematic::from_schematic(data_slice) {
            Ok(res) => { *obj = res; 0 }
            Err(_) => -2,
        }
    } else {
        -3
    }
}

#[no_mangle]
pub extern "C" fn schematic_from_litematic(handle: *mut SchematicHandle, data: *const c_uchar) -> c_int {
    if handle.is_null() { return -1; }
    let obj = unsafe { &mut *(*handle).0 };
    // TODO: Implement from_litematic
    -1
}

#[no_mangle]
pub extern "C" fn schematic_to_litematic(handle: *mut SchematicHandle) -> c_int {
    if handle.is_null() { return -1; }
    let obj = unsafe { &mut *(*handle).0 };
    // TODO: Implement to_litematic
    -1
}

#[no_mangle]
pub extern "C" fn schematic_from_schematic(handle: *mut SchematicHandle, data: *const c_uchar) -> c_int {
    if handle.is_null() { return -1; }
    let obj = unsafe { &mut *(*handle).0 };
    // TODO: Implement from_schematic
    -1
}

#[no_mangle]
pub extern "C" fn schematic_to_schematic(handle: *mut SchematicHandle) -> c_int {
    if handle.is_null() { return -1; }
    let obj = unsafe { &mut *(*handle).0 };
    // TODO: Implement to_schematic
    -1
}

#[no_mangle]
pub extern "C" fn schematic_set_block(handle: *mut SchematicHandle, x: c_int, y: c_int, z: c_int, block_name: *const c_char) -> c_int {
    if handle.is_null() { return 0; }
    let obj = unsafe { &mut *(*handle).0 };
    let block_name_str = unsafe { CStr::from_ptr(block_name).to_string_lossy().into_owned() };
    let block_state = BlockState::new(block_name_str);
    obj.set_block(x, y, z, block_state);
    0
}

#[no_mangle]
pub extern "C" fn schematic_set_block_with_properties(handle: *mut SchematicHandle, x: c_int, y: c_int, z: c_int, block_name: *const c_char, properties: *const CProperty) -> c_int {
    if handle.is_null() { return 0; }
    let obj = unsafe { &mut *(*handle).0 };
    // TODO: Implement set_block_with_properties
    0
}

#[no_mangle]
pub extern "C" fn schematic_get_block(handle: *mut SchematicHandle, x: c_int, y: c_int, z: c_int) -> *mut BlockStateHandle {
    if handle.is_null() { return ptr::null_mut(); }
    let obj = unsafe { &mut *(*handle).0 };
    obj.get_block(x, y, z).map_or(ptr::null_mut(), |block_state| {
        CString::new(block_state.name.clone()).unwrap().into_raw()
    })
}

#[no_mangle]
pub extern "C" fn schematic_get_dimensions(handle: *const SchematicHandle) -> IntArray {
    if handle.is_null() { return IntArray { data: ptr::null_mut(), len: 0 }; }
    let obj = unsafe { &*(*handle).0 };
    let (x, y, z) = obj.get_dimensions();
    let dims = vec![x, y, z];
    let mut boxed_slice = dims.into_boxed_slice();
    let ptr = boxed_slice.as_mut_ptr();
    let len = boxed_slice.len();
    std::mem::forget(boxed_slice);
    IntArray { data: ptr, len }
}

#[no_mangle]
pub extern "C" fn schematic_get_block_count(handle: *const SchematicHandle) -> c_int {
    if handle.is_null() { return 0; }
    let obj = unsafe { &*(*handle).0 };
    obj.total_blocks()
}

#[no_mangle]
pub extern "C" fn schematic_get_volume(handle: *const SchematicHandle) -> c_int {
    if handle.is_null() { return 0; }
    let obj = unsafe { &*(*handle).0 };
    obj.total_volume()
}

#[no_mangle]
pub extern "C" fn schematic_get_region_names(handle: *const SchematicHandle) -> IntArray {
    if handle.is_null() { return IntArray { data: ptr::null_mut(), len: 0 }; }
    let obj = unsafe { &*(*handle).0 };
    IntArray { data: ptr::null_mut(), len: 0 }
}

#[no_mangle]
pub extern "C" fn load_schematic(path: *const c_char) -> c_int {
    // TODO: Implement load_schematic
    -1
}

#[no_mangle]
pub extern "C" fn save_schematic(schematic: *mut SchematicHandle, path: *const c_char, format: *const c_char) -> c_int {
    // TODO: Implement save_schematic
    -1
}

#[no_mangle]
pub extern "C" fn debug_schematic(schematic: *mut SchematicHandle) -> *const c_char {
    // TODO: Implement debug_schematic
    ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn debug_json_schematic(schematic: *mut SchematicHandle) -> *const c_char {
    // TODO: Implement debug_json_schematic
    ptr::null_mut()
}

