//! Binding validation tests
//! 
//! This module contains tests that validate the functionality of generated bindings
//! across different targets (WASM, Python, FFI) to ensure feature parity.

pub mod shared_test_cases;
pub mod wasm_tests;
pub mod python_tests;
pub mod test_utils;

use std::path::PathBuf;

/// Common test data used across all binding tests
pub struct TestData {
    pub sample_litematic: PathBuf,
    pub sample_schematic: PathBuf,
    pub small_test_file: PathBuf,
}

impl TestData {
    pub fn new() -> Self {
        let samples_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/samples");
        
        Self {
            sample_litematic: samples_dir.join("sample.litematic"),
            sample_schematic: samples_dir.join("sample.schem"),
            small_test_file: samples_dir.join("1x1.litematic"),
        }
    }
    
    pub fn get_sample_data(&self, file: &PathBuf) -> Vec<u8> {
        std::fs::read(file).expect("Failed to read test file")
    }
}

/// Test case definition for cross-platform validation
pub struct CrossPlatformTestCase {
    pub name: String,
    pub description: String,
    pub setup: fn() -> TestData,
    pub validate_schematic: fn(&TestData, schematic_name: &str) -> bool,
    pub validate_blockstate: fn(block_name: &str, properties: std::collections::HashMap<String, String>) -> bool,
    pub expected_block_count: usize,
    pub expected_dimensions: (i32, i32, i32),
}

/// Generate test cases that should work identically across all bindings
pub fn get_cross_platform_test_cases() -> Vec<CrossPlatformTestCase> {
    vec![
        CrossPlatformTestCase {
            name: "basic_schematic_load".to_string(),
            description: "Load a basic schematic and verify properties".to_string(),
            setup: TestData::new,
            validate_schematic: |data, name| {
                !name.is_empty() && name != "Unnamed"
            },
            validate_blockstate: |name, _| !name.is_empty(),
            expected_block_count: 1, // Will be overridden per test
            expected_dimensions: (1, 1, 1), // Will be overridden per test
        },
        CrossPlatformTestCase {
            name: "litematic_to_schematic_conversion".to_string(),
            description: "Convert between litematic and schematic formats".to_string(),
            setup: TestData::new,
            validate_schematic: |_, _| true, // Just ensure no errors
            validate_blockstate: |_, _| true,
            expected_block_count: 0, // Variable
            expected_dimensions: (0, 0, 0), // Variable
        },
        CrossPlatformTestCase {
            name: "block_manipulation".to_string(),
            description: "Set and get blocks with properties".to_string(),
            setup: TestData::new,
            validate_schematic: |_, _| true,
            validate_blockstate: |name, props| {
                name == "minecraft:stone" && props.get("variant").map_or(false, |v| v == "smooth")
            },
            expected_block_count: 1,
            expected_dimensions: (1, 1, 1),
        },
    ]
}
