// src/wasm.rs

use wasm_bindgen::prelude::*;
use js_sys::{self, Array, Object, Reflect};
use web_sys::console;
use crate::{
    UniversalSchematic,
    BlockState,
    formats::{litematic, schematic},
    print_utils::{format_schematic as print_schematic, format_json_schematic as print_json_schematic},
    block_position::BlockPosition,
    mchprs_world::MchprsWorld,
};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use mchprs_blocks::BlockPos;
use crate::bounding_box::BoundingBox;
use crate::chunk_iterator::ChunksIterator;
use crate::mchprs_world::generate_truth_table;
use crate::universal_schematic::ChunkLoadingStrategy;

#[wasm_bindgen(start)]
pub fn start() {
    console::log_1(&"Initializing schematic utilities".into());
}

// Wrapper structs
#[wasm_bindgen]
pub struct SchematicWrapper(pub(crate) UniversalSchematic);

#[wasm_bindgen]
pub struct MchprsWorldWrapper {
    world: MchprsWorld,
}

#[wasm_bindgen]
pub struct BlockStateWrapper(pub(crate) BlockState);

#[wasm_bindgen(js_name = JsChunksIterator)]
pub struct JsChunksIterator {
    inner: ChunksIterator,
}

#[wasm_bindgen]
impl JsChunksIterator {
    #[wasm_bindgen(constructor)]
    pub fn new(schematic_wrapper: &SchematicWrapper, chunk_width: i32, chunk_height: i32, chunk_length: i32) -> Self {
        // Clone the schematic into an Rc to ensure it lives as long as the iterator
        let schematic = Rc::new(schematic_wrapper.0.clone());

        JsChunksIterator {
            inner: ChunksIterator::new(schematic, chunk_width, chunk_height, chunk_length),
        }
    }

    #[wasm_bindgen(js_name = next)]
    pub fn next(&mut self) -> JsValue {
        // Get the next chunk
        if let Some((chunk_x, chunk_y, chunk_z, blocks)) = self.inner.next_chunk() {
            // Create chunk object
            let chunk_obj = js_sys::Object::new();
            js_sys::Reflect::set(&chunk_obj, &"chunk_x".into(), &chunk_x.into()).unwrap();
            js_sys::Reflect::set(&chunk_obj, &"chunk_y".into(), &chunk_y.into()).unwrap();
            js_sys::Reflect::set(&chunk_obj, &"chunk_z".into(), &chunk_z.into()).unwrap();

            // Create blocks array
            let blocks_array = js_sys::Array::new();

            for (pos, block) in blocks {
                let block_obj = js_sys::Object::new();
                js_sys::Reflect::set(&block_obj, &"x".into(), &pos.x.into()).unwrap();
                js_sys::Reflect::set(&block_obj, &"y".into(), &pos.y.into()).unwrap();
                js_sys::Reflect::set(&block_obj, &"z".into(), &pos.z.into()).unwrap();
                js_sys::Reflect::set(&block_obj, &"name".into(), &JsValue::from_str(&block.name)).unwrap();

                // Add properties
                let properties = js_sys::Object::new();
                for (key, value) in &block.properties {
                    js_sys::Reflect::set(&properties, &JsValue::from_str(key), &JsValue::from_str(value)).unwrap();
                }
                js_sys::Reflect::set(&block_obj, &"properties".into(), &properties).unwrap();

                blocks_array.push(&block_obj);
            }

            js_sys::Reflect::set(&chunk_obj, &"blocks".into(), &blocks_array).unwrap();

            // Create iterator result object {value, done}
            let result = js_sys::Object::new();
            js_sys::Reflect::set(&result, &"value".into(), &chunk_obj).unwrap();
            js_sys::Reflect::set(&result, &"done".into(), &JsValue::from_bool(false)).unwrap();

            result.into()
        } else {
            // Return {done: true} when iteration is complete
            let result = js_sys::Object::new();
            js_sys::Reflect::set(&result, &"done".into(), &JsValue::from_bool(true)).unwrap();
            result.into()
        }
    }

    #[wasm_bindgen(js_name = countNonEmptyChunks)]
    pub fn count_non_empty_chunks(&self) -> i32 {
        // Create a clone of the iterator to avoid consuming the original
        let schematic = self.inner.schematic.clone();
        let bbox = schematic.get_bounding_box();
        let chunk_width = self.inner.chunk_width;
        let chunk_height = self.inner.chunk_height;
        let chunk_length = self.inner.chunk_length;

        // Calculate min and max chunk coordinates
        let min_chunk_x = if bbox.min.0 < 0 {
            (bbox.min.0 - chunk_width + 1) / chunk_width
        } else {
            bbox.min.0 / chunk_width
        };

        let min_chunk_y = if bbox.min.1 < 0 {
            (bbox.min.1 - chunk_height + 1) / chunk_height
        } else {
            bbox.min.1 / chunk_height
        };

        let min_chunk_z = if bbox.min.2 < 0 {
            (bbox.min.2 - chunk_length + 1) / chunk_length
        } else {
            bbox.min.2 / chunk_length
        };

        let max_chunk_x = (bbox.max.0 + chunk_width - 1) / chunk_width;
        let max_chunk_y = (bbox.max.1 + chunk_height - 1) / chunk_height;
        let max_chunk_z = (bbox.max.2 + chunk_length - 1) / chunk_length;

        let mut count = 0;

        // Iterate through all possible chunks
        for chunk_x in min_chunk_x..=max_chunk_x {
            for chunk_y in min_chunk_y..=max_chunk_y {
                for chunk_z in min_chunk_z..=max_chunk_z {
                    // Calculate chunk bounds
                    let chunk_min_x = chunk_x * chunk_width;
                    let chunk_min_y = chunk_y * chunk_height;
                    let chunk_min_z = chunk_z * chunk_length;

                    let chunk_max_x = chunk_min_x + chunk_width - 1;
                    let chunk_max_y = chunk_min_y + chunk_height - 1;
                    let chunk_max_z = chunk_min_z + chunk_length - 1;

                    // Check if this chunk intersects with the bounding box
                    if chunk_min_x > bbox.max.0 || chunk_max_x < bbox.min.0 ||
                        chunk_min_y > bbox.max.1 || chunk_max_y < bbox.min.1 ||
                        chunk_min_z > bbox.max.2 || chunk_max_z < bbox.min.2 {
                        continue;
                    }

                    // Define chunk bounds clamped to the schematic bounding box
                    let min_x = std::cmp::max(chunk_min_x, bbox.min.0);
                    let min_y = std::cmp::max(chunk_min_y, bbox.min.1);
                    let min_z = std::cmp::max(chunk_min_z, bbox.min.2);

                    let max_x = std::cmp::min(chunk_max_x, bbox.max.0);
                    let max_y = std::cmp::min(chunk_max_y, bbox.max.1);
                    let max_z = std::cmp::min(chunk_max_z, bbox.max.2);

                    // Check if chunk has any non-air blocks
                    let mut has_blocks = false;
                    'outer: for x in min_x..=max_x {
                        for y in min_y..=max_y {
                            for z in min_z..=max_z {
                                if let Some(block) = schematic.get_block(x, y, z) {
                                    // Skip air blocks
                                    if !block.name.contains("air") {
                                        has_blocks = true;
                                        break 'outer;
                                    }
                                }
                            }
                        }
                    }

                    if has_blocks {
                        count += 1;
                    }
                }
            }
        }

        count
    }
}
// All your existing WASM implementations go here...
#[wasm_bindgen]
impl SchematicWrapper {

    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        SchematicWrapper(UniversalSchematic::new("Default".to_string()))
    }

    pub fn create_simulation_world(&self) -> MchprsWorldWrapper {
        MchprsWorldWrapper::new(self).unwrap()
    }


    pub fn from_data(&mut self, data: &[u8]) -> Result<(), JsValue> {
        if litematic::is_litematic(data) {
            console::log_1(&"Parsing litematic data".into());
            self.from_litematic(data)
        } else if schematic::is_schematic(data) {
            console::log_1(&"Parsing schematic data".into());
            self.from_schematic(data)
        } else {
            Err(JsValue::from_str("Unknown or unsupported schematic format"))
        }
    }

    pub fn from_litematic(&mut self, data: &[u8]) -> Result<(), JsValue> {
        self.0 = litematic::from_litematic(data)
            .map_err(|e| JsValue::from_str(&format!("Litematic parsing error: {}", e)))?;
        Ok(())
    }

    pub fn to_litematic(&self) -> Result<Vec<u8>, JsValue> {
        litematic::to_litematic(&self.0)
            .map_err(|e| JsValue::from_str(&format!("Litematic conversion error: {}", e)))
    }

    pub fn from_schematic(&mut self, data: &[u8]) -> Result<(), JsValue> {
        self.0 = schematic::from_schematic(data)
            .map_err(|e| JsValue::from_str(&format!("Schematic parsing error: {}", e)))?;
        Ok(())
    }

    pub fn to_schematic(&self) -> Result<Vec<u8>, JsValue> {
        schematic::to_schematic(&self.0)
            .map_err(|e| JsValue::from_str(&format!("Schematic conversion error: {}", e)))
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block_name: &str) {
        self.0.set_block(x, y, z, BlockState::new(block_name.to_string()));
    }

    pub fn set_block_from_string(&mut self, x: i32, y: i32, z: i32, block_string: &str) -> Result<(), JsValue> {
        self.0.set_block_from_string(x, y, z, block_string)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse block string: {}", e)))?;
        Ok(())
    }

    pub fn copy_region(
        &mut self,
        from_schematic: &SchematicWrapper,
        min_x: i32,
        min_y: i32,
        min_z: i32,
        max_x: i32,
        max_y: i32,
        max_z: i32,
        target_x: i32,
        target_y: i32,
        target_z: i32,
        excluded_blocks: &JsValue,
    ) -> Result<(), JsValue> {
        let bounds = BoundingBox::new(
            (min_x, min_y, min_z),
            (max_x, max_y, max_z)
        );

        let excluded_blocks = if !excluded_blocks.is_undefined() && !excluded_blocks.is_null() {
            let js_array: Array = excluded_blocks.clone().dyn_into().map_err(|_| {
                JsValue::from_str("Excluded blocks should be an array")
            })?;
            let mut rust_vec: Vec<BlockState> = Vec::new();
            for i in 0..js_array.length() {
                let block_string = match js_array.get(i).as_string() {
                    Some(name) => name,
                    None => return Err(JsValue::from_str("Excluded blocks should be strings"))
                };
                let (block_state, _) = UniversalSchematic::parse_block_string(&block_string)
                    .map_err(|e| JsValue::from_str(&format!("Invalid block state: {}", e)))?;
                rust_vec.push(block_state);
            }

            rust_vec
        } else {
            Vec::new()  // Return empty vec instead of None
        };

        self.0.copy_region(
            &from_schematic.0,
            &bounds,
            (target_x, target_y, target_z),
            &excluded_blocks  // Now we can pass a direct reference to the Vec
        ).map_err(|e| JsValue::from_str(&format!("Failed to copy region: {}", e)))
    }



    pub fn set_block_with_properties(
        &mut self,
        x: i32,
        y: i32,
        z: i32,
        block_name: &str,
        properties: &JsValue,
    ) -> Result<(), JsValue> {
        // Convert JsValue to HashMap<String, String>
        let mut props = HashMap::new();

        if !properties.is_undefined() && !properties.is_null() {
            let obj: Object = properties.clone().dyn_into().map_err(|_| {
                JsValue::from_str("Properties should be an object")
            })?;

            let keys = js_sys::Object::keys(&obj);
            for i in 0..keys.length() {
                let key = keys.get(i);
                let key_str = key.as_string().ok_or_else(|| {
                    JsValue::from_str("Property keys should be strings")
                })?;

                let value = Reflect::get(&obj, &key).map_err(|_| {
                    JsValue::from_str("Error getting property value")
                })?;

                let value_str = value.as_string().ok_or_else(|| {
                    JsValue::from_str("Property values should be strings")
                })?;

                props.insert(key_str, value_str);
            }
        }

        let block_state = BlockState {
            name: Arc::from(block_name),
            properties: props.into_iter()
                .map(|(k, v)| (Arc::from(k.as_str()), Arc::from(v.as_str())))
                .collect(),
        };

        // Set the block in the schematic
        self.0.set_block(x, y, z, block_state);

        Ok(())
    }


    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Option<String> {
        self.0.get_block(x, y, z).map(|block_state| block_state.name.to_string())
    }

    pub fn get_block_with_properties(&self, x: i32, y: i32, z: i32) -> Option<BlockStateWrapper> {
        self.0.get_block(x, y, z).cloned().map(BlockStateWrapper)
    }

    pub fn get_block_entity(&self, x: i32, y: i32, z: i32) -> JsValue {
        let block_position = BlockPosition { x, y, z };
        if let Some(block_entity) = self.0.get_block_entity(block_position) {
            if block_entity.id.contains("chest") {
                let obj = Object::new();
                Reflect::set(&obj, &"id".into(), &JsValue::from_str(&block_entity.id)).unwrap();

                let position = Array::new();
                position.push(&JsValue::from(block_entity.position.0));
                position.push(&JsValue::from(block_entity.position.1));
                position.push(&JsValue::from(block_entity.position.2));
                Reflect::set(&obj, &"position".into(), &position).unwrap();

                // Use the new to_js_value method
                Reflect::set(&obj, &"nbt".into(), &block_entity.nbt.to_js_value()).unwrap();

                obj.into()
            } else {
                JsValue::NULL
            }
        } else {
            JsValue::NULL
        }
    }

    pub fn get_all_block_entities(&self) -> JsValue {
        let block_entities = self.0.get_block_entities_as_list();
        let js_block_entities = Array::new();
        for block_entity in block_entities {
            let obj = Object::new();
            Reflect::set(&obj, &"id".into(), &JsValue::from_str(&block_entity.id)).unwrap();

            let position = Array::new();
            position.push(&JsValue::from(block_entity.position.0));
            position.push(&JsValue::from(block_entity.position.1));
            position.push(&JsValue::from(block_entity.position.2));
            Reflect::set(&obj, &"position".into(), &position).unwrap();

            // Use the new to_js_value method
            Reflect::set(&obj, &"nbt".into(), &block_entity.nbt.to_js_value()).unwrap();

            js_block_entities.push(&obj);
        }
        js_block_entities.into()
    }


    pub fn print_schematic(&self) -> String {
        print_schematic(&self.0)
    }

    pub fn debug_info(&self) -> String {
        format!("Schematic name: {}, Regions: {}",
                self.0.metadata.name.as_ref().unwrap_or(&"Unnamed".to_string()),
                self.0.regions.len()
        )
    }


    // Add these methods back
    pub fn get_dimensions(&self) -> Vec<i32> {
        let (x, y, z) = self.0.get_dimensions();
        vec![x, y, z]
    }

    pub fn get_block_count(&self) -> i32 {
        self.0.total_blocks()
    }

    pub fn get_volume(&self) -> i32 {
        self.0.total_volume()
    }

    pub fn get_region_names(&self) -> Vec<String> {
        self.0.get_region_names()
    }

    pub fn blocks(&self) -> Array {
        self.0.iter_blocks()
            .map(|(pos, block)| {
                let obj = js_sys::Object::new();
                js_sys::Reflect::set(&obj, &"x".into(), &pos.x.into()).unwrap();
                js_sys::Reflect::set(&obj, &"y".into(), &pos.y.into()).unwrap();
                js_sys::Reflect::set(&obj, &"z".into(), &pos.z.into()).unwrap();
                js_sys::Reflect::set(&obj, &"name".into(), &JsValue::from_str(&block.name)).unwrap();
                let properties = js_sys::Object::new();
                for (key, value) in &block.properties {
                    js_sys::Reflect::set(&properties, &JsValue::from_str(key), &JsValue::from_str(value)).unwrap();
                }
                js_sys::Reflect::set(&obj, &"properties".into(), &properties).unwrap();
                obj
            })
            .collect::<Array>()
    }

    #[wasm_bindgen]
    pub fn chunks(&self, chunk_width: i32, chunk_height: i32, chunk_length: i32) -> JsValue {
        // 1. Create the iterator instance
        let iterator = JsChunksIterator::new(self, chunk_width, chunk_height, chunk_length);

        // 2. Get the count of non-empty chunks
        let count = iterator.count_non_empty_chunks();

        // 3. Create a new iterator for actual iteration
        let iterator_for_js = JsChunksIterator::new(self, chunk_width, chunk_height, chunk_length);

        // 4. Create the JS iterable object
        let js_iterable = js_sys::Object::new();

        // 5. Store the iterator
        js_sys::Reflect::set(
            &js_iterable,
            &JsValue::from_str("_iterator"),
            &iterator_for_js.into(),
        ).unwrap();

        // 6. Make the object iterable
        let symbol_iterator_fn = js_sys::Function::new_no_args(r#"return this._iterator;"#);
        js_sys::Reflect::set(
            &js_iterable,
            &js_sys::Symbol::iterator().into(),
            &symbol_iterator_fn.into(),
        ).unwrap();

        // 7. Set the exact length property
        js_sys::Reflect::set(
            &js_iterable,
            &JsValue::from_str("length"),
            &JsValue::from(count),
        ).unwrap();

        // 8. Add the toArray helper
        let to_array_fn = js_sys::Function::new_no_args(
            r#"
    const out = [];
    for (const chunk of this) out.push(chunk);
    return out;
    "#
        );
        js_sys::Reflect::set(&js_iterable, &JsValue::from_str("toArray"), &to_array_fn.into()).unwrap();

        js_iterable.into()
    }

    pub fn chunks_with_strategy(
        &self,
        chunk_width: i32,
        chunk_height: i32,
        chunk_length: i32,
        strategy: &str,
        camera_x: f32,
        camera_y: f32,
        camera_z: f32
    ) -> Array {
        // Map the string strategy to enum
        let strategy_enum = match strategy {
            "distance_to_camera" => Some(ChunkLoadingStrategy::DistanceToCamera(camera_x, camera_y, camera_z)),
            "top_down" => Some(ChunkLoadingStrategy::TopDown),
            "bottom_up" => Some(ChunkLoadingStrategy::BottomUp),
            "center_outward" => Some(ChunkLoadingStrategy::CenterOutward),
            "random" => Some(ChunkLoadingStrategy::Random),
            _ => None // Default
        };

        // Use the enhanced iter_chunks method
        self.0.iter_chunks(chunk_width, chunk_height, chunk_length, strategy_enum)
            .map(|chunk| {
                let chunk_obj = js_sys::Object::new();
                js_sys::Reflect::set(&chunk_obj, &"chunk_x".into(), &chunk.chunk_x.into()).unwrap();
                js_sys::Reflect::set(&chunk_obj, &"chunk_y".into(), &chunk.chunk_y.into()).unwrap();
                js_sys::Reflect::set(&chunk_obj, &"chunk_z".into(), &chunk.chunk_z.into()).unwrap();

                let blocks_array = chunk.positions.into_iter()
                    .map(|pos| {
                        let block = self.0.get_block(pos.x, pos.y, pos.z).unwrap();
                        let obj = js_sys::Object::new();
                        js_sys::Reflect::set(&obj, &"x".into(), &pos.x.into()).unwrap();
                        js_sys::Reflect::set(&obj, &"y".into(), &pos.y.into()).unwrap();
                        js_sys::Reflect::set(&obj, &"z".into(), &pos.z.into()).unwrap();
                        js_sys::Reflect::set(&obj, &"name".into(), &JsValue::from_str(&block.name)).unwrap();
                        let properties = js_sys::Object::new();
                        for (key, value) in &block.properties {
                            js_sys::Reflect::set(&properties, &JsValue::from_str(key), &JsValue::from_str(value)).unwrap();
                        }
                        js_sys::Reflect::set(&obj, &"properties".into(), &properties).unwrap();
                        obj
                    })
                    .collect::<Array>();

                js_sys::Reflect::set(&chunk_obj, &"blocks".into(), &blocks_array).unwrap();
                chunk_obj
            })
            .collect::<Array>()
    }


    pub fn get_chunk_blocks(&self, offset_x: i32, offset_y: i32, offset_z: i32, width: i32, height: i32, length: i32) -> js_sys::Array {
        let blocks = self.0.iter_blocks()
            .filter(|(pos, _)| {
                pos.x >= offset_x && pos.x < offset_x + width &&
                    pos.y >= offset_y && pos.y < offset_y + height &&
                    pos.z >= offset_z && pos.z < offset_z + length
            })
            .map(|(pos, block)| {
                let obj = js_sys::Object::new();
                js_sys::Reflect::set(&obj, &"x".into(), &pos.x.into()).unwrap();
                js_sys::Reflect::set(&obj, &"y".into(), &pos.y.into()).unwrap();
                js_sys::Reflect::set(&obj, &"z".into(), &pos.z.into()).unwrap();
                js_sys::Reflect::set(&obj, &"name".into(), &JsValue::from_str(&block.name)).unwrap();
                let properties = js_sys::Object::new();
                for (key, value) in &block.properties {
                    js_sys::Reflect::set(&properties, &JsValue::from_str(key), &JsValue::from_str(value)).unwrap();
                }
                js_sys::Reflect::set(&obj, &"properties".into(), &properties).unwrap();
                obj
            })
            .collect::<js_sys::Array>();

        blocks
    }

    pub fn get_block_palette(&self) -> js_sys::Array {
        let palette_strings = self.0.get_block_palette_as_strings();
        let js_array = js_sys::Array::new();
        
        for block_string in palette_strings {
            js_array.push(&JsValue::from_str(&block_string));
        }
        
        js_array
    }


}


#[wasm_bindgen]
impl MchprsWorldWrapper {
    #[wasm_bindgen(constructor)]
    pub fn new(schematic: &SchematicWrapper) -> Result<MchprsWorldWrapper, JsValue> {

        let world = MchprsWorld::new(schematic.0.clone())
            .map_err(|e| JsValue::from_str(&format!("Failed to create MchprsWorld: {}", e)))?;

        Ok(MchprsWorldWrapper { world })
    }

    pub fn on_use_block(&mut self, x: i32, y: i32, z: i32) {
        self.world.on_use_block(BlockPos::new(x, y, z));
    }

    pub fn tick(&mut self, number_of_ticks: u32) {
        self.world.tick(number_of_ticks);
    }

    pub fn flush(&mut self) {
        self.world.flush();
    }

    pub fn is_lit(&self, x: i32, y: i32, z: i32) -> bool {
        self.world.is_lit(BlockPos::new(x, y, z))
    }

    pub fn get_lever_power(&self, x: i32, y: i32, z: i32) -> bool {
        self.world.get_lever_power(BlockPos::new(x, y, z))
    }

    pub fn get_redstone_power(&self, x: i32, y: i32, z: i32) -> u8 {
        self.world.get_redstone_power(BlockPos::new(x, y, z))
    }

    pub fn get_truth_table(&self) -> JsValue {
        // Get the truth table result from the Rust implementation
        let truth_table = generate_truth_table(&self.world.schematic);

        // Create a JavaScript array to hold the results
        let result = js_sys::Array::new();
        // Convert each row in the truth table to a JavaScript object
        for row in truth_table {
            let row_obj = js_sys::Object::new();

            // Add each entry in the row to the object
            for (key, value) in row {
                js_sys::Reflect::set(
                    &row_obj,
                    &JsValue::from_str(&key),
                    &JsValue::from_bool(value)
                ).unwrap();
            }

            result.push(&row_obj);
        }

        result.into()
    }
}


#[wasm_bindgen]
impl BlockStateWrapper {
    #[wasm_bindgen(constructor)]
    pub fn new(name: &str) -> Self {
        BlockStateWrapper(BlockState::new(name.to_string()))
    }

    pub fn with_property(&mut self, key: &str, value: &str) {
        self.0 = self.0.clone().with_prop(key, value);
    }

    pub fn name(&self) -> String {
        self.0.name.clone().to_string()
    }

    pub fn properties(&self) -> JsValue {
        let properties = self.0.properties.clone();
        let js_properties = js_sys::Object::new();
        for (key, value) in properties {
            js_sys::Reflect::set(&js_properties, &JsValue::from_str(&key.to_string()), &JsValue::from_str(&value.to_string())).unwrap();        }
        js_properties.into()
    }
}


// Standalone functions
#[wasm_bindgen]
pub fn debug_schematic(schematic: &SchematicWrapper) -> String {
    format!("{}\n{}", schematic.debug_info(), print_schematic(&schematic.0))
}

#[wasm_bindgen]
pub fn debug_json_schematic(schematic: &SchematicWrapper) -> String {
    format!("{}\n{}", schematic.debug_info(), print_json_schematic(&schematic.0))
}