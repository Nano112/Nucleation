// Fixed and complete version of src/ffi.rs
#![cfg(feature = "ffi")]

use std::os::raw::{c_char};
use std::ffi::{CStr, CString};
use crate::UniversalSchematic;

#[no_mangle]
pub extern "C" fn schematic_debug_info(schematic: *const UniversalSchematic) -> *mut c_char {
    if schematic.is_null() {
        return CString::new("null schematic").unwrap().into_raw();
    }

    let schematic = unsafe { &*schematic };

    let info = format!(
        "Schematic has {} regions",
        schematic.regions.len()
    );

    CString::new(info).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn print_debug_info(schematic: *const UniversalSchematic) {
    if schematic.is_null() {
        println!("Debug info: null schematic");
        return;
    }

    let debug_ptr = schematic_debug_info(schematic);
    if debug_ptr.is_null() {
        println!("Debug info: error generating debug string");
        return;
    }

    let c_str = unsafe { CStr::from_ptr(debug_ptr) };
    let msg = c_str.to_string_lossy();
    println!("Debug info: {}", msg);
    unsafe { CString::from_raw(debug_ptr) }; // Free after printing
}
