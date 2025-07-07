//! FFI binding generator
//! 
//! Generates C-compatible FFI bindings and header files from API definitions

use crate::api_definition::{ApiDefinition, ApiClass, ApiMethod, ApiFunction, ApiParam, ApiType, ApiProperty};
use std::fmt::Write;

pub fn generate_ffi_bindings(api: &ApiDefinition) -> String {
    let mut output = String::new();
    
    // Write header
    writeln!(&mut output, "// Auto-generated FFI bindings for {}.", api.name).unwrap();
    writeln!(&mut output, "// This file is generated from api_definition.rs - DO NOT EDIT MANUALLY!").unwrap();
    writeln!(&mut output).unwrap();
    writeln!(&mut output, "#![cfg(feature = \"ffi\")]").unwrap();
    writeln!(&mut output).unwrap();
    
    // Write imports
    write_ffi_imports(&mut output);
    
    // Write C-compatible structs
    write_ffi_structs(&mut output);
    
    // Write wrapper types
    write_ffi_wrappers(&mut output, api);
    
    // Write memory management functions
    write_ffi_memory_management(&mut output);
    
    // Generate class functions
    for class in &api.classes {
        write_ffi_class_functions(&mut output, class);
    }
    
    // Generate free functions
    for function in &api.functions {
        write_ffi_function(&mut output, function);
    }
    
    output
}

pub fn generate_c_header(api: &ApiDefinition) -> String {
    let mut output = String::new();
    
    // Write header guard and includes
    writeln!(&mut output, "#ifndef NUCLEATION_H").unwrap();
    writeln!(&mut output, "#define NUCLEATION_H").unwrap();
    writeln!(&mut output).unwrap();
    writeln!(&mut output, "// Auto-generated C header for {}.", api.name).unwrap();
    writeln!(&mut output, "// {}.", api.docs).unwrap();
    writeln!(&mut output, "// This file is generated from api_definition.rs - DO NOT EDIT MANUALLY!").unwrap();
    writeln!(&mut output).unwrap();
    writeln!(&mut output, "#ifdef __cplusplus").unwrap();
    writeln!(&mut output, "extern \"C\" {{").unwrap();
    writeln!(&mut output, "#endif").unwrap();
    writeln!(&mut output).unwrap();
    writeln!(&mut output, "#include <stddef.h>").unwrap();
    writeln!(&mut output, "#include <stdint.h>").unwrap();
    writeln!(&mut output, "#include <stdbool.h>").unwrap();
    writeln!(&mut output).unwrap();
    
    // Write C struct definitions
    write_c_structs(&mut output);
    
    // Write opaque handle types
    writeln!(&mut output, "// Opaque handles").unwrap();
    for class in &api.classes {
        writeln!(&mut output, "typedef struct {}Handle {}Handle;", class.name, class.name).unwrap();
    }
    writeln!(&mut output).unwrap();
    
    // Write function declarations
    writeln!(&mut output, "// Memory management").unwrap();
    write_c_memory_functions(&mut output);
    
    for class in &api.classes {
        write_c_class_functions(&mut output, class);
    }
    
    for function in &api.functions {
        write_c_function_declaration(&mut output, function);
    }
    
    // Close header guard
    writeln!(&mut output, "#ifdef __cplusplus").unwrap();
    writeln!(&mut output, "}}").unwrap();
    writeln!(&mut output, "#endif").unwrap();
    writeln!(&mut output).unwrap();
    writeln!(&mut output, "#endif // NUCLEATION_H").unwrap();
    
    output
}

fn write_ffi_imports(output: &mut String) {
    writeln!(output, "use std::os::raw::{{c_char, c_uchar, c_int, c_float}};").unwrap();
    writeln!(output, "use std::ffi::{{CStr, CString}};").unwrap();
    writeln!(output, "use std::collections::HashMap;").unwrap();
    writeln!(output, "use std::ptr;").unwrap();
    writeln!(output).unwrap();
    writeln!(output, "use crate::{{").unwrap();
    writeln!(output, "    UniversalSchematic,").unwrap();
    writeln!(output, "    BlockState,").unwrap();
    writeln!(output, "    formats::{{litematic, schematic}},").unwrap();
    writeln!(output, "    print_utils::{{format_schematic, format_json_schematic}},").unwrap();
    writeln!(output, "}};").unwrap();
    writeln!(output).unwrap();
}

fn write_ffi_structs(output: &mut String) {
    writeln!(output, "// C-compatible data structures").unwrap();
    writeln!(output).unwrap();
    
    writeln!(output, "#[repr(C)]").unwrap();
    writeln!(output, "pub struct ByteArray {{").unwrap();
    writeln!(output, "    data: *mut c_uchar,").unwrap();
    writeln!(output, "    len: usize,").unwrap();
    writeln!(output, "}}").unwrap();
    writeln!(output).unwrap();
    
    writeln!(output, "#[repr(C)]").unwrap();
    writeln!(output, "pub struct StringArray {{").unwrap();
    writeln!(output, "    data: *mut *mut c_char,").unwrap();
    writeln!(output, "    len: usize,").unwrap();
    writeln!(output, "}}").unwrap();
    writeln!(output).unwrap();
    
    writeln!(output, "#[repr(C)]").unwrap();
    writeln!(output, "pub struct IntArray {{").unwrap();
    writeln!(output, "    data: *mut c_int,").unwrap();
    writeln!(output, "    len: usize,").unwrap();
    writeln!(output, "}}").unwrap();
    writeln!(output).unwrap();
    
    writeln!(output, "#[repr(C)]").unwrap();
    writeln!(output, "pub struct CProperty {{").unwrap();
    writeln!(output, "    key: *mut c_char,").unwrap();
    writeln!(output, "    value: *mut c_char,").unwrap();
    writeln!(output, "}}").unwrap();
    writeln!(output).unwrap();
    
    writeln!(output, "#[repr(C)]").unwrap();
    writeln!(output, "pub struct CPropertyArray {{").unwrap();
    writeln!(output, "    data: *mut CProperty,").unwrap();
    writeln!(output, "    len: usize,").unwrap();
    writeln!(output, "}}").unwrap();
    writeln!(output).unwrap();
}

fn write_ffi_wrappers(output: &mut String, api: &ApiDefinition) {
    writeln!(output, "// Wrapper types").unwrap();
    for class in &api.classes {
        writeln!(output, "pub struct {}Handle(*mut {});", class.name, get_rust_type(&class.name)).unwrap();
    }
    writeln!(output).unwrap();
}

fn write_ffi_memory_management(output: &mut String) {
    writeln!(output, "// Memory management functions").unwrap();
    writeln!(output).unwrap();
    
    writeln!(output, "#[no_mangle]").unwrap();
    writeln!(output, "pub extern \"C\" fn free_byte_array(array: ByteArray) {{").unwrap();
    writeln!(output, "    if !array.data.is_null() {{").unwrap();
    writeln!(output, "        unsafe {{").unwrap();
    writeln!(output, "            let _ = Vec::from_raw_parts(array.data, array.len, array.len);").unwrap();
    writeln!(output, "        }}").unwrap();
    writeln!(output, "    }}").unwrap();
    writeln!(output, "}}").unwrap();
    writeln!(output).unwrap();
    
    writeln!(output, "#[no_mangle]").unwrap();
    writeln!(output, "pub extern \"C\" fn free_string_array(array: StringArray) {{").unwrap();
    writeln!(output, "    if !array.data.is_null() {{").unwrap();
    writeln!(output, "        unsafe {{").unwrap();
    writeln!(output, "            let strings = Vec::from_raw_parts(array.data, array.len, array.len);").unwrap();
    writeln!(output, "            for s in strings {{").unwrap();
    writeln!(output, "                let _ = CString::from_raw(s);").unwrap();
    writeln!(output, "            }}").unwrap();
    writeln!(output, "        }}").unwrap();
    writeln!(output, "    }}").unwrap();
    writeln!(output, "}}").unwrap();
    writeln!(output).unwrap();
    
    writeln!(output, "#[no_mangle]").unwrap();
    writeln!(output, "pub extern \"C\" fn free_string(string: *mut c_char) {{").unwrap();
    writeln!(output, "    if !string.is_null() {{").unwrap();
    writeln!(output, "        unsafe {{").unwrap();
    writeln!(output, "            let _ = CString::from_raw(string);").unwrap();
    writeln!(output, "        }}").unwrap();
    writeln!(output, "    }}").unwrap();
    writeln!(output, "}}").unwrap();
    writeln!(output).unwrap();
}

fn write_ffi_class_functions(output: &mut String, class: &ApiClass) {
    let handle_type = format!("{}Handle", class.name);
    let rust_type = get_rust_type(&class.name);
    
    writeln!(output, "// {} functions", class.name).unwrap();
    writeln!(output).unwrap();
    
    // Constructor
    for method in &class.methods {
        if method.is_constructor {
            writeln!(output, "#[no_mangle]").unwrap();
            write!(output, "pub extern \"C\" fn {}_new(", class.name.to_lowercase()).unwrap();
            
            for (i, param) in method.params.iter().enumerate() {
                if i > 0 { write!(output, ", ").unwrap(); }
                write!(output, "{}: {}", param.name, ffi_type(&param.param_type)).unwrap();
            }
            
            writeln!(output, ") -> *mut {} {{", handle_type).unwrap();
            
            if class.name == "Schematic" {
                if method.params.is_empty() {
                    writeln!(output, "    let schematic = UniversalSchematic::new(\"Default\".to_string());").unwrap();
                } else {
                    writeln!(output, "    let name_str = if {}.is_null() {{", method.params[0].name).unwrap();
                    writeln!(output, "        \"Default\".to_string()").unwrap();
                    writeln!(output, "    }} else {{").unwrap();
                    writeln!(output, "        unsafe {{ CStr::from_ptr({}).to_string_lossy().into_owned() }}", method.params[0].name).unwrap();
                    writeln!(output, "    }};").unwrap();
                    writeln!(output, "    let schematic = UniversalSchematic::new(name_str);").unwrap();
                }
                writeln!(output, "    let handle = {}(Box::into_raw(Box::new(schematic)));", handle_type).unwrap();
            } else if class.name == "BlockState" {
                writeln!(output, "    let name_str = unsafe {{ CStr::from_ptr({}).to_string_lossy().into_owned() }};", method.params[0].name).unwrap();
                writeln!(output, "    let block_state = BlockState::new(name_str);").unwrap();
                writeln!(output, "    let handle = {}(Box::into_raw(Box::new(block_state)));", handle_type).unwrap();
            }
            
            writeln!(output, "    Box::into_raw(Box::new(handle))").unwrap();
            writeln!(output, "}}").unwrap();
            writeln!(output).unwrap();
        }
    }
    
    // Destructor
    writeln!(output, "#[no_mangle]").unwrap();
    writeln!(output, "pub extern \"C\" fn {}_free(handle: *mut {}) {{", class.name.to_lowercase(), handle_type).unwrap();
    writeln!(output, "    if !handle.is_null() {{").unwrap();
    writeln!(output, "        unsafe {{").unwrap();
    writeln!(output, "            let wrapper = Box::from_raw(handle);").unwrap();
    writeln!(output, "            let _ = Box::from_raw(wrapper.0);").unwrap();
    writeln!(output, "        }}").unwrap();
    writeln!(output, "    }}").unwrap();
    writeln!(output, "}}").unwrap();
    writeln!(output).unwrap();
    
    // Methods
    for method in &class.methods {
        if !method.is_constructor {
            write_ffi_method(output, method, class);
        }
    }
    
    // Property getters
    for property in &class.properties {
        write_ffi_property_getter(output, property, class);
    }
}

fn write_ffi_method(output: &mut String, method: &ApiMethod, class: &ApiClass) {
    let handle_type = format!("{}Handle", class.name);
    
    writeln!(output, "#[no_mangle]").unwrap();
    write!(output, "pub extern \"C\" fn {}_{}", class.name.to_lowercase(), method.name).unwrap();
    write!(output, "(handle: *mut {}", handle_type).unwrap();
    
    for param in &method.params {
        write!(output, ", {}: {}", param.name, ffi_type(&param.param_type)).unwrap();
    }
    
    let return_type = ffi_return_type(&method.return_type);
    writeln!(output, ") -> {} {{", return_type).unwrap();
    
    writeln!(output, "    if handle.is_null() {{ return {}; }}", ffi_null_value(&method.return_type)).unwrap();
    writeln!(output, "    let obj = unsafe {{ &mut *(*handle).0 }};").unwrap();
    
    // Write method body
    match method.name.as_str() {
        "from_data" => {
            if method.params.len() >= 2 {
                writeln!(output, "    let data_slice = unsafe {{ std::slice::from_raw_parts({}, {} as usize) }};", method.params[0].name, method.params[1].name).unwrap();
            } else {
                writeln!(output, "    let data_slice = {};", method.params[0].name).unwrap();
            }
            writeln!(output, "    if crate::formats::litematic::is_litematic(data_slice) {{").unwrap();
            writeln!(output, "        match crate::formats::litematic::from_litematic(data_slice) {{").unwrap();
            writeln!(output, "            Ok(res) => {{ *obj = res; 0 }}").unwrap();
            writeln!(output, "            Err(_) => -2,").unwrap();
            writeln!(output, "        }}").unwrap();
            writeln!(output, "    }} else if crate::formats::schematic::is_schematic(data_slice) {{").unwrap();
            writeln!(output, "        match crate::formats::schematic::from_schematic(data_slice) {{").unwrap();
            writeln!(output, "            Ok(res) => {{ *obj = res; 0 }}").unwrap();
            writeln!(output, "            Err(_) => -2,").unwrap();
            writeln!(output, "        }}").unwrap();
            writeln!(output, "    }} else {{").unwrap();
            writeln!(output, "        -3").unwrap();
            writeln!(output, "    }}").unwrap();
        },
        "set_block" => {
            writeln!(output, "    let block_name_str = unsafe {{ CStr::from_ptr({}).to_string_lossy().into_owned() }};", method.params[3].name).unwrap();
            writeln!(output, "    let block_state = BlockState::new(block_name_str);").unwrap();
            writeln!(output, "    obj.set_block({}, {}, {}, block_state);", method.params[0].name, method.params[1].name, method.params[2].name).unwrap();
            writeln!(output, "    0").unwrap();
        },
        "get_block" => {
            writeln!(output, "    obj.get_block({}, {}, {}).map_or(ptr::null_mut(), |block_state| {{", 
                method.params[0].name, method.params[1].name, method.params[2].name).unwrap();
            writeln!(output, "        CString::new(block_state.name.clone()).unwrap().into_raw()").unwrap();
            writeln!(output, "    }})").unwrap();
        },
        "with_property" => {
            writeln!(output, "    let key_str = unsafe {{ CStr::from_ptr({}).to_string_lossy().into_owned() }};", method.params[0].name).unwrap();
            writeln!(output, "    let value_str = unsafe {{ CStr::from_ptr({}).to_string_lossy().into_owned() }};", method.params[1].name).unwrap();
            writeln!(output, "    let new_state = obj.clone().with_property(key_str, value_str);").unwrap();
            writeln!(output, "    Box::into_raw(Box::new({}Handle(Box::into_raw(Box::new(new_state)))))", class.name).unwrap();
        },
        _ => {
            writeln!(output, "    // TODO: Implement {}", method.name).unwrap();
            writeln!(output, "    {}", ffi_null_value(&method.return_type)).unwrap();
        }
    }
    
    writeln!(output, "}}").unwrap();
    writeln!(output).unwrap();
}

fn write_ffi_property_getter(output: &mut String, property: &ApiProperty, class: &ApiClass) {
    let handle_type = format!("{}Handle", class.name);
    
    writeln!(output, "#[no_mangle]").unwrap();
    writeln!(output, "pub extern \"C\" fn {}_get_{}(handle: *const {}) -> {} {{", 
        class.name.to_lowercase(), property.name, handle_type, ffi_type(&property.property_type)).unwrap();
    
    writeln!(output, "    if handle.is_null() {{ return {}; }}", ffi_null_value(&property.property_type)).unwrap();
    writeln!(output, "    let obj = unsafe {{ &*(*handle).0 }};").unwrap();
    
    match property.name.as_str() {
        "name" => writeln!(output, "    CString::new(obj.name.clone()).unwrap().into_raw()").unwrap(),
        "block_count" => writeln!(output, "    obj.total_blocks()").unwrap(),
        "volume" => writeln!(output, "    obj.total_volume()").unwrap(),
        "dimensions" => {
            writeln!(output, "    let (x, y, z) = obj.get_dimensions();").unwrap();
            writeln!(output, "    let dims = vec![x, y, z];").unwrap();
            writeln!(output, "    let mut boxed_slice = dims.into_boxed_slice();").unwrap();
            writeln!(output, "    let ptr = boxed_slice.as_mut_ptr();").unwrap();
            writeln!(output, "    let len = boxed_slice.len();").unwrap();
            writeln!(output, "    std::mem::forget(boxed_slice);").unwrap();
            writeln!(output, "    IntArray {{ data: ptr, len }}").unwrap();
        },
        _ => writeln!(output, "    {}", ffi_null_value(&property.property_type)).unwrap(),
    }
    
    writeln!(output, "}}").unwrap();
    writeln!(output).unwrap();
}

fn write_ffi_function(output: &mut String, function: &ApiFunction) {
    writeln!(output, "#[no_mangle]").unwrap();
    write!(output, "pub extern \"C\" fn {}(", function.name).unwrap();
    
    for (i, param) in function.params.iter().enumerate() {
        if i > 0 { write!(output, ", ").unwrap(); }
        write!(output, "{}: {}", param.name, ffi_type(&param.param_type)).unwrap();
    }
    
    let return_type = ffi_return_type(&function.return_type);
    writeln!(output, ") -> {} {{", return_type).unwrap();
    
    writeln!(output, "    // TODO: Implement {}", function.name).unwrap();
    writeln!(output, "    {}", ffi_null_value(&function.return_type)).unwrap();
    
    writeln!(output, "}}").unwrap();
    writeln!(output).unwrap();
}

fn write_c_structs(output: &mut String) {
    writeln!(output, "// C data structures").unwrap();
    writeln!(output).unwrap();
    
    writeln!(output, "typedef struct {{").unwrap();
    writeln!(output, "    unsigned char* data;").unwrap();
    writeln!(output, "    size_t len;").unwrap();
    writeln!(output, "}} ByteArray;").unwrap();
    writeln!(output).unwrap();
    
    writeln!(output, "typedef struct {{").unwrap();
    writeln!(output, "    char** data;").unwrap();
    writeln!(output, "    size_t len;").unwrap();
    writeln!(output, "}} StringArray;").unwrap();
    writeln!(output).unwrap();
    
    writeln!(output, "typedef struct {{").unwrap();
    writeln!(output, "    int* data;").unwrap();
    writeln!(output, "    size_t len;").unwrap();
    writeln!(output, "}} IntArray;").unwrap();
    writeln!(output).unwrap();
    
    writeln!(output, "typedef struct {{").unwrap();
    writeln!(output, "    char* key;").unwrap();
    writeln!(output, "    char* value;").unwrap();
    writeln!(output, "}} CProperty;").unwrap();
    writeln!(output).unwrap();
    
    writeln!(output, "typedef struct {{").unwrap();
    writeln!(output, "    CProperty* data;").unwrap();
    writeln!(output, "    size_t len;").unwrap();
    writeln!(output, "}} CPropertyArray;").unwrap();
    writeln!(output).unwrap();
}

fn write_c_memory_functions(output: &mut String) {
    writeln!(output, "void free_byte_array(ByteArray array);").unwrap();
    writeln!(output, "void free_string_array(StringArray array);").unwrap();
    writeln!(output, "void free_string(char* string);").unwrap();
    writeln!(output).unwrap();
}

fn write_c_class_functions(output: &mut String, class: &ApiClass) {
    let handle_type = format!("{}Handle", class.name);
    
    writeln!(output, "// {} functions", class.name).unwrap();
    
    // Constructor
    for method in &class.methods {
        if method.is_constructor {
            write!(output, "{}* {}_new(", handle_type, class.name.to_lowercase()).unwrap();
            for (i, param) in method.params.iter().enumerate() {
                if i > 0 { write!(output, ", ").unwrap(); }
                write!(output, "{} {}", c_type(&param.param_type), param.name).unwrap();
            }
            writeln!(output, ");").unwrap();
        }
    }
    
    // Destructor
    writeln!(output, "void {}_free({}* handle);", class.name.to_lowercase(), handle_type).unwrap();
    
    // Methods
    for method in &class.methods {
        if !method.is_constructor {
            write!(output, "{} {}_{}", c_return_type(&method.return_type), class.name.to_lowercase(), method.name).unwrap();
            write!(output, "({}* handle", handle_type).unwrap();
            for param in &method.params {
                write!(output, ", {} {}", c_type(&param.param_type), param.name).unwrap();
            }
            writeln!(output, ");").unwrap();
        }
    }
    
    // Property getters
    for property in &class.properties {
        writeln!(output, "{} {}_get_{}(const {}* handle);", 
            c_type(&property.property_type), class.name.to_lowercase(), property.name, handle_type).unwrap();
    }
    
    writeln!(output).unwrap();
}

fn write_c_function_declaration(output: &mut String, function: &ApiFunction) {
    write!(output, "{} {}(", c_return_type(&function.return_type), function.name).unwrap();
    
    for (i, param) in function.params.iter().enumerate() {
        if i > 0 { write!(output, ", ").unwrap(); }
        write!(output, "{} {}", c_type(&param.param_type), param.name).unwrap();
    }
    
    writeln!(output, ");").unwrap();
}

fn ffi_type(api_type: &ApiType) -> String {
    match api_type {
        ApiType::String => "*const c_char".to_string(),
        ApiType::I32 => "c_int".to_string(),
        ApiType::F32 => "c_float".to_string(),
        ApiType::Bool => "bool".to_string(),
        ApiType::Bytes => "*const c_uchar".to_string(),
        ApiType::Vec(_) => "IntArray".to_string(), // Simplified
        ApiType::HashMap(_, _) => "*const CProperty".to_string(),
        ApiType::Option(inner) => ffi_type(inner), // Simplified
        ApiType::Result(ok, _) => ffi_type(ok), // Simplified
        ApiType::Custom(name) => format!("*mut {}Handle", name),
        ApiType::Void => "c_int".to_string(), // Use int for error codes
    }
}

fn ffi_return_type(api_type: &ApiType) -> String {
    match api_type {
        ApiType::Result(_, _) => "c_int".to_string(), // Error codes
        _ => ffi_type(api_type),
    }
}

fn ffi_null_value(api_type: &ApiType) -> String {
    match api_type {
        ApiType::String => "ptr::null_mut()".to_string(),
        ApiType::I32 => "0".to_string(),
        ApiType::F32 => "0.0".to_string(),
        ApiType::Bool => "false".to_string(),
        ApiType::Bytes => "ptr::null_mut()".to_string(),
        ApiType::Vec(_) => "IntArray { data: ptr::null_mut(), len: 0 }".to_string(),
        ApiType::HashMap(_, _) => "ptr::null_mut()".to_string(),
        ApiType::Option(_) => "ptr::null_mut()".to_string(),
        ApiType::Result(_, _) => "-1".to_string(),
        ApiType::Custom(_) => "ptr::null_mut()".to_string(),
        ApiType::Void => "0".to_string(),
    }
}

fn c_type(api_type: &ApiType) -> String {
    match api_type {
        ApiType::String => "const char*".to_string(),
        ApiType::I32 => "int".to_string(),
        ApiType::F32 => "float".to_string(),
        ApiType::Bool => "bool".to_string(),
        ApiType::Bytes => "const unsigned char*".to_string(),
        ApiType::Vec(_) => "IntArray".to_string(),
        ApiType::HashMap(_, _) => "const CProperty*".to_string(),
        ApiType::Option(inner) => c_type(inner),
        ApiType::Result(ok, _) => c_type(ok),
        ApiType::Custom(name) => format!("{}Handle*", name),
        ApiType::Void => "void".to_string(),
    }
}

fn c_return_type(api_type: &ApiType) -> String {
    match api_type {
        ApiType::Result(_, _) => "int".to_string(),
        _ => c_type(api_type),
    }
}

fn get_rust_type(type_name: &str) -> String {
    match type_name {
        "Schematic" => "UniversalSchematic".to_string(),
        "BlockState" => "BlockState".to_string(),
        _ => type_name.to_string(),
    }
}
