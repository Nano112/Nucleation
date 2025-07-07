//! WASM binding generator
//! 
//! Generates wasm-bindgen compatible Rust code from API definitions

use crate::api_definition::{ApiDefinition, ApiClass, ApiMethod, ApiFunction, ApiParam, ApiType, ApiProperty};
use std::fmt::Write;

pub fn generate_wasm_bindings(api: &ApiDefinition) -> String {
    let mut output = String::new();
    
    // Write header
    writeln!(&mut output, "// Auto-generated WASM bindings for {}.", api.name).unwrap();
    writeln!(&mut output, "// This file is generated from api_definition.rs - DO NOT EDIT MANUALLY!").unwrap();
    writeln!(&mut output).unwrap();
    
    // Write imports
    write_wasm_imports(&mut output);
    
    // Generate wrapper structs for each class
    for class in &api.classes {
        write_wasm_class(&mut output, class);
    }
    
    // Generate free functions
    for function in &api.functions {
        write_wasm_function(&mut output, function);
    }
    
    output
}

fn write_wasm_imports(output: &mut String) {
    writeln!(output, "use wasm_bindgen::prelude::*;").unwrap();
    writeln!(output, "use js_sys::{{self, Array, Object, Reflect}};").unwrap();
    writeln!(output, "use web_sys::console;").unwrap();
    writeln!(output, "use crate::{{").unwrap();
    writeln!(output, "    UniversalSchematic,").unwrap();
    writeln!(output, "    BlockState,").unwrap();
    writeln!(output, "    formats::{{litematic, schematic}},").unwrap();
    writeln!(output, "    print_utils::{{format_schematic, format_json_schematic}},").unwrap();
    writeln!(output, "}};").unwrap();
    writeln!(output, "use std::collections::HashMap;").unwrap();
    writeln!(output).unwrap();
    
    writeln!(output, "#[wasm_bindgen(start)]").unwrap();
    writeln!(output, "pub fn start() {{").unwrap();
    writeln!(output, "    console::log_1(&\"Initializing nucleation\".into());").unwrap();
    writeln!(output, "}}").unwrap();
    writeln!(output).unwrap();
}

fn write_wasm_class(output: &mut String, class: &ApiClass) {
    let wrapper_name = format!("{}Wrapper", class.name);
    
    // Write class documentation
    writeln!(output, "/// {}", class.docs).unwrap();
    writeln!(output, "#[wasm_bindgen]").unwrap();
    writeln!(output, "pub struct {} {{ inner: {} }}", wrapper_name, get_rust_type(&class.name)).unwrap();
    writeln!(output).unwrap();
    
    // Write implementation
    writeln!(output, "#[wasm_bindgen]").unwrap();
    writeln!(output, "impl {} {{", wrapper_name).unwrap();
    
    for method in &class.methods {
        write_wasm_method(output, method, &class.name);
    }
    
    // Write property getters
    for property in &class.properties {
        write_wasm_property_getter(output, property);
    }
    
    writeln!(output, "}}").unwrap();
    writeln!(output).unwrap();
}

fn write_wasm_method(output: &mut String, method: &ApiMethod, class_name: &str) {
    // Write method documentation
    writeln!(output, "    /// {}", method.docs).unwrap();
    
    // Write method signature
    if method.is_constructor {
        write!(output, "    #[wasm_bindgen(constructor)]").unwrap();
        writeln!(output).unwrap();
        write!(output, "    pub fn new(").unwrap();
    } else if method.is_getter {
        write!(output, "    #[wasm_bindgen(getter)]").unwrap();
        writeln!(output).unwrap();
        write!(output, "    pub fn {}(&self", method.name).unwrap();
    } else {
        write!(output, "    pub fn {}(", method.name).unwrap();
        if !method.is_static {
            write!(output, "&mut self").unwrap();
            if !method.params.is_empty() {
                write!(output, ", ").unwrap();
            }
        }
    }
    
    // Write parameters
    for (i, param) in method.params.iter().enumerate() {
        if i > 0 { write!(output, ", ").unwrap(); }
        write!(output, "{}: {}", param.name, wasm_type(&param.param_type)).unwrap();
    }
    
    write!(output, ")").unwrap();
    
    // Write return type
    let return_type = wasm_type(&method.return_type);
    if method.is_constructor {
        writeln!(output, " -> Self {{").unwrap();
    } else if return_type != "()" {
        writeln!(output, " -> {} {{", return_type).unwrap();
    } else {
        writeln!(output, " {{").unwrap();
    }
    
    // Write method body
    write_wasm_method_body(output, method, class_name);
    
    writeln!(output, "    }}").unwrap();
    writeln!(output).unwrap();
}

fn write_wasm_method_body(output: &mut String, method: &ApiMethod, class_name: &str) {
    if method.is_constructor {
        if class_name == "Schematic" {
            writeln!(output, "        let name = {};", 
                method.params.first()
                    .map(|p| if p.optional { format!("{}.unwrap_or_else(|| \"Default\".to_string())", p.name) } else { p.name.clone() })
                    .unwrap_or_else(|| "\"Default\".to_string()".to_string())).unwrap();
            writeln!(output, "        Self {{ inner: UniversalSchematic::new(name) }}").unwrap();
        } else if class_name == "BlockState" {
            writeln!(output, "        Self {{ inner: BlockState::new({}) }}", method.params[0].name).unwrap();
        }
    } else {
        match method.name.as_str() {
            "from_data" => {
                writeln!(output, "        if crate::formats::litematic::is_litematic({}) {{", method.params[0].name).unwrap();
                writeln!(output, "            self.inner = crate::formats::litematic::from_litematic({})", method.params[0].name).unwrap();
                writeln!(output, "                .map_err(|e| JsValue::from_str(&format!(\"Litematic error: {{}}\", e)))?;").unwrap();
                writeln!(output, "        }} else if crate::formats::schematic::is_schematic({}) {{", method.params[0].name).unwrap();
                writeln!(output, "            self.inner = crate::formats::schematic::from_schematic({})", method.params[0].name).unwrap();
                writeln!(output, "                .map_err(|e| JsValue::from_str(&format!(\"Schematic error: {{}}\", e)))?;").unwrap();
                writeln!(output, "        }} else {{").unwrap();
                writeln!(output, "            return Err(JsValue::from_str(\"Unknown format\"));").unwrap();
                writeln!(output, "        }}").unwrap();
                writeln!(output, "        Ok(())").unwrap();
            },
            "from_litematic" => {
                writeln!(output, "        self.inner = crate::formats::litematic::from_litematic({})", method.params[0].name).unwrap();
                writeln!(output, "            .map_err(|e| JsValue::from_str(&format!(\"Litematic error: {{}}\", e)))?;").unwrap();
                writeln!(output, "        Ok(())").unwrap();
            },
            "to_litematic" => {
                writeln!(output, "        crate::formats::litematic::to_litematic(&self.inner)").unwrap();
                writeln!(output, "            .map_err(|e| JsValue::from_str(&format!(\"Litematic error: {{}}\", e)))").unwrap();
            },
            "from_schematic" => {
                writeln!(output, "        self.inner = crate::formats::schematic::from_schematic({})", method.params[0].name).unwrap();
                writeln!(output, "            .map_err(|e| JsValue::from_str(&format!(\"Schematic error: {{}}\", e)))?;").unwrap();
                writeln!(output, "        Ok(())").unwrap();
            },
            "to_schematic" => {
                writeln!(output, "        crate::formats::schematic::to_schematic(&self.inner)").unwrap();
                writeln!(output, "            .map_err(|e| JsValue::from_str(&format!(\"Schematic error: {{}}\", e)))").unwrap();
            },
            "set_block" => {
                writeln!(output, "        self.inner.set_block({}, {}, {}, BlockState::new({}.to_string()));", 
                    method.params[0].name, method.params[1].name, method.params[2].name, method.params[3].name).unwrap();
            },
            "set_block_with_properties" => {
                writeln!(output, "        // Convert JsValue properties to HashMap").unwrap();
                writeln!(output, "        let mut props = HashMap::new();").unwrap();
                writeln!(output, "        // Implementation would convert JS object to HashMap").unwrap();
                writeln!(output, "        let block_state = BlockState {{ name: {}.to_string(), properties: props }};", method.params[3].name).unwrap();
                writeln!(output, "        self.inner.set_block({}, {}, {}, block_state);", 
                    method.params[0].name, method.params[1].name, method.params[2].name).unwrap();
            },
            "get_block" => {
                writeln!(output, "        self.inner.get_block({}, {}, {}).cloned().map(|bs| BlockStateWrapper {{ inner: bs }})", 
                    method.params[0].name, method.params[1].name, method.params[2].name).unwrap();
            },
            "with_property" => {
                writeln!(output, "        Self {{ inner: self.inner.clone().with_property({}.to_string(), {}.to_string()) }}", 
                    method.params[0].name, method.params[1].name).unwrap();
            },
            _ => {
                writeln!(output, "        // TODO: Implement {}", method.name).unwrap();
                writeln!(output, "        unimplemented!()").unwrap();
            }
        }
    }
}

fn write_wasm_property_getter(output: &mut String, property: &ApiProperty) {
    writeln!(output, "    /// {}", property.docs).unwrap();
    writeln!(output, "    #[wasm_bindgen(getter)]").unwrap();
    write!(output, "    pub fn {}(&self) -> {}", property.name, wasm_type(&property.property_type)).unwrap();
    writeln!(output, " {{").unwrap();
    
    match property.name.as_str() {
        "name" => writeln!(output, "        self.inner.name.clone()").unwrap(),
        "properties" => {
            writeln!(output, "        let obj = Object::new();").unwrap();
            writeln!(output, "        for (key, value) in &self.inner.properties {{").unwrap();
            writeln!(output, "            Reflect::set(&obj, &JsValue::from_str(key), &JsValue::from_str(value)).unwrap();").unwrap();
            writeln!(output, "        }}").unwrap();
            writeln!(output, "        obj.into()").unwrap();
        },
        "dimensions" => {
            writeln!(output, "        let (x, y, z) = self.inner.get_dimensions();").unwrap();
            writeln!(output, "        vec![x, y, z]").unwrap();
        },
        "block_count" => writeln!(output, "        self.inner.total_blocks()").unwrap(),
        "volume" => writeln!(output, "        self.inner.total_volume()").unwrap(),
        "region_names" => writeln!(output, "        self.inner.get_region_names()").unwrap(),
        _ => writeln!(output, "        unimplemented!()").unwrap(),
    }
    
    writeln!(output, "    }}").unwrap();
    writeln!(output).unwrap();
}

fn write_wasm_function(output: &mut String, function: &ApiFunction) {
    writeln!(output, "/// {}", function.docs).unwrap();
    writeln!(output, "#[wasm_bindgen]").unwrap();
    write!(output, "pub fn {}(", function.name).unwrap();
    
    for (i, param) in function.params.iter().enumerate() {
        if i > 0 { write!(output, ", ").unwrap(); }
        write!(output, "{}: {}", param.name, wasm_type(&param.param_type)).unwrap();
    }
    
    let return_type = wasm_type(&function.return_type);
    if return_type != "()" {
        writeln!(output, ") -> {} {{", return_type).unwrap();
    } else {
        writeln!(output, ") {{").unwrap();
    }
    
    // Write function body
    match function.name.as_str() {
        "load_schematic" => {
            writeln!(output, "    // TODO: Implement load_schematic").unwrap();
            writeln!(output, "    Err(JsValue::from_str(\"Not implemented\"))").unwrap();
        },
        "save_schematic" => {
            writeln!(output, "    // TODO: Implement save_schematic").unwrap();
            writeln!(output, "    Err(JsValue::from_str(\"Not implemented\"))").unwrap();
        },
        "debug_schematic" => {
            writeln!(output, "    format!(\"{{}}\\n{{}}\", {}.debug_info(), crate::print_utils::format_schematic(&{}.inner))", 
                function.params[0].name, function.params[0].name).unwrap();
        },
        "debug_json_schematic" => {
            writeln!(output, "    format!(\"{{}}\\n{{}}\", {}.debug_info(), crate::print_utils::format_json_schematic(&{}.inner))", 
                function.params[0].name, function.params[0].name).unwrap();
        },
        _ => {
            writeln!(output, "    // TODO: Implement {}", function.name).unwrap();
            writeln!(output, "    unimplemented!()").unwrap();
        }
    }
    
    writeln!(output, "}}").unwrap();
    writeln!(output).unwrap();
}

fn wasm_type(api_type: &ApiType) -> String {
    match api_type {
        ApiType::String => "String".to_string(),
        ApiType::I32 => "i32".to_string(),
        ApiType::F32 => "f32".to_string(),
        ApiType::Bool => "bool".to_string(),
        ApiType::Bytes => "&[u8]".to_string(),
        ApiType::Vec(inner) => format!("Vec<{}>", wasm_type(inner)),
        ApiType::HashMap(_, _) => "JsValue".to_string(), // JS objects
        ApiType::Option(inner) => format!("Option<{}>", wasm_type(inner)),
        ApiType::Result(ok, err) => {
            format!("Result<{}, {}>", wasm_type(ok), wasm_type(err))
        },
        ApiType::Custom(name) => format!("{}Wrapper", name),
        ApiType::Void => "()".to_string(),
    }
}

fn get_rust_type(type_name: &str) -> String {
    match type_name {
        "Schematic" => "UniversalSchematic".to_string(),
        "BlockState" => "BlockState".to_string(),
        _ => type_name.to_string(),
    }
}
