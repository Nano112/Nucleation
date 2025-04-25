use crate::utils::{NbtMap, parse_custom_name, parse_items_array};
use std::collections::HashMap;
use quartz_nbt::{NbtCompound, NbtTag};
use rand::SeedableRng;
use serde::{Deserialize, Serialize};
use crate::{BlockState};
use crate::block_entity::BlockEntity;
use crate::block_position::BlockPosition;
use crate::bounding_box::BoundingBox;
use crate::chunk::Chunk;
use crate::entity::Entity;
use crate::metadata::Metadata;
use crate::region::Region;
use crate::utils::NbtValue;

#[derive(Serialize, Deserialize, Clone)]
pub struct UniversalSchematic {
    pub metadata: Metadata,
    pub regions: HashMap<String, Region>,
    pub default_region_name: String,
}

pub enum ChunkLoadingStrategy {
    Default,
    DistanceToCamera(f32, f32, f32), // Camera position
    TopDown,
    BottomUp,
    CenterOutward,
    Random
}
pub type SimpleBlockMapping = (&'static str, Vec<(&'static str, &'static str)>);

impl UniversalSchematic {
    pub fn new(name: String) -> Self {
        UniversalSchematic {
            metadata: Metadata {
                name: Some(name),
                ..Metadata::default()
            },
            regions: HashMap::new(),
            default_region_name: "Main".to_string(),
        }
    }


    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block: BlockState) -> bool {
        let region_name = self.default_region_name.clone();
        self.set_block_in_region(&region_name, x, y, z, block)
    }

    pub fn set_block_in_region(&mut self, region_name: &str, x: i32, y: i32, z: i32, block: BlockState) -> bool {
        let region = self.regions.entry(region_name.to_string()).or_insert_with(|| {
            Region::new(region_name.to_string(), (x, y, z), (1, 1, 1))
        });

        region.set_block(x, y, z, block)
    }


    pub fn from_layers(name: String, block_mappings: &[(&'static char, SimpleBlockMapping)], layers: &str) -> Self {
        let mut schematic = UniversalSchematic::new(name);
        let full_mappings = Self::convert_to_full_mappings(block_mappings);

        let layers: Vec<&str> = layers.split("\n\n")
            .map(|layer| layer.trim())
            .filter(|layer| !layer.is_empty())
            .collect();

        for (y, layer) in layers.iter().enumerate() {
            let rows: Vec<&str> = layer.lines()
                .map(|row| row.trim())
                .filter(|row| !row.is_empty())
                .collect();

            for (z, row) in rows.iter().enumerate() {
                for (x, c) in row.chars().enumerate() {
                    if let Some(block_state) = full_mappings.get(&c) {
                        schematic.set_block(x as i32, y as i32, z as i32, block_state.clone());
                    } else if c != ' ' {
                        println!("Warning: Unknown character '{}' at position ({}, {}, {})", c, x, y, z);
                    }
                }
            }
        }

        schematic
    }

    fn convert_to_full_mappings(simple_mappings: &[(&'static char, SimpleBlockMapping)]) -> HashMap<char, BlockState> {
        simple_mappings.iter().map(|(&c, (name, props))| {
            let block_state = BlockState::new(format!("minecraft:{}", name))
                .with_properties(props.iter().map(|&(k, v)| (k.to_string(), v.to_string())).collect());
            (c, block_state)
        }).collect()
    }

    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Option<&BlockState> {
        for region in self.regions.values() {
            if region.get_bounding_box().contains((x, y, z)) {
                return region.get_block(x, y, z);
            }
        }
        None
    }

    pub fn get_block_entity(&self, position: BlockPosition) -> Option<&BlockEntity> {
        for region in self.regions.values() {
            if region.get_bounding_box().contains((position.x, position.y, position.z)) {
                return region.get_block_entity(position);
            }
        }
        None
    }

    pub fn get_block_entities_as_list(&self) -> Vec<BlockEntity> {
        let mut block_entities = Vec::new();
        for region in self.regions.values() {
            block_entities.extend(region.get_block_entities_as_list());
        }
        block_entities
    }

    pub fn get_entities_as_list(&self) -> Vec<Entity> {
        let mut entities = Vec::new();
        for region in self.regions.values() {
            entities.extend(region.entities.clone());
        }
        entities
    }

    pub fn set_block_entity(&mut self, position: BlockPosition, block_entity: BlockEntity) -> bool {
        let region_name = self.default_region_name.clone();
        self.set_block_entity_in_region(&region_name, position, block_entity)
    }

    pub fn set_block_entity_in_region(&mut self, region_name: &str, position: BlockPosition, block_entity: BlockEntity) -> bool {
        let region = self.regions.entry(region_name.to_string()).or_insert_with(|| {
            Region::new(region_name.to_string(), (position.x, position.y, position.z), (1, 1, 1))
        });

        region.set_block_entity(position, block_entity)
    }

    pub fn get_blocks(&self) -> Vec<BlockState> {
        let mut blocks: Vec<BlockState> = Vec::new();
        for region in self.regions.values() {
            let region_palette = region.get_palette();
            for block_index in &region.blocks {
                blocks.push(region_palette[*block_index as usize].clone());
            }
        }
        blocks
    }

    pub fn get_region_names(&self) -> Vec<String> {
        self.regions.keys().cloned().collect()
    }

    pub fn get_region_from_index(&self, index: usize) -> Option<&Region> {
        self.regions.values().nth(index)
    }


    pub fn get_block_from_region(&self, region_name: &str, x: i32, y: i32, z: i32) -> Option<&BlockState> {
        self.regions.get(region_name).and_then(|region| region.get_block(x, y, z))
    }

    pub fn get_dimensions(&self) -> (i32, i32, i32) {
        let bounding_box = self.get_bounding_box();
        bounding_box.get_dimensions()
    }


    pub fn get_json_string(&self) -> Result<String, String> {
        // Attempt to serialize the name
        let metadata_json = serde_json::to_string(&self.metadata)
            .map_err(|e| format!("Failed to serialize 'metadata' in UniversalSchematic: {}", e))?;

        // Attempt to serialize the regions
        let regions_json = serde_json::to_string(&self.regions)
            .map_err(|e| format!("Failed to serialize 'regions' in UniversalSchematic: {}", e))?;


        // Combine everything into a single JSON object manually
        let combined_json = format!(
            "{{\"metadata\":{},\"regions\":{}}}",
            metadata_json, regions_json
        );

        Ok(combined_json)
    }

    pub(crate) fn total_blocks(&self) -> i32 {
        self.regions.values().map(|r| r.count_blocks() as i32).sum()
    }

    pub(crate) fn total_volume(&self) -> i32 {
        self.regions.values().map(|r| r.volume() as i32).sum()
    }


    pub fn get_region_bounding_box(&self, region_name: &str) -> Option<BoundingBox> {
        self.regions.get(region_name).map(|region| region.get_bounding_box())
    }

    pub fn get_schematic_bounding_box(&self) -> Option<BoundingBox> {
        if self.regions.is_empty() {
            return None;
        }

        let mut bounding_box = self.regions.values().next().unwrap().get_bounding_box();
        for region in self.regions.values().skip(1) {
            bounding_box = bounding_box.union(&region.get_bounding_box());
        }
        Some(bounding_box)
    }


    pub fn add_region(&mut self, region: Region) -> bool {
        if self.regions.contains_key(&region.name) {
            false
        } else {
            self.regions.insert(region.name.clone(), region);
            true
        }
    }

    pub fn remove_region(&mut self, name: &str) -> Option<Region> {
        self.regions.remove(name)
    }

    pub fn get_region(&self, name: &str) -> Option<&Region> {
        self.regions.get(name)
    }

    pub fn get_region_mut(&mut self, name: &str) -> Option<&mut Region> {
        self.regions.get_mut(name)
    }

    pub fn get_merged_region(&self) -> Region {
        let mut merged_region = self.regions.values().next().unwrap().clone();

        for region in self.regions.values().skip(1) {
            merged_region.merge(region);
        }

        merged_region
    }

    pub fn add_block_entity_in_region(&mut self, region_name: &str, block_entity: BlockEntity) -> bool {
        let region = self.regions.entry(region_name.to_string()).or_insert_with(|| {
            Region::new(region_name.to_string(), block_entity.position, (1, 1, 1))
        });

        region.add_block_entity(block_entity);
        true
    }

    pub fn remove_block_entity_in_region(&mut self, region_name: &str, position: (i32, i32, i32)) -> Option<BlockEntity> {
        self.regions.get_mut(region_name)?.remove_block_entity(position)
    }

    pub fn add_block_entity(&mut self, block_entity: BlockEntity) -> bool {
        let region_name = self.default_region_name.clone();
        self.add_block_entity_in_region(&region_name, block_entity)
    }

    pub fn remove_block_entity(&mut self, position: (i32, i32, i32)) -> Option<BlockEntity> {
        let region_name = self.default_region_name.clone();
        self.remove_block_entity_in_region(&region_name, position)
    }

    pub fn add_entity_in_region(&mut self, region_name: &str, entity: Entity) -> bool {
        let region = self.regions.entry(region_name.to_string()).or_insert_with(|| {
            let rounded_position = (entity.position.0.round() as i32, entity.position.1.round() as i32, entity.position.2.round() as i32);
            Region::new(region_name.to_string(), rounded_position, (1, 1, 1))
        });

        region.add_entity(entity);
        true
    }

    pub fn remove_entity_in_region(&mut self, region_name: &str, index: usize) -> Option<Entity> {
        self.regions.get_mut(region_name)?.remove_entity(index)
    }

    pub fn add_entity(&mut self, entity: Entity) -> bool {
        let region_name = self.default_region_name.clone();
        self.add_entity_in_region(&region_name, entity)
    }

    pub fn remove_entity(&mut self, index: usize) -> Option<Entity> {
        let region_name = self.default_region_name.clone();
        self.remove_entity_in_region(&region_name, index)
    }

    pub fn to_nbt(&self) -> NbtCompound {
        let mut root = NbtCompound::new();

        root.insert("Metadata", self.metadata.to_nbt());

        let mut regions_tag = NbtCompound::new();
        for (name, region) in &self.regions {
            regions_tag.insert(name, region.to_nbt());
        }
        root.insert("Regions", NbtTag::Compound(regions_tag));

        root.insert("DefaultRegion", NbtTag::String(self.default_region_name.clone()));

        root
    }

    pub fn from_nbt(nbt: NbtCompound) -> Result<Self, String> {
        let metadata = Metadata::from_nbt(nbt.get::<_, &NbtCompound>("Metadata")
            .map_err(|e| format!("Failed to get Metadata: {}", e))?)?;

        let regions_tag = nbt.get::<_, &NbtCompound>("Regions")
            .map_err(|e| format!("Failed to get Regions: {}", e))?;
        let mut regions = HashMap::new();
        for (region_name, region_tag) in regions_tag.inner() {
            if let NbtTag::Compound(region_compound) = region_tag {
                regions.insert(region_name.to_string(), Region::from_nbt(&region_compound.clone())?);
            }
        }

        let default_region_name = nbt.get::<_, &str>("DefaultRegion")
            .map_err(|e| format!("Failed to get DefaultRegion: {}", e))?
            .to_string();

        Ok(UniversalSchematic {
            metadata,
            regions,
            default_region_name,
        })
    }


    pub fn get_bounding_box(&self) -> BoundingBox {
        if self.regions.is_empty() {
            return BoundingBox::new((0, 0, 0), (0, 0, 0));
        }
        let mut bounding_box = BoundingBox::new((i32::MAX, i32::MAX, i32::MAX), (i32::MIN, i32::MIN, i32::MIN));

        for region in self.regions.values() {
            let region_bb = region.get_bounding_box();
            bounding_box = bounding_box.union(&region_bb);
        }

        bounding_box
    }

    pub fn to_schematic(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        crate::formats::schematic::to_schematic(self)
    }

    pub fn from_schematic(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        crate::formats::schematic::from_schematic(data)
    }

    pub fn count_block_types(&self) -> HashMap<BlockState, usize> {
        let mut block_counts = HashMap::new();
        for region in self.regions.values() {
            let region_block_counts = region.count_block_types();
            for (block, count) in region_block_counts {
                *block_counts.entry(block).or_insert(0) += count;
            }
        }
        block_counts
    }


    pub fn copy_region(
        &mut self,
        from_schematic: &UniversalSchematic,
        bounds: &BoundingBox,
        target_position: (i32, i32, i32),
        excluded_blocks: &[BlockState],
    ) -> Result<(), String> {
        let offset = (
            target_position.0 - bounds.min.0,
            target_position.1 - bounds.min.1,
            target_position.2 - bounds.min.2
        );

        // Copy blocks
        for x in bounds.min.0..=bounds.max.0 {
            for y in bounds.min.1..=bounds.max.1 {
                for z in bounds.min.2..=bounds.max.2 {
                    if let Some(block) = from_schematic.get_block(x, y, z) {
                        if excluded_blocks.contains(block) {
                            continue;
                        }
                        let new_x = x + offset.0;
                        let new_y = y + offset.1;
                        let new_z = z + offset.2;
                        self.set_block(new_x, new_y, new_z, block.clone());
                    }
                }
            }
        }

        // Copy block entities
        for x in bounds.min.0..=bounds.max.0 {
            for y in bounds.min.1..=bounds.max.1 {
                for z in bounds.min.2..=bounds.max.2 {
                    let pos = BlockPosition { x, y, z };
                    if let Some(block_entity) = from_schematic.get_block_entity(pos) {
                        let mut new_block_entity = block_entity.clone();
                        new_block_entity.position = (
                            block_entity.position.0 + offset.0,
                            block_entity.position.1 + offset.1,
                            block_entity.position.2 + offset.2
                        );
                        self.set_block_entity(BlockPosition {
                            x: x + offset.0,
                            y: y + offset.1,
                            z: z + offset.2,
                        }, new_block_entity);
                    }
                }
            }
        }

        // Copy entities that are within the bounds
        for region in from_schematic.regions.values() {
            for entity in &region.entities {
                let entity_pos = (
                    entity.position.0.floor() as i32,
                    entity.position.1.floor() as i32,
                    entity.position.2.floor() as i32
                );

                if bounds.contains(entity_pos) {
                    let mut new_entity = entity.clone();
                    new_entity.position = (
                        entity.position.0 + offset.0 as f64,
                        entity.position.1 + offset.1 as f64,
                        entity.position.2 + offset.2 as f64
                    );
                    self.add_entity(new_entity);
                }
            }
        }

        Ok(())
    }

    pub fn split_into_chunks(&self, chunk_width: i32, chunk_height: i32, chunk_length: i32) -> Vec<Chunk> {
        use std::collections::HashMap;
        let mut chunk_map: HashMap<(i32, i32, i32), Vec<BlockPosition>> = HashMap::new();
        let bbox = self.get_bounding_box();

        // Helper function to get chunk coordinate
        let get_chunk_coord = |pos: i32, chunk_size: i32| -> i32 {
            let offset = if pos < 0 { chunk_size - 1 } else { 0 };
            (pos - offset) / chunk_size
        };

        // Iterate through the actual bounding box instead of dimensions
        for x in bbox.min.0..=bbox.max.0 {
            for y in bbox.min.1..=bbox.max.1 {
                for z in bbox.min.2..=bbox.max.2 {
                    if self.get_block(x, y, z).is_some() {
                        let chunk_x = get_chunk_coord(x, chunk_width);
                        let chunk_y = get_chunk_coord(y, chunk_height);
                        let chunk_z = get_chunk_coord(z, chunk_length);
                        let chunk_key = (chunk_x, chunk_y, chunk_z);

                        chunk_map
                            .entry(chunk_key)
                            .or_insert_with(Vec::new)
                            .push(BlockPosition { x, y, z });
                    }
                }
            }
        }

        chunk_map.into_iter()
            .map(|((chunk_x, chunk_y, chunk_z), positions)| Chunk {
                chunk_x,
                chunk_y,
                chunk_z,
                positions,
            })
            .collect()
    }


    pub fn iter_blocks(&self) -> impl Iterator<Item=(BlockPosition, &BlockState)> {
        self.regions.values().flat_map(|region| {
            region.blocks.iter().enumerate().filter_map(move |(index, block_index)| {
                let (x, y, z) = region.index_to_coords(index);
                Some((
                    BlockPosition { x, y, z },
                    &region.palette[*block_index as usize]
                ))
            })
        })
    }

    pub fn iter_chunks(&self, chunk_width: i32, chunk_height: i32, chunk_length: i32,
                       strategy: Option<ChunkLoadingStrategy>) -> impl Iterator<Item=Chunk> + '_ {
        let chunks = self.split_into_chunks(chunk_width, chunk_height, chunk_length);

        // Apply sorting based on strategy
        let mut ordered_chunks = chunks;
        if let Some(strategy) = strategy {
            match strategy {
                ChunkLoadingStrategy::Default => {
                    // Default order - no sorting needed
                },
                ChunkLoadingStrategy::DistanceToCamera(cam_x, cam_y, cam_z) => {
                    // Sort by distance to camera
                    ordered_chunks.sort_by(|a, b| {
                        let a_center_x = (a.chunk_x * chunk_width) + (chunk_width / 2);
                        let a_center_y = (a.chunk_y * chunk_height) + (chunk_height / 2);
                        let a_center_z = (a.chunk_z * chunk_length) + (chunk_length / 2);

                        let b_center_x = (b.chunk_x * chunk_width) + (chunk_width / 2);
                        let b_center_y = (b.chunk_y * chunk_height) + (chunk_height / 2);
                        let b_center_z = (b.chunk_z * chunk_length) + (chunk_length / 2);

                        let a_dist = (a_center_x as f32 - cam_x).powi(2) +
                            (a_center_y as f32 - cam_y).powi(2) +
                            (a_center_z as f32 - cam_z).powi(2);

                        let b_dist = (b_center_x as f32 - cam_x).powi(2) +
                            (b_center_y as f32 - cam_y).powi(2) +
                            (b_center_z as f32 - cam_z).powi(2);

                        // Sort by ascending distance (closest first)
                        a_dist.partial_cmp(&b_dist).unwrap_or(std::cmp::Ordering::Equal)
                    });
                },
                ChunkLoadingStrategy::TopDown => {
                    // Sort by y-coordinate, highest first
                    ordered_chunks.sort_by(|a, b| b.chunk_y.cmp(&a.chunk_y));
                },
                ChunkLoadingStrategy::BottomUp => {
                    // Sort by y-coordinate, lowest first
                    ordered_chunks.sort_by(|a, b| a.chunk_y.cmp(&b.chunk_y));
                },
                ChunkLoadingStrategy::CenterOutward => {
                    // Calculate schematic center in chunk coordinates
                    let (width, height, depth) = self.get_dimensions();
                    let center_x = (width / 2) / chunk_width;
                    let center_y = (height / 2) / chunk_height;
                    let center_z = (depth / 2) / chunk_length;

                    // Sort by distance from center
                    ordered_chunks.sort_by(|a, b| {
                        let a_dist = (a.chunk_x - center_x).pow(2) +
                            (a.chunk_y - center_y).pow(2) +
                            (a.chunk_z - center_z).pow(2);

                        let b_dist = (b.chunk_x - center_x).pow(2) +
                            (b.chunk_y - center_y).pow(2) +
                            (b.chunk_z - center_z).pow(2);

                        a_dist.cmp(&b_dist)
                    });
                },
                ChunkLoadingStrategy::Random => {
                    // Shuffle the chunks using a deterministic seed
                    use std::hash::{Hash, Hasher};
                    use std::collections::hash_map::DefaultHasher;

                    let mut hasher = DefaultHasher::new();
                    if let Some(name) = &self.metadata.name {
                        name.hash(&mut hasher);
                    } else {
                        "Default".hash(&mut hasher);
                    }
                    let seed = hasher.finish();

                    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
                    use rand::seq::SliceRandom;
                    ordered_chunks.shuffle(&mut rng);
                },
            }
        }

        // Process each chunk like in the original implementation
        ordered_chunks.into_iter().map(move |chunk| {
            let positions = chunk.positions;
            let blocks = positions.into_iter()
                .filter_map(|pos| {
                    self.get_block(pos.x, pos.y, pos.z)
                        .map(|block| (pos, block))
                })
                .collect::<Vec<_>>();

            Chunk {
                chunk_x: chunk.chunk_x,
                chunk_y: chunk.chunk_y,
                chunk_z: chunk.chunk_z,
                positions: blocks.iter().map(|(pos, _)| *pos).collect(),
            }
        })
    }

    // Keep the original method for backward compatibility
    pub fn iter_chunks_original(&self, chunk_width: i32, chunk_height: i32, chunk_length: i32) -> impl Iterator<Item=Chunk> + '_ {
        self.iter_chunks(chunk_width, chunk_height, chunk_length, None)
    }

    pub fn set_block_from_string(&mut self, x: i32, y: i32, z: i32, block_string: &str) -> Result<bool, String> {
        let (block_state, nbt_data) = Self::parse_block_string(block_string)?;

        // Set the basic block first
        if !self.set_block(x, y, z, block_state.clone()) {
            return Ok(false);
        }

        // If we have NBT data, create and set the block entity
        if let Some(nbt_data) = nbt_data {
            let mut block_entity = BlockEntity::new(
                block_state.name.clone(),
                (x, y, z),
            );

            // Add NBT data
            for (key, value) in nbt_data {
                block_entity = block_entity.with_nbt_data(key, value);
            }

            self.set_block_entity(BlockPosition { x, y, z }, block_entity);
        }

        Ok(true)
    }

    /// Parses a block string into its components (block state and optional NBT data)
    fn calculate_items_for_signal(signal_strength: u8) -> u32 {
        if signal_strength == 0 {
            return 0;
        }

        const BARREL_SLOTS: u32 = 27;
        const MAX_STACK: u32 = 64;
        const MAX_SIGNAL: u32 = 14;

        let calculated = ((BARREL_SLOTS * MAX_STACK) as f64 / MAX_SIGNAL as f64)
            * (signal_strength as f64 - 1.0);
        let items_needed = calculated.ceil() as u32;

        std::cmp::max(signal_strength as u32, items_needed)
    }

    /// Creates Items NBT data for a barrel to achieve desired signal strength
    fn create_barrel_items_nbt(signal_strength: u8) -> Vec<NbtValue> {
        let total_items = Self::calculate_items_for_signal(signal_strength);
        let mut items = Vec::new();
        let mut remaining_items = total_items;
        let mut slot: u8 = 0;

        while remaining_items > 0 {
            let stack_size = std::cmp::min(remaining_items, 64) as u8;
            let mut item_nbt = NbtMap::new();  // Using NbtMap instead of HashMap
            item_nbt.insert("Count".to_string(), NbtValue::Byte(stack_size as i8));
            item_nbt.insert("Slot".to_string(), NbtValue::Byte(slot as i8));
            item_nbt.insert("id".to_string(), NbtValue::String("minecraft:redstone_block".to_string()));

            items.push(NbtValue::Compound(item_nbt));

            remaining_items -= stack_size as u32;
            slot += 1;
        }

        items
    }
    /// Parse a block string into its components, handling special signal strength case
    pub fn parse_block_string(block_string: &str) -> Result<(BlockState, Option<HashMap<String, NbtValue>>), String> {
        let mut parts = block_string.splitn(2, '{');
        let block_state_str = parts.next().unwrap().trim();
        let nbt_str = parts.next().map(|s| s.trim_end_matches('}'));

        // Parse block state
        let block_state = if block_state_str.contains('[') {
            let mut state_parts = block_state_str.splitn(2, '[');
            let block_name = state_parts.next().unwrap();
            let properties_str = state_parts.next()
                .ok_or("Missing properties closing bracket")?
                .trim_end_matches(']');

            let mut properties = HashMap::new();
            for prop in properties_str.split(',') {
                let mut kv = prop.split('=');
                let key = kv.next().ok_or("Missing property key")?.trim();
                let value = kv.next().ok_or("Missing property value")?.trim()
                    .trim_matches(|c| c == '\'' || c == '"');
                properties.insert(key.to_string(), value.to_string());
            }

            BlockState::new(block_name.to_string()).with_properties(properties)
        } else {
            BlockState::new(block_state_str.to_string())
        };

        // Parse NBT data if present
        let nbt_data = if let Some(nbt_str) = nbt_str {
            let mut nbt_map = HashMap::new();

            // Check for signal strength specification
            if block_state.get_name() == "minecraft:barrel" && nbt_str.contains("signal=") {
                if let Some(signal_str) = nbt_str.split('=').nth(1) {
                    let signal_strength: u8 = signal_str.trim().parse()
                        .map_err(|_| "Invalid signal strength value")?;

                    if signal_strength > 15 {
                        return Err("Signal strength must be between 0 and 15".to_string());
                    }

                    let items = Self::create_barrel_items_nbt(signal_strength);
                    nbt_map.insert("Items".to_string(), NbtValue::List(items));
                }
            } else {
                // Handle regular NBT parsing
                if nbt_str.contains("Items:[") {
                    let items = parse_items_array(nbt_str)?;
                    nbt_map.insert("Items".to_string(), NbtValue::List(items));
                }

                if nbt_str.contains("CustomName:") {
                    let name = parse_custom_name(nbt_str)?;
                    nbt_map.insert("CustomName".to_string(), NbtValue::String(name));
                }
            }

            Some(nbt_map)
        } else {
            None
        };

        Ok((block_state, nbt_data))
    }

    pub fn create_schematic_from_region(&self, bounds: &BoundingBox) -> Self {
        let mut new_schematic = UniversalSchematic::new(format!("Region_{}", self.default_region_name));

        // Normalize coordinates to start at 0,0,0 in the new schematic
        let offset = (
            -bounds.min.0,
            -bounds.min.1,
            -bounds.min.2
        );

        // Copy blocks
        for x in bounds.min.0..=bounds.max.0 {
            for y in bounds.min.1..=bounds.max.1 {
                for z in bounds.min.2..=bounds.max.2 {
                    if let Some(block) = self.get_block(x, y, z) {
                        let new_x = x + offset.0;
                        let new_y = y + offset.1;
                        let new_z = z + offset.2;
                        new_schematic.set_block(new_x, new_y, new_z, block.clone());
                    }
                }
            }
        }

        // Copy block entities
        for x in bounds.min.0..=bounds.max.0 {
            for y in bounds.min.1..=bounds.max.1 {
                for z in bounds.min.2..=bounds.max.2 {
                    let pos = BlockPosition { x, y, z };
                    if let Some(block_entity) = self.get_block_entity(pos) {
                        let mut new_block_entity = block_entity.clone();
                        new_block_entity.position = (
                            block_entity.position.0 + offset.0,
                            block_entity.position.1 + offset.1,
                            block_entity.position.2 + offset.2
                        );
                        new_schematic.set_block_entity(BlockPosition {
                            x: x + offset.0,
                            y: y + offset.1,
                            z: z + offset.2,
                        }, new_block_entity);
                    }
                }
            }
        }

        // Copy entities that are within the bounds
        for region in self.regions.values() {
            for entity in &region.entities {
                let entity_pos = (
                    entity.position.0.floor() as i32,
                    entity.position.1.floor() as i32,
                    entity.position.2.floor() as i32
                );

                if bounds.contains(entity_pos) {
                    let mut new_entity = entity.clone();
                    new_entity.position = (
                        entity.position.0 + offset.0 as f64,
                        entity.position.1 + offset.1 as f64,
                        entity.position.2 + offset.2 as f64
                    );
                    new_schematic.add_entity(new_entity);
                }
            }
        }

        new_schematic
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use quartz_nbt::io::{read_nbt, write_nbt};
    use crate::block_entity;
    use crate::item::ItemStack;
    use super::*;


    #[test]
    fn test_schematic_operations() {
        let mut schematic = UniversalSchematic::new("Test Schematic".to_string());

        // Test automatic region creation and expansion
        let stone = BlockState::new("minecraft:stone".to_string());
        let dirt = BlockState::new("minecraft:dirt".to_string());

        assert!(schematic.set_block(0, 0, 0, stone.clone()));
        assert_eq!(schematic.get_block(0, 0, 0), Some(&stone));

        assert!(schematic.set_block(5, 5, 5, dirt.clone()));
        assert_eq!(schematic.get_block(5, 5, 5), Some(&dirt));

        // Check that the default region was created and expanded
        let default_region = schematic.get_region("Main").unwrap();

        // Test explicit region creation and manipulation
        let obsidian = BlockState::new("minecraft:obsidian".to_string());
        assert!(schematic.set_block_in_region("Custom", 10, 10, 10, obsidian.clone()));
        assert_eq!(schematic.get_block_from_region("Custom", 10, 10, 10), Some(&obsidian));

        // Check that the custom region was created
        let custom_region = schematic.get_region("Custom").unwrap();
        assert_eq!(custom_region.position, (10, 10, 10));

        // Test manual region addition
        let region2 = Region::new("Region2".to_string(), (20, 0, 0), (5, 5, 5));
        assert!(schematic.add_region(region2));
        assert!(!schematic.add_region(Region::new("Region2".to_string(), (0, 0, 0), (1, 1, 1))));

        // Test getting non-existent blocks
        assert_eq!(schematic.get_block(100, 100, 100), None);
        assert_eq!(schematic.get_block_from_region("NonexistentRegion", 0, 0, 0), None);

        // Test removing regions
        assert!(schematic.remove_region("Region2").is_some());
        assert!(schematic.remove_region("Region2").is_none());

        // Test that removed region's blocks are no longer accessible
        assert_eq!(schematic.get_block_from_region("Region2", 20, 0, 0), None);
    }

    #[test]
    fn test_bounding_box_and_dimensions() {
        // Create a new empty schematic
        let mut schematic = UniversalSchematic::new("Test Bounding Box".to_string());

        // Initially, the schematic should be empty or have minimal dimensions
        let initial_bbox = schematic.get_bounding_box();
        println!("Initial bounding box: {:?}", initial_bbox);
        println!("Initial dimensions: {:?}", schematic.get_dimensions());

        // Add blocks at various positions to test expansion
        schematic.set_block(0, 0, 0, BlockState::new("minecraft:stone".to_string()));
        schematic.set_block(4, 4, 4, BlockState::new("minecraft:sea_lantern".to_string()));

        // Now check the bounding box and dimensions
        let bbox = schematic.get_bounding_box();
        println!("Updated bounding box: min={:?}, max={:?}", bbox.min, bbox.max);

        // The bounding box should contain points from (0,0,0) to (4,4,4)
        assert_eq!(bbox.min, (0, 0, 0));
        assert_eq!(bbox.max, (4, 4, 4));

        // The dimensions should be 5x5x5 (including both end points)
        let dimensions = schematic.get_dimensions();
        assert_eq!(dimensions, (5, 5, 5));

        // Add a block with negative coordinates to test expansion in that direction
        schematic.set_block(-3, 2, 1, BlockState::new("minecraft:dirt".to_string()));

        // Check the updated bounding box
        let expanded_bbox = schematic.get_bounding_box();
        assert_eq!(expanded_bbox.min, (-3, 0, 0));
        assert_eq!(expanded_bbox.max, (4, 4, 4));

        // And the updated dimensions
        let expanded_dimensions = schematic.get_dimensions();
        assert_eq!(expanded_dimensions, (8, 5, 5));
    }


    #[test]
    fn test_schematic_large_coordinates() {
        let mut schematic = UniversalSchematic::new("Large Schematic".to_string());

        let far_block = BlockState::new("minecraft:diamond_block".to_string());
        assert!(schematic.set_block(1000, 1000, 1000, far_block.clone()));
        assert_eq!(schematic.get_block(1000, 1000, 1000), Some(&far_block));

        let main_region = schematic.get_region("Main").unwrap();
        assert_eq!(main_region.position, (1000, 1000, 1000));
        assert_eq!(main_region.size, (1, 1, 1));

        // Test that blocks outside the region are not present
        assert_eq!(schematic.get_block(999, 1000, 1000), None);
        // Since the schematic region scaling scales by a factor of 1.5, we need to check 2 blocks away (we apply a ceil)  since we expanded the region (previously 1x1x1)
        assert_eq!(schematic.get_block(1002, 1000, 1000), None);
    }

    #[test]
    fn test_schematic_region_expansion() {
        let mut schematic = UniversalSchematic::new("Expanding Schematic".to_string());

        let block1 = BlockState::new("minecraft:stone".to_string());
        let block2 = BlockState::new("minecraft:dirt".to_string());

        assert!(schematic.set_block(0, 0, 0, block1.clone()));
        assert!(schematic.set_block(10, 20, 30, block2.clone()));

        let main_region = schematic.get_region("Main").unwrap();
        assert_eq!(main_region.position, (0, 0, 0));

        assert_eq!(schematic.get_block(0, 0, 0), Some(&block1));
        assert_eq!(schematic.get_block(10, 20, 30), Some(&block2));
        assert_eq!(schematic.get_block(5, 10, 15), Some(&BlockState::new("minecraft:air".to_string())));
    }

    #[test]
    fn test_copy_bounded_region() {
        // Create source schematic
        let mut source = UniversalSchematic::new("Source".to_string());

        // Add some blocks in a pattern
        source.set_block(0, 0, 0, BlockState::new("minecraft:stone".to_string()));
        source.set_block(1, 1, 1, BlockState::new("minecraft:dirt".to_string()));
        source.set_block(2, 2, 2, BlockState::new("minecraft:diamond_block".to_string()));

        // Add a block entity
        let chest = BlockEntity::create_chest((1, 1, 1), vec![
            ItemStack::new("minecraft:diamond", 64).with_slot(0)
        ]);
        source.set_block_entity(BlockPosition { x: 1, y: 1, z: 1 }, chest);

        // Add an entity
        let entity = Entity::new("minecraft:creeper".to_string(), (1.5, 1.0, 1.5));
        source.add_entity(entity);

        // Create target schematic
        let mut target = UniversalSchematic::new("Target".to_string());

        // Define a bounding box that includes part of the pattern
        let bounds = BoundingBox::new((0, 0, 0), (1, 1, 1));

        // Copy to new position
        assert!(target.copy_region(&source, &bounds, (10, 10, 10), &[]).is_ok());

        // Verify copied blocks
        assert_eq!(target.get_block(10, 10, 10).unwrap().get_name(), "minecraft:stone");
        assert_eq!(target.get_block(11, 11, 11).unwrap().get_name(), "minecraft:dirt");

        // Block at (2, 2, 2) should not have been copied as it's outside bounds
        assert!(target.get_block(12, 12, 12).is_none());

        // Verify block entity was copied and moved
        assert!(target.get_block_entity(BlockPosition { x: 11, y: 11, z: 11 }).is_some());

        // Verify entity was copied and moved
        let main_region = target.get_region("Main").unwrap();
        assert_eq!(main_region.entities.len(), 1);
        assert_eq!(
            main_region.entities[0].position,
            (11.5, 11.0, 11.5)
        );
    }

    #[test]
    fn test_copy_region_excluded_blocks() {
        // Create source schematic
        let mut source = UniversalSchematic::new("Source".to_string());

        // Add blocks in a pattern including blocks we'll want to exclude
        let stone = BlockState::new("minecraft:stone".to_string());
        let dirt = BlockState::new("minecraft:dirt".to_string());
        let diamond = BlockState::new("minecraft:diamond_block".to_string());
        let air = BlockState::new("minecraft:air".to_string());

        // Create a 2x2x2 cube with different blocks
        source.set_block(0, 0, 0, stone.clone());
        source.set_block(0, 1, 0, dirt.clone());  // Changed position to avoid overlap
        source.set_block(1, 0, 0, diamond.clone());
        source.set_block(1, 1, 0, dirt.clone());

        // Create target schematic
        let mut target = UniversalSchematic::new("Target".to_string());

        // Define bounds that include all blocks
        let bounds = BoundingBox::new((0, 0, 0), (1, 1, 0));

        // List of blocks to exclude (stone and diamond)
        let excluded_blocks = vec![stone.clone(), diamond.clone()];

        // Copy region with exclusions to position (10, 10, 10)
        assert!(target.copy_region(&source, &bounds, (10, 10, 10), &excluded_blocks).is_ok());

        // Test some specific positions
        // Where dirt blocks were in source (should be copied)
        assert_eq!(target.get_block(10, 11, 10), Some(&dirt), "Dirt block should be copied at (10, 11, 10)");
        assert_eq!(target.get_block(11, 11, 10), Some(&dirt), "Dirt block should be copied at (11, 11, 10)");

        // check that excluded blocks were not copied
        assert_eq!(target.get_block(10, 10, 10), None, "Stone block should not be copied at (10, 10, 10)");
        assert_eq!(target.get_block(11, 10, 10), None, "Diamond block should not be copied at (11, 10, 10)");

        // Count the total number of dirt blocks
        let dirt_blocks: Vec<_> = target.get_blocks().into_iter()
            .filter(|b| b == &dirt)
            .collect();

        assert_eq!(dirt_blocks.len(), 2, "Should have exactly 2 dirt blocks");
    }

    #[test]
    fn test_schematic_negative_coordinates() {
        let mut schematic = UniversalSchematic::new("Negative Coordinates Schematic".to_string());

        let neg_block = BlockState::new("minecraft:emerald_block".to_string());
        assert!(schematic.set_block(-10, -10, -10, neg_block.clone()));
        assert_eq!(schematic.get_block(-10, -10, -10), Some(&neg_block));

        let main_region = schematic.get_region("Main").unwrap();
        assert!(main_region.position.0 <= -10 && main_region.position.1 <= -10 && main_region.position.2 <= -10);
    }


    #[test]
    fn test_entity_operations() {
        let mut schematic = UniversalSchematic::new("Test Schematic".to_string());

        let entity = Entity::new("minecraft:creeper".to_string(), (10.5, 65.0, 20.5))
            .with_nbt_data("Fuse".to_string(), "30".to_string());

        assert!(schematic.add_entity(entity.clone()));

        let region = schematic.get_region("Main").unwrap();
        assert_eq!(region.entities.len(), 1);
        assert_eq!(region.entities[0], entity);

        let removed_entity = schematic.remove_entity(0).unwrap();
        assert_eq!(removed_entity, entity);

        let region = schematic.get_region("Main").unwrap();
        assert_eq!(region.entities.len(), 0);
    }

    #[test]
    fn test_block_entity_operations() {
        let mut schematic = UniversalSchematic::new("Test Schematic".to_string());


        let chest = BlockEntity::create_chest((5, 10, 15), vec![
            ItemStack::new("minecraft:diamond", 64).with_slot(0)
        ]);

        assert!(schematic.add_block_entity(chest.clone()));

        let region = schematic.get_region("Main").unwrap();
        assert_eq!(region.block_entities.len(), 1);
        assert_eq!(region.block_entities.get(&(5, 10, 15)), Some(&chest));

        let removed_block_entity = schematic.remove_block_entity((5, 10, 15)).unwrap();
        assert_eq!(removed_block_entity, chest);

        let region = schematic.get_region("Main").unwrap();
        assert_eq!(region.block_entities.len(), 0);
    }

    #[test]
    fn test_block_entity_helper_operations() {
        let mut schematic = UniversalSchematic::new("Test Schematic".to_string());

        let diamond = ItemStack::new("minecraft:diamond", 64).with_slot(0);
        let chest = BlockEntity::create_chest((5, 10, 15), vec![diamond]);

        assert!(schematic.add_block_entity(chest.clone()));

        let region = schematic.get_region("Main").unwrap();
        assert_eq!(region.block_entities.len(), 1);
        assert_eq!(region.block_entities.get(&(5, 10, 15)), Some(&chest));

        let removed_block_entity = schematic.remove_block_entity((5, 10, 15)).unwrap();
        assert_eq!(removed_block_entity, chest);

        let region = schematic.get_region("Main").unwrap();
        assert_eq!(region.block_entities.len(), 0);
    }

    #[test]
    fn test_block_entity_in_region_operations() {
        let mut schematic = UniversalSchematic::new("Test Schematic".to_string());


        let chest = BlockEntity::create_chest((5, 10, 15), vec![ItemStack::new("minecraft:diamond", 64).with_slot(0)]);
        assert!(schematic.add_block_entity_in_region("Main", chest.clone()));

        let region = schematic.get_region("Main").unwrap();
        assert_eq!(region.block_entities.len(), 1);
        assert_eq!(region.block_entities.get(&(5, 10, 15)), Some(&chest));

        let removed_block_entity = schematic.remove_block_entity_in_region("Main", (5, 10, 15)).unwrap();
        assert_eq!(removed_block_entity, chest);

        let region = schematic.get_region("Main").unwrap();
        assert_eq!(region.block_entities.len(), 0);
    }

    #[test]
    fn test_set_block_from_string() {
        let mut schematic = UniversalSchematic::new("Test".to_string());

        // Test simple block
        assert!(schematic.set_block_from_string(0, 0, 0, "minecraft:stone").unwrap());

        // Test block with properties
        assert!(schematic.set_block_from_string(1, 0, 0, "minecraft:chest[facing=north]").unwrap());

        // Test container with items
        let barrel_str = r#"minecraft:barrel[facing=up]{CustomName:'{"text":"Storage"}',Items:[{Count:64b,Slot:0b,id:"minecraft:redstone"}]}"#;
        assert!(schematic.set_block_from_string(2, 0, 0, barrel_str).unwrap());

        // Verify the blocks were set correctly
        assert_eq!(schematic.get_block(0, 0, 0).unwrap().get_name(), "minecraft:stone");
        assert_eq!(schematic.get_block(1, 0, 0).unwrap().get_name(), "minecraft:chest");
        assert_eq!(schematic.get_block(2, 0, 0).unwrap().get_name(), "minecraft:barrel");

        // Verify container contents
        let barrel_entity = schematic.get_block_entity(BlockPosition { x: 2, y: 0, z: 0 }).unwrap();
        let items = barrel_entity.nbt.get("Items").unwrap();
        if let NbtValue::List(items) = items {
            assert_eq!(items.len(), 1);
            if let NbtValue::Compound(item) = &items[0] {
                assert_eq!(item.get("id").unwrap(), &NbtValue::String("minecraft:redstone".to_string()));
                assert_eq!(item.get("Count").unwrap(), &NbtValue::Byte(64));
                assert_eq!(item.get("Slot").unwrap(), &NbtValue::Byte(0));
            } else {
                panic!("Expected compound NBT value");
            }
        } else {
            panic!("Expected list of items");
        }
    }


    #[test]
    fn test_region_palette_operations() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));

        let stone = BlockState::new("minecraft:stone".to_string());
        let dirt = BlockState::new("minecraft:dirt".to_string());

        region.set_block(0, 0, 0, stone.clone());
        region.set_block(0, 1, 0, dirt.clone());
        region.set_block(1, 0, 0, stone.clone());

        assert_eq!(region.get_block(0, 0, 0), Some(&stone));
        assert_eq!(region.get_block(0, 1, 0), Some(&dirt));
        assert_eq!(region.get_block(1, 0, 0), Some(&stone));
        assert_eq!(region.get_block(1, 1, 1), Some(&BlockState::new("minecraft:air".to_string())));

        // Check the palette size
        assert_eq!(region.palette.len(), 3); // air, stone, dirt
    }

    #[test]
    fn test_nbt_serialization_deserialization() {
        let mut schematic = UniversalSchematic::new("Test Schematic".to_string());

        // Add some blocks and entities
        schematic.set_block(0, 0, 0, BlockState::new("minecraft:stone".to_string()));
        schematic.set_block(1, 1, 1, BlockState::new("minecraft:dirt".to_string()));
        schematic.add_entity(Entity::new("minecraft:creeper".to_string(), (0.5, 0.0, 0.5)));

        // Serialize to NBT
        let nbt = schematic.to_nbt();

        // Write NBT to a buffer
        let mut buffer = Vec::new();
        write_nbt(&mut buffer, None, &nbt, quartz_nbt::io::Flavor::Uncompressed).unwrap();

        // Read NBT from the buffer
        let (read_nbt, _) = read_nbt(&mut Cursor::new(buffer), quartz_nbt::io::Flavor::Uncompressed).unwrap();

        // Deserialize from NBT
        let deserialized_schematic = UniversalSchematic::from_nbt(read_nbt).unwrap();

        // Compare original and deserialized schematics
        assert_eq!(schematic.metadata, deserialized_schematic.metadata);
        assert_eq!(schematic.regions.len(), deserialized_schematic.regions.len());

        // Check if blocks are correctly deserialized
        assert_eq!(schematic.get_block(0, 0, 0), deserialized_schematic.get_block(0, 0, 0));
        assert_eq!(schematic.get_block(1, 1, 1), deserialized_schematic.get_block(1, 1, 1));

        // Check if entities are correctly deserialized
        let original_entities = schematic.get_region("Main").unwrap().entities.clone();
        let deserialized_entities = deserialized_schematic.get_region("Main").unwrap().entities.clone();
        assert_eq!(original_entities, deserialized_entities);

        // Check if palettes are correctly deserialized (now checking the region's palette)
        let original_palette = schematic.get_region("Main").unwrap().get_palette_nbt().clone();
        let deserialized_palette = deserialized_schematic.get_region("Main").unwrap().get_palette_nbt().clone();
        assert_eq!(original_palette, deserialized_palette);
    }


    #[test]
    fn test_multiple_region_merging() {
        let mut schematic = UniversalSchematic::new("Test Schematic".to_string());

        let mut region1 = Region::new("Region1".to_string(), (0, 0, 0), (2, 2, 2));
        let mut region2 = Region::new("Region4".to_string(), (0, 0, 0), (-2, -2, -2));

        // Add some blocks to the regions
        region1.set_block(0, 0, 0, BlockState::new("minecraft:stone".to_string()));
        region1.set_block(1, 1, 1, BlockState::new("minecraft:dirt".to_string()));
        region2.set_block(0, -1, -1, BlockState::new("minecraft:gold_block".to_string()));


        schematic.add_region(region1);
        schematic.add_region(region2);

        let merged_region = schematic.get_merged_region();

        assert_eq!(merged_region.count_blocks(), 3);
        assert_eq!(merged_region.get_block(0, 0, 0), Some(&BlockState::new("minecraft:stone".to_string())));
        assert_eq!(merged_region.get_block(1, 1, 1), Some(&BlockState::new("minecraft:dirt".to_string())));
    }

    #[test]
    fn test_calculate_items_for_signal() {
        assert_eq!(UniversalSchematic::calculate_items_for_signal(0), 0);
        assert_eq!(UniversalSchematic::calculate_items_for_signal(1), 1);
        assert_eq!(UniversalSchematic::calculate_items_for_signal(15), 1728); // Full barrel
    }

    #[test]
    fn test_barrel_signal_strength() {
        let mut schematic = UniversalSchematic::new("Test".to_string());

        // Test simple signal strength
        let barrel_str = "minecraft:barrel{signal=13}";
        assert!(schematic.set_block_from_string(0, 0, 0, barrel_str).unwrap());

        let barrel_entity = schematic.get_block_entity(BlockPosition { x: 0, y: 0, z: 0 }).unwrap();
        let items = barrel_entity.nbt.get("Items").unwrap();

        if let NbtValue::List(items) = items {
            // Calculate expected total items
            let mut total_items = 0;
            for item in items {
                if let NbtValue::Compound(item_map) = item {
                    if let Some(NbtValue::Byte(count)) = item_map.get("Count") {
                        total_items += *count as u32;
                    }
                }
            }

            // Verify the total items matches what's needed for signal strength 13
            let expected_items = UniversalSchematic::calculate_items_for_signal(13);
            assert_eq!(total_items as u32, expected_items);
        }

        // Test invalid signal strength
        let invalid_barrel = "minecraft:barrel{signal=16}";
        assert!(schematic.set_block_from_string(1, 0, 0, invalid_barrel).is_err());
    }

    #[test]
    fn test_barrel_with_properties_and_signal() {
        let mut schematic = UniversalSchematic::new("Test".to_string());

        let barrel_str = "minecraft:barrel[facing=up]{signal=7}";
        assert!(schematic.set_block_from_string(0, 0, 0, barrel_str).unwrap());

        // Verify the block state properties
        let block = schematic.get_block(0, 0, 0).unwrap();
        assert_eq!(block.get_property("facing"), Some(&"up".to_string()));

        // Verify the signal strength items
        let barrel_entity = schematic.get_block_entity(BlockPosition { x: 0, y: 0, z: 0 }).unwrap();
        let items = barrel_entity.nbt.get("Items").unwrap();
        if let NbtValue::List(items) = items {
            let mut total_items = 0;
            for item in items {
                if let NbtValue::Compound(item_map) = item {
                    if let Some(NbtValue::Byte(count)) = item_map.get("Count") {
                        total_items += *count as u32;
                    }
                }
            }
            let expected_items = UniversalSchematic::calculate_items_for_signal(7);
            assert_eq!(total_items as u32, expected_items);
        }
    }
}
