//! Python binding generator
//! 
//! Generates PyO3 compatible Rust code and Python stub files from API definitions

use crate::api_definition::{ApiDefinition, ApiClass, ApiMethod, ApiFunction, ApiParam, ApiType, ApiProperty};
use std::fmt::Write;

pub fn generate_python_bindings(api: &ApiDefinition) -> String {
    let mut output = String::new();
    
    // Write header
    writeln!(&mut output, "// Auto-generated Python bindings for {}.", api.name).unwrap();
    writeln!(&mut output, "// This file is generated from api_definition.rs - DO NOT EDIT MANUALLY!").unwrap();
    writeln!(&mut output).unwrap();
    writeln!(&mut output, "#![cfg(feature = \"python\")]").unwrap();
    writeln!(&mut output).unwrap();
    
    // Write imports
    write_python_imports(&mut output);
    
    // Generate wrapper classes
    for class in &api.classes {
        write_python_class(&mut output, class);
    }
    
    // Generate free functions
    for function in &api.functions {
        write_python_function(&mut output, function);
    }
    
    // Generate module definition
    write_python_module(&mut output, api);
    
    output
}

pub fn generate_python_stubs(api: &ApiDefinition) -> String {
    let mut output = String::new();
    
    writeln!(&mut output, "\"\"\"").unwrap();
    writeln!(&mut output, "Auto-generated Python stubs for {}.", api.name).unwrap();
    writeln!(&mut output, "{}.", api.docs).unwrap();
    writeln!(&mut output, "").unwrap();
    writeln!(&mut output, "This file is generated from api_definition.rs - DO NOT EDIT MANUALLY!").unwrap();
    writeln!(&mut output, "\"\"\"").unwrap();
    writeln!(&mut output).unwrap();
    
    writeln!(&mut output, "from typing import Optional, List, Dict, Union, Any").unwrap();
    writeln!(&mut output).unwrap();
    
    // Generate class stubs
    for class in &api.classes {
        write_python_class_stub(&mut output, class);
    }
    
    // Generate function stubs
    for function in &api.functions {
        write_python_function_stub(&mut output, function);
    }
    
    output
}

fn write_python_imports(output: &mut String) {
    writeln!(output, "use pyo3::prelude::*;").unwrap();
    writeln!(output, "use pyo3::types::{{PyDict, PyList, PyBytes}};").unwrap();
    writeln!(output, "use std::collections::HashMap;").unwrap();
    writeln!(output, "use std::fs;").unwrap();
    writeln!(output, "use std::path::Path;").unwrap();
    writeln!(output).unwrap();
    writeln!(output, "use crate::{{").unwrap();
    writeln!(output, "    UniversalSchematic,").unwrap();
    writeln!(output, "    BlockState,").unwrap();
    writeln!(output, "    formats::{{litematic, schematic}},").unwrap();
    writeln!(output, "    print_utils::{{format_schematic, format_json_schematic}},").unwrap();
    writeln!(output, "}};").unwrap();
    writeln!(output).unwrap();
}

fn write_python_class(output: &mut String, class: &ApiClass) {
    let class_name = format!("Py{}", class.name);
    
    writeln!(output, "#[pyclass(name = \"{}\")]", class.name).unwrap();
    if class.copyable {
        writeln!(output, "#[derive(Clone)]").unwrap();
    }
    writeln!(output, "pub struct {} {{", class_name).unwrap();
    writeln!(output, "    pub(crate) inner: {},", get_rust_type(&class.name)).unwrap();
    writeln!(output, "}}").unwrap();
    writeln!(output).unwrap();
    
    writeln!(output, "#[pymethods]").unwrap();
    writeln!(output, "impl {} {{", class_name).unwrap();
    
    // Generate methods
    for method in &class.methods {
        write_python_method(output, method, &class.name);
    }
    
    // Generate property getters
    for property in &class.properties {
        write_python_property_getter(output, property);
    }
    
    // Generate __str__ and __repr__
    writeln!(output, "    fn __str__(&self) -> String {{").unwrap();
    if class.name == "BlockState" {
        writeln!(output, "        self.inner.to_string()").unwrap();
    } else {
        writeln!(output, "        format_schematic(&self.inner)").unwrap();
    }
    writeln!(output, "    }}").unwrap();
    writeln!(output).unwrap();
    
    writeln!(output, "    fn __repr__(&self) -> String {{").unwrap();
    if class.name == "BlockState" {
        writeln!(output, "        format!(\"<BlockState '{{}}>'>\", self.inner.to_string())").unwrap();
    } else {
        writeln!(output, "        format!(\"<{{}} '{{}}, {{}} blocks'\", \"Schematic\",").unwrap();
        writeln!(output, "                self.inner.metadata.name.as_ref().unwrap_or(&\"Unnamed\".to_string()),").unwrap();
        writeln!(output, "                self.inner.total_blocks())").unwrap();
    }
    writeln!(output, "    }}").unwrap();
    
    writeln!(output, "}}").unwrap();
    writeln!(output).unwrap();
}

fn write_python_method(output: &mut String, method: &ApiMethod, class_name: &str) {
    // Write method signature
    if method.is_constructor {
        writeln!(output, "    #[new]").unwrap();
        write!(output, "    fn new(").unwrap();
    } else if method.is_getter {
        writeln!(output, "    #[getter]").unwrap();
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
        if i > 0 || (!method.is_constructor && !method.is_static) { 
            write!(output, ", ").unwrap(); 
        }
        write!(output, "{}: {}", param.name, python_type(&param.param_type)).unwrap();
    }
    
    write!(output, ")").unwrap();
    
    // Write return type
    let return_type = python_return_type(&method.return_type, method.error_type.as_ref());
    if method.is_constructor {
        writeln!(output, " -> Self {{").unwrap();
    } else if return_type != "()" {
        writeln!(output, " -> {} {{", return_type).unwrap();
    } else {
        writeln!(output, " {{").unwrap();
    }
    
    // Write method body
    write_python_method_body(output, method, class_name);
    
    writeln!(output, "    }}").unwrap();
    writeln!(output).unwrap();
}

fn write_python_method_body(output: &mut String, method: &ApiMethod, class_name: &str) {
    if method.is_constructor {
        if class_name == "Schematic" {
            writeln!(output, "        Self {{").unwrap();
            writeln!(output, "            inner: UniversalSchematic::new({}.unwrap_or_else(|| \"Default\".to_string())),", 
                method.params.first().map(|p| &p.name).unwrap_or(&"name".to_string())).unwrap();
            writeln!(output, "        }}").unwrap();
        } else if class_name == "BlockState" {
            writeln!(output, "        Self {{").unwrap();
            writeln!(output, "            inner: BlockState::new({}),", method.params[0].name).unwrap();
            writeln!(output, "        }}").unwrap();
        }
    } else {
        match method.name.as_str() {
            "from_data" => {
                writeln!(output, "        if litematic::is_litematic({}) {{", method.params[0].name).unwrap();
                writeln!(output, "            self.inner = litematic::from_litematic({})", method.params[0].name).unwrap();
                writeln!(output, "                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;").unwrap();
                writeln!(output, "        }} else if schematic::is_schematic({}) {{", method.params[0].name).unwrap();
                writeln!(output, "            self.inner = schematic::from_schematic({})", method.params[0].name).unwrap();
                writeln!(output, "                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;").unwrap();
                writeln!(output, "        }} else {{").unwrap();
                writeln!(output, "            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(\"Unknown format\"));").unwrap();
                writeln!(output, "        }}").unwrap();
                writeln!(output, "        Ok(())").unwrap();
            },
            "from_litematic" => {
                writeln!(output, "        self.inner = litematic::from_litematic({})", method.params[0].name).unwrap();
                writeln!(output, "            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;").unwrap();
                writeln!(output, "        Ok(())").unwrap();
            },
            "to_litematic" => {
                writeln!(output, "        let bytes = litematic::to_litematic(&self.inner)").unwrap();
                writeln!(output, "            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;").unwrap();
                writeln!(output, "        Ok(PyBytes::new(py, &bytes).into())").unwrap();
            },
            "from_schematic" => {
                writeln!(output, "        self.inner = schematic::from_schematic({})", method.params[0].name).unwrap();
                writeln!(output, "            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;").unwrap();
                writeln!(output, "        Ok(())").unwrap();
            },
            "to_schematic" => {
                writeln!(output, "        let bytes = schematic::to_schematic(&self.inner)").unwrap();
                writeln!(output, "            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;").unwrap();
                writeln!(output, "        Ok(PyBytes::new(py, &bytes).into())").unwrap();
            },
            "set_block" => {
                writeln!(output, "        self.inner.set_block({}, {}, {}, BlockState::new({}.to_string()));", 
                    method.params[0].name, method.params[1].name, method.params[2].name, method.params[3].name).unwrap();
            },
            "set_block_with_properties" => {
                writeln!(output, "        let block_state = BlockState {{").unwrap();
                writeln!(output, "            name: {}.to_string(),", method.params[3].name).unwrap();
                writeln!(output, "            properties: {},", method.params[4].name).unwrap();
                writeln!(output, "        }};").unwrap();
                writeln!(output, "        self.inner.set_block({}, {}, {}, block_state);", 
                    method.params[0].name, method.params[1].name, method.params[2].name).unwrap();
            },
            "get_block" => {
                writeln!(output, "        self.inner.get_block({}, {}, {}).cloned().map(|bs| Py{} {{ inner: bs }})", 
                    method.params[0].name, method.params[1].name, method.params[2].name, class_name).unwrap();
            },
            "with_property" => {
                writeln!(output, "        let new_inner = self.inner.clone().with_property({}, {});", 
                    method.params[0].name, method.params[1].name).unwrap();
                writeln!(output, "        Self {{ inner: new_inner }}").unwrap();
            },
            _ => {
                writeln!(output, "        // TODO: Implement {}", method.name).unwrap();
                writeln!(output, "        unimplemented!()").unwrap();
            }
        }
    }
}

fn write_python_property_getter(output: &mut String, property: &ApiProperty) {
    writeln!(output, "    #[getter]").unwrap();
    write!(output, "    pub fn {}(&self) -> {}", property.name, python_type(&property.property_type)).unwrap();
    writeln!(output, " {{").unwrap();
    
    match property.name.as_str() {
        "name" => writeln!(output, "        self.inner.name.clone()").unwrap(),
        "properties" => writeln!(output, "        self.inner.properties.clone()").unwrap(),
        "dimensions" => writeln!(output, "        let (x, y, z) = self.inner.get_dimensions(); vec![x, y, z]").unwrap(),
        "block_count" => writeln!(output, "        self.inner.total_blocks()").unwrap(),
        "volume" => writeln!(output, "        self.inner.total_volume()").unwrap(),
        "region_names" => writeln!(output, "        self.inner.get_region_names()").unwrap(),
        _ => writeln!(output, "        unimplemented!()").unwrap(),
    }
    
    writeln!(output, "    }}").unwrap();
    writeln!(output).unwrap();
}

fn write_python_function(output: &mut String, function: &ApiFunction) {
    writeln!(output, "#[pyfunction]").unwrap();
    
    // Write function signature
    if function.name == "save_schematic" {
        writeln!(output, "#[pyo3(signature = (schematic, path, format = \"auto\"))]").unwrap();
    }
    
    write!(output, "fn {}(", function.name).unwrap();
    
    // Add Python context if needed for bytes operations
    let needs_py = matches!(function.return_type, ApiType::Bytes) ||
                   function.params.iter().any(|p| matches!(p.param_type, ApiType::Bytes));
    if needs_py {
        write!(output, "py: Python<'_>, ").unwrap();
    }
    
    for (i, param) in function.params.iter().enumerate() {
        if i > 0 || needs_py { write!(output, ", ").unwrap(); }
        write!(output, "{}: {}", param.name, python_type(&param.param_type)).unwrap();
    }
    
    let return_type = python_return_type(&function.return_type, function.error_type.as_ref());
    writeln!(output, ") -> {} {{", return_type).unwrap();
    
    // Write function body
    match function.name.as_str() {
        "load_schematic" => {
            writeln!(output, "    let data = fs::read({})", function.params[0].name).unwrap();
            writeln!(output, "        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;").unwrap();
            writeln!(output).unwrap();
            writeln!(output, "    let mut sch = PySchematic::new(Some(").unwrap();
            writeln!(output, "        Path::new({})", function.params[0].name).unwrap();
            writeln!(output, "            .file_stem()").unwrap();
            writeln!(output, "            .and_then(|s| s.to_str())").unwrap();
            writeln!(output, "            .unwrap_or(\"Unnamed\")").unwrap();
            writeln!(output, "            .to_owned(),").unwrap();
            writeln!(output, "    ));").unwrap();
            writeln!(output, "    sch.from_data(&data)?;").unwrap();
            writeln!(output, "    Ok(sch)").unwrap();
        },
        "save_schematic" => {
            writeln!(output, "    let py_bytes = match {} {{", function.params[2].name).unwrap();
            writeln!(output, "        \"litematic\" => {}.to_litematic(py)?,", function.params[0].name).unwrap();
            writeln!(output, "        \"schematic\" => {}.to_schematic(py)?,", function.params[0].name).unwrap();
            writeln!(output, "        \"auto\" => {{").unwrap();
            writeln!(output, "            if {}.ends_with(\".litematic\") {{", function.params[1].name).unwrap();
            writeln!(output, "                {}.to_litematic(py)?", function.params[0].name).unwrap();
            writeln!(output, "            }} else {{").unwrap();
            writeln!(output, "                {}.to_schematic(py)?", function.params[0].name).unwrap();
            writeln!(output, "            }}").unwrap();
            writeln!(output, "        }}").unwrap();
            writeln!(output, "        other => return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(").unwrap();
            writeln!(output, "            format!(\"Unknown format '{{}}'\", other)))").unwrap();
            writeln!(output, "    }};").unwrap();
            writeln!(output).unwrap();
            writeln!(output, "    let bytes_obj = py_bytes.bind(py).downcast::<PyBytes>()?;").unwrap();
            writeln!(output, "    let bytes = bytes_obj.as_bytes();").unwrap();
            writeln!(output).unwrap();
            writeln!(output, "    fs::write({}, bytes)", function.params[1].name).unwrap();
            writeln!(output, "        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;").unwrap();
            writeln!(output).unwrap();
            writeln!(output, "    Ok(())").unwrap();
        },
        "debug_schematic" => {
            writeln!(output, "    format!(\"{{}}\\n{{}}\", {}.debug_info(), format_schematic(&{}.inner))", 
                function.params[0].name, function.params[0].name).unwrap();
        },
        "debug_json_schematic" => {
            writeln!(output, "    format!(\"{{}}\\n{{}}\", {}.debug_info(), format_json_schematic(&{}.inner))", 
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

fn write_python_module(output: &mut String, api: &ApiDefinition) {
    writeln!(output, "#[pymodule]").unwrap();
    writeln!(output, "fn {}(m: &Bound<'_, PyModule>) -> PyResult<()> {{", api.name).unwrap();
    
    for class in &api.classes {
        writeln!(output, "    m.add_class::<Py{}>()?;", class.name).unwrap();
    }
    
    for function in &api.functions {
        writeln!(output, "    m.add_function(wrap_pyfunction!({}, m)?)?;", function.name).unwrap();
    }
    
    writeln!(output, "    Ok(())").unwrap();
    writeln!(output, "}}").unwrap();
}

fn write_python_class_stub(output: &mut String, class: &ApiClass) {
    writeln!(output, "class {}:", class.name).unwrap();
    writeln!(output, "    \"\"\"{}\"\"\"", class.docs).unwrap();
    writeln!(output).unwrap();
    
    // Constructor
    for method in &class.methods {
        if method.is_constructor {
            write!(output, "    def __init__(self").unwrap();
            for param in &method.params {
                write!(output, ", {}: {}", param.name, python_stub_type(&param.param_type)).unwrap();
                if param.optional {
                    if let Some(default) = &param.default {
                        write!(output, " = {}", default.replace("\"", "")).unwrap();
                    } else {
                        write!(output, " = None").unwrap();
                    }
                }
            }
            writeln!(output, ") -> None:").unwrap();
            writeln!(output, "        \"\"\"{}\"\"\"", method.docs).unwrap();
            writeln!(output, "        ...").unwrap();
            writeln!(output).unwrap();
        }
    }
    
    // Properties
    for property in &class.properties {
        writeln!(output, "    @property").unwrap();
        writeln!(output, "    def {}(self) -> {}: ", property.name, python_stub_type(&property.property_type)).unwrap();
        writeln!(output, "        \"\"\"{}\"\"\"", property.docs).unwrap();
        writeln!(output, "        ...").unwrap();
        writeln!(output).unwrap();
    }
    
    // Methods
    for method in &class.methods {
        if !method.is_constructor && !method.is_getter {
            write!(output, "    def {}(self", method.name).unwrap();
            for param in &method.params {
                write!(output, ", {}: {}", param.name, python_stub_type(&param.param_type)).unwrap();
                if param.optional {
                    if let Some(default) = &param.default {
                        write!(output, " = {}", default.replace("\"", "")).unwrap();
                    } else {
                        write!(output, " = None").unwrap();
                    }
                }
            }
            let return_type = python_stub_return_type(&method.return_type, method.error_type.as_ref());
            writeln!(output, ") -> {}:", return_type).unwrap();
            writeln!(output, "        \"\"\"{}\"\"\"", method.docs).unwrap();
            writeln!(output, "        ...").unwrap();
            writeln!(output).unwrap();
        }
    }
    
    writeln!(output).unwrap();
}

fn write_python_function_stub(output: &mut String, function: &ApiFunction) {
    write!(output, "def {}(", function.name).unwrap();
    
    for (i, param) in function.params.iter().enumerate() {
        if i > 0 { write!(output, ", ").unwrap(); }
        write!(output, "{}: {}", param.name, python_stub_type(&param.param_type)).unwrap();
        if param.optional {
            if let Some(default) = &param.default {
                write!(output, " = {}", default.replace("\"", "")).unwrap();
            } else {
                write!(output, " = None").unwrap();
            }
        }
    }
    
    let return_type = python_stub_return_type(&function.return_type, function.error_type.as_ref());
    writeln!(output, ") -> {}:", return_type).unwrap();
    writeln!(output, "    \"\"\"{}\"\"\"", function.docs).unwrap();
    writeln!(output, "    ...").unwrap();
    writeln!(output).unwrap();
}

fn python_type(api_type: &ApiType) -> String {
    match api_type {
        ApiType::String => "String".to_string(),
        ApiType::I32 => "i32".to_string(),
        ApiType::F32 => "f32".to_string(),
        ApiType::Bool => "bool".to_string(),
        ApiType::Bytes => "&[u8]".to_string(),
        ApiType::Vec(inner) => format!("Vec<{}>", python_type(inner)),
        ApiType::HashMap(k, v) => format!("HashMap<{}, {}>", python_type(k), python_type(v)),
        ApiType::Option(inner) => format!("Option<{}>", python_type(inner)),
        ApiType::Result(_, _) => "PyResult<()>".to_string(), // Simplified for method params
        ApiType::Custom(name) => format!("&Py{}", name),
        ApiType::Void => "()".to_string(),
    }
}

fn python_return_type(api_type: &ApiType, error_type: Option<&String>) -> String {
    match api_type {
        ApiType::Result(ok, _) => {
            if error_type.is_some() {
                format!("PyResult<{}>", python_type(ok))
            } else {
                python_type(ok)
            }
        },
        _ => {
            if error_type.is_some() {
                format!("PyResult<{}>", python_type(api_type))
            } else {
                python_type(api_type)
            }
        }
    }
}

fn python_stub_type(api_type: &ApiType) -> String {
    match api_type {
        ApiType::String => "str".to_string(),
        ApiType::I32 => "int".to_string(),
        ApiType::F32 => "float".to_string(),
        ApiType::Bool => "bool".to_string(),
        ApiType::Bytes => "bytes".to_string(),
        ApiType::Vec(inner) => format!("List[{}]", python_stub_type(inner)),
        ApiType::HashMap(k, v) => format!("Dict[{}, {}]", python_stub_type(k), python_stub_type(v)),
        ApiType::Option(inner) => format!("Optional[{}]", python_stub_type(inner)),
        ApiType::Result(ok, _) => python_stub_type(ok), // Just return the success type for stubs
        ApiType::Custom(name) => name.clone(),
        ApiType::Void => "None".to_string(),
    }
}

fn python_stub_return_type(api_type: &ApiType, _error_type: Option<&String>) -> String {
    python_stub_type(api_type)
}

fn get_rust_type(type_name: &str) -> String {
    match type_name {
        "Schematic" => "UniversalSchematic".to_string(),
        "BlockState" => "BlockState".to_string(),
        _ => type_name.to_string(),
    }
}
