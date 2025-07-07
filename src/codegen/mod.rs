//! Code generation system for maintaining binding parity
//! 
//! This module provides generators for all binding targets from a single
//! source of truth API definition.

pub mod wasm_generator;
pub mod python_generator;
pub mod ffi_generator;

use crate::api_definition::{ApiDefinition, nucleation_api};
use std::fs;
use std::path::Path;

/// Generate all bindings and write them to the appropriate files
pub fn generate_all_bindings() -> Result<(), Box<dyn std::error::Error>> {
    let api = nucleation_api();
    
    // Generate WASM bindings
    let wasm_code = wasm_generator::generate_wasm_bindings(&api);
    fs::write("src/generated_wasm.rs", wasm_code)?;
    println!("✅ Generated WASM bindings: src/generated_wasm.rs");
    
    // Generate Python bindings
    let python_code = python_generator::generate_python_bindings(&api);
    fs::write("src/generated_python.rs", python_code)?;
    println!("✅ Generated Python bindings: src/generated_python.rs");
    
    // Generate Python stubs
    let python_stubs = python_generator::generate_python_stubs(&api);
    fs::create_dir_all("python-stubs")?;
    fs::write("python-stubs/nucleation.pyi", python_stubs)?;
    println!("✅ Generated Python stubs: python-stubs/nucleation.pyi");
    
    // Generate FFI bindings
    let ffi_code = ffi_generator::generate_ffi_bindings(&api);
    fs::write("src/generated_ffi.rs", ffi_code)?;
    println!("✅ Generated FFI bindings: src/generated_ffi.rs");
    
    // Generate FFI header
    let ffi_header = ffi_generator::generate_c_header(&api);
    fs::write("include/nucleation.h", ffi_header)?;
    println!("✅ Generated C header: include/nucleation.h");
    
    println!("\n🎉 All bindings generated successfully!");
    println!("📝 Remember to update your imports to use the generated files.");
    
    Ok(())
}

/// Check if all generated files are up to date
pub fn check_bindings_parity() -> Result<bool, Box<dyn std::error::Error>> {
    let api = nucleation_api();
    
    // Check WASM
    let expected_wasm = wasm_generator::generate_wasm_bindings(&api);
    let current_wasm = fs::read_to_string("src/generated_wasm.rs").unwrap_or_default();
    
    // Check Python
    let expected_python = python_generator::generate_python_bindings(&api);
    let current_python = fs::read_to_string("src/generated_python.rs").unwrap_or_default();
    
    // Check FFI
    let expected_ffi = ffi_generator::generate_ffi_bindings(&api);
    let current_ffi = fs::read_to_string("src/generated_ffi.rs").unwrap_or_default();
    
    let wasm_match = expected_wasm.trim() == current_wasm.trim();
    let python_match = expected_python.trim() == current_python.trim();
    let ffi_match = expected_ffi.trim() == current_ffi.trim();
    
    if !wasm_match {
        println!("❌ WASM bindings are out of date");
    }
    if !python_match {
        println!("❌ Python bindings are out of date");
    }
    if !ffi_match {
        println!("❌ FFI bindings are out of date");
    }
    
    let all_match = wasm_match && python_match && ffi_match;
    
    if all_match {
        println!("✅ All bindings are up to date");
    } else {
        println!("⚠️  Some bindings are out of date. Run `cargo run --bin generate-bindings` to update them.");
    }
    
    Ok(all_match)
}

/// Generate a comparison report showing API differences
pub fn generate_api_report() -> Result<(), Box<dyn std::error::Error>> {
    let api = nucleation_api();
    
    let mut report = String::new();
    report.push_str("# Nucleation API Report\n\n");
    report.push_str(&format!("**Library:** {} v{}\n", api.name, api.version));
    report.push_str(&format!("**Description:** {}\n\n", api.docs));
    
    report.push_str("## Classes\n\n");
    for class in &api.classes {
        report.push_str(&format!("### {}\n", class.name));
        report.push_str(&format!("{}\n\n", class.docs));
        
        report.push_str("**Properties:**\n");
        for prop in &class.properties {
            report.push_str(&format!("- `{}`: {} - {}\n", prop.name, format_type(&prop.property_type), prop.docs));
        }
        report.push_str("\n");
        
        report.push_str("**Methods:**\n");
        for method in &class.methods {
            let params: Vec<String> = method.params.iter()
                .map(|p| format!("{}: {}", p.name, format_type(&p.param_type)))
                .collect();
            report.push_str(&format!("- `{}({})` -> {} - {}\n", 
                method.name, 
                params.join(", "), 
                format_type(&method.return_type),
                method.docs
            ));
        }
        report.push_str("\n");
    }
    
    report.push_str("## Functions\n\n");
    for function in &api.functions {
        let params: Vec<String> = function.params.iter()
            .map(|p| format!("{}: {}", p.name, format_type(&p.param_type)))
            .collect();
        report.push_str(&format!("- `{}({})` -> {} - {}\n", 
            function.name, 
            params.join(", "), 
            format_type(&function.return_type),
            function.docs
        ));
    }
    
    fs::write("API_REPORT.md", report)?;
    println!("📊 Generated API report: API_REPORT.md");
    
    Ok(())
}

fn format_type(api_type: &crate::api_definition::ApiType) -> String {
    match api_type {
        crate::api_definition::ApiType::String => "String".to_string(),
        crate::api_definition::ApiType::I32 => "i32".to_string(),
        crate::api_definition::ApiType::F32 => "f32".to_string(),
        crate::api_definition::ApiType::Bool => "bool".to_string(),
        crate::api_definition::ApiType::Bytes => "bytes".to_string(),
        crate::api_definition::ApiType::Vec(inner) => format!("Vec<{}>", format_type(inner)),
        crate::api_definition::ApiType::HashMap(k, v) => format!("HashMap<{}, {}>", format_type(k), format_type(v)),
        crate::api_definition::ApiType::Option(inner) => format!("Option<{}>", format_type(inner)),
        crate::api_definition::ApiType::Result(ok, err) => format!("Result<{}, {}>", format_type(ok), format_type(err)),
        crate::api_definition::ApiType::Custom(name) => name.clone(),
        crate::api_definition::ApiType::Void => "()".to_string(),
    }
}
