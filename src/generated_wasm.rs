// Auto-generated WASM bindings for nucleation.
// This file is generated from api_definition.rs - DO NOT EDIT MANUALLY!

use wasm_bindgen::prelude::*;
use js_sys::{self, Array, Object, Reflect};
use web_sys::console;
use crate::{
    UniversalSchematic,
    BlockState,
    formats::{litematic, schematic},
    print_utils::{format_schematic, format_json_schematic},
};
use std::collections::HashMap;

#[wasm_bindgen(start)]
pub fn start() {
    console::log_1(&"Initializing nucleation".into());
}

/// Represents a Minecraft block with its properties
#[wasm_bindgen]
pub struct BlockStateWrapper { inner: BlockState }

#[wasm_bindgen]
impl BlockStateWrapper {
    /// Create a new BlockState with the given name
    #[wasm_bindgen(constructor)]
    pub fn new(name: String) -> Self {
        Self { inner: BlockState::new(name) }
    }

    /// Create a new BlockState with an additional property
    pub fn with_property(&mut self, key: String, value: String) -> BlockStateWrapper {
        Self { inner: self.inner.clone().with_property(key.to_string(), value.to_string()) }
    }

    /// The block's resource name (e.g., 'minecraft:stone')
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.inner.name.clone()
    }

    /// The block's state properties
    #[wasm_bindgen(getter)]
    pub fn properties(&self) -> JsValue {
        let obj = Object::new();
        for (key, value) in &self.inner.properties {
            Reflect::set(&obj, &JsValue::from_str(key), &JsValue::from_str(value)).unwrap();
        }
        obj.into()
    }

}

/// A Minecraft schematic containing blocks and metadata
#[wasm_bindgen]
pub struct SchematicWrapper { inner: UniversalSchematic }

#[wasm_bindgen]
impl SchematicWrapper {
    /// Create a new empty schematic
    #[wasm_bindgen(constructor)]
    pub fn new(name: Option<String>) -> Self {
        let name = name.unwrap_or_else(|| "Default".to_string());
        Self { inner: UniversalSchematic::new(name) }
    }

    /// Load schematic from byte data, auto-detecting format
    pub fn from_data(&mut self, data: &[u8]) -> Result<(), String> {
        if crate::formats::litematic::is_litematic(data) {
            self.inner = crate::formats::litematic::from_litematic(data)
                .map_err(|e| JsValue::from_str(&format!("Litematic error: {}", e)))?;
        } else if crate::formats::schematic::is_schematic(data) {
            self.inner = crate::formats::schematic::from_schematic(data)
                .map_err(|e| JsValue::from_str(&format!("Schematic error: {}", e)))?;
        } else {
            return Err(JsValue::from_str("Unknown format"));
        }
        Ok(())
    }

    /// Load schematic from Litematic format data
    pub fn from_litematic(&mut self, data: &[u8]) -> Result<(), String> {
        self.inner = crate::formats::litematic::from_litematic(data)
            .map_err(|e| JsValue::from_str(&format!("Litematic error: {}", e)))?;
        Ok(())
    }

    /// Convert schematic to Litematic format
    pub fn to_litematic(&mut self) -> Result<&[u8], String> {
        crate::formats::litematic::to_litematic(&self.inner)
            .map_err(|e| JsValue::from_str(&format!("Litematic error: {}", e)))
    }

    /// Load schematic from classic .schematic format data
    pub fn from_schematic(&mut self, data: &[u8]) -> Result<(), String> {
        self.inner = crate::formats::schematic::from_schematic(data)
            .map_err(|e| JsValue::from_str(&format!("Schematic error: {}", e)))?;
        Ok(())
    }

    /// Convert schematic to classic .schematic format
    pub fn to_schematic(&mut self) -> Result<&[u8], String> {
        crate::formats::schematic::to_schematic(&self.inner)
            .map_err(|e| JsValue::from_str(&format!("Schematic error: {}", e)))
    }

    /// Set a block at the specified position
    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block_name: String) {
        self.inner.set_block(x, y, z, BlockState::new(block_name.to_string()));
    }

    /// Set a block at the specified position with properties
    pub fn set_block_with_properties(&mut self, x: i32, y: i32, z: i32, block_name: String, properties: JsValue) {
        // Convert JsValue properties to HashMap
        let mut props = HashMap::new();
        // Implementation would convert JS object to HashMap
        let block_state = BlockState { name: block_name.to_string(), properties: props };
        self.inner.set_block(x, y, z, block_state);
    }

    /// Get block at the specified position
    pub fn get_block(&mut self, x: i32, y: i32, z: i32) -> Option<BlockStateWrapper> {
        self.inner.get_block(x, y, z).cloned().map(|bs| BlockStateWrapper { inner: bs })
    }

    /// The schematic's dimensions [width, height, length]
    #[wasm_bindgen(getter)]
    pub fn dimensions(&self) -> Vec<i32> {
        let (x, y, z) = self.inner.get_dimensions();
        vec![x, y, z]
    }

    /// Total number of non-air blocks
    #[wasm_bindgen(getter)]
    pub fn block_count(&self) -> i32 {
        self.inner.total_blocks()
    }

    /// Total volume of the schematic
    #[wasm_bindgen(getter)]
    pub fn volume(&self) -> i32 {
        self.inner.total_volume()
    }

    /// Names of all regions in the schematic
    #[wasm_bindgen(getter)]
    pub fn region_names(&self) -> Vec<String> {
        self.inner.get_region_names()
    }

}

/// Load a schematic from file path
#[wasm_bindgen]
pub fn load_schematic(path: String) -> Result<SchematicWrapper, String> {
    // TODO: Implement load_schematic
    Err(JsValue::from_str("Not implemented"))
}

/// Save a schematic to file path
#[wasm_bindgen]
pub fn save_schematic(schematic: SchematicWrapper, path: String, format: String) -> Result<(), String> {
    // TODO: Implement save_schematic
    Err(JsValue::from_str("Not implemented"))
}

/// Get debug information for a schematic
#[wasm_bindgen]
pub fn debug_schematic(schematic: SchematicWrapper) -> String {
    format!("{}\n{}", schematic.debug_info(), crate::print_utils::format_schematic(&schematic.inner))
}

/// Get debug information for a schematic in JSON format
#[wasm_bindgen]
pub fn debug_json_schematic(schematic: SchematicWrapper) -> String {
    format!("{}\n{}", schematic.debug_info(), crate::print_utils::format_json_schematic(&schematic.inner))
}

