use std::sync::Arc;
use hashbrown::HashMap;
use std::collections::HashMap as StdHashMap;
use quartz_nbt::{NbtCompound, NbtList, NbtTag};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use crate::BlockState;
use crate::block_entity::BlockEntity;
use crate::block_position::BlockPosition;
use crate::bounding_box::BoundingBox;
use crate::entity::Entity;

const SUB: i32 = 16; // sub-chunk edge
type PaletteIndex = u16; // 0 == air
const CHUNK_SIZE: usize = SUB as usize * SUB as usize * SUB as usize; // 4096

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Region {
    pub name: String,
    pub position: (i32, i32, i32),
    pub size: (i32, i32, i32),
    // Private implementation details - not part of public API
    #[serde(skip)]
    pub(crate) chunks: HashMap<(i32, i32, i32), Box<[PaletteIndex; CHUNK_SIZE]>>,
    pub(crate) palette: Vec<BlockState>,
    #[serde(skip)]
    pub(crate) palette_lookup: HashMap<BlockState, PaletteIndex>,
    pub entities: Vec<Entity>,
    #[serde(serialize_with = "serialize_block_entities", deserialize_with = "deserialize_block_entities")]
    pub block_entities: StdHashMap<(i32, i32, i32), BlockEntity>,
}


fn serialize_block_entities<S>(
    block_entities: &StdHashMap<(i32, i32, i32), BlockEntity>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let block_entities_vec: Vec<&BlockEntity> = block_entities.values().collect();
    block_entities_vec.serialize(serializer)
}

fn deserialize_block_entities<'de, D>(
    deserializer: D,
) -> Result<StdHashMap<(i32, i32, i32), BlockEntity>, D::Error>
where
    D: Deserializer<'de>,
{
    let block_entities_vec: Vec<BlockEntity> = Vec::deserialize(deserializer)?;
    Ok(block_entities_vec
        .into_iter()
        .map(|be| {
            let pos = (be.position.0 as i32, be.position.1 as i32, be.position.2 as i32);
            (pos, be)
        })
        .collect())
}

impl Region {
    pub fn new(name: String, position: (i32, i32, i32), size: (i32, i32, i32)) -> Self {
        let bounding_box = BoundingBox::from_position_and_size(position, size);
        let position_and_size = bounding_box.to_position_and_size();
        let mut palette = Vec::new();
        let mut palette_lookup = HashMap::new();

        // Add air as the first block in the palette (index 0)
        palette.push(BlockState::air());
        palette_lookup.insert(BlockState::air(), 0);

        Region {
            name,
            position: position_and_size.0,
            size: position_and_size.1,
            chunks: HashMap::new(),
            palette,
            palette_lookup,
            entities: Vec::new(),
            block_entities: StdHashMap::new(),
        }
    }

    pub fn get_block_entities_as_list(&self) -> Vec<BlockEntity> {
        self.block_entities.values().cloned().collect()
    }

    pub fn is_in_region(&self, x: i32, y: i32, z: i32) -> bool {
        let bounding_box = self.get_bounding_box();
        bounding_box.contains((x, y, z))
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block: BlockState) -> bool {
        if !self.is_in_region(x, y, z) {
            self.expand_to_fit(x, y, z);
        }

        let palette_index = self.get_or_insert_in_palette(block);
        self.set_block_at_index(x, y, z, palette_index);
        true
    }

    pub fn set_block_entity(&mut self, position: BlockPosition, block_entity: BlockEntity) -> bool {
        self.block_entities.insert((position.x, position.y, position.z), block_entity);
        true
    }

    pub fn get_block_entity(&self, position: BlockPosition) -> Option<&BlockEntity> {
        self.block_entities.get(&(position.x, position.y, position.z))
    }

    pub fn get_bounding_box(&self) -> BoundingBox {
        BoundingBox::from_position_and_size(self.position, self.size)
    }

    pub fn index_to_coords(&self, index: usize) -> (i32, i32, i32) {
        self.get_bounding_box().index_to_coords(index)
    }

    pub fn get_dimensions(&self) -> (i32, i32, i32) {
        let bounding_box = self.get_bounding_box();
        bounding_box.get_dimensions()
    }

    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Option<&BlockState> {
        if !self.is_in_region(x, y, z) {
            return None;
        }

        let block_index = self.get_block_index(x, y, z);
        if let Some(idx) = block_index {
            self.palette.get(idx as usize)
        } else {
            // Air block (index 0) if chunk doesn't exist or if index is 0
            Some(&self.palette[0])
        }
    }

    pub fn get_block_index(&self, x: i32, y: i32, z: i32) -> Option<usize> {
        if !self.is_in_region(x, y, z) {
            return None;
        }

        let (chunk_x, chunk_y, chunk_z, idx) = self.get_chunk_coords_and_index(x, y, z);
        let chunk_key = (chunk_x, chunk_y, chunk_z);

        if let Some(chunk) = self.chunks.get(&chunk_key) {
            Some(chunk[idx] as usize)
        } else {
            // If the chunk doesn't exist, it's all air (index 0)
            Some(0)
        }
    }

    pub fn volume(&self) -> usize {
        self.size.0 as usize * self.size.1 as usize * self.size.2 as usize
    }

    pub fn expand_to_fit(&mut self, x: i32, y: i32, z: i32) {
        let current_bounding_box = self.get_bounding_box();
        let fit_position_bounding_box = BoundingBox::new((x, y, z), (x, y, z));
        let new_bounding_box = current_bounding_box.union(&fit_position_bounding_box);
        let new_size = new_bounding_box.get_dimensions();
        let new_position = new_bounding_box.min;

        if new_size == self.size && new_position == self.position {
            return;
        }

        // Just update the position and size - no need to copy chunks
        self.position = new_position;
        self.size = new_size;
    }

    fn calculate_bits_per_block(&self) -> usize {
        let palette_size = self.palette.len();
        let bits_per_block = std::cmp::max((palette_size as f64).log2().ceil() as usize, 2);
        bits_per_block
    }

    pub fn merge(&mut self, other: &Region) {
        let bounding_box = self.get_bounding_box();
        let other_bounding_box = other.get_bounding_box();
        let combined_bounding_box = bounding_box.union(&other_bounding_box);
        let new_size = combined_bounding_box.get_dimensions();
        let new_position = combined_bounding_box.min;

        // Update region properties
        self.position = new_position;
        self.size = new_size;

        // Merge palettes
        let original_palette_size = self.palette.len();
        let mut palette_mapping = HashMap::new();

        for (idx, block) in other.palette.iter().enumerate() {
            if let Some(&existing_idx) = self.palette_lookup.get(block) {
                palette_mapping.insert(idx, existing_idx as usize);
            } else {
                let new_idx = self.palette.len();
                self.palette.push(block.clone());
                self.palette_lookup.insert(block.clone(), new_idx as PaletteIndex);
                palette_mapping.insert(idx, new_idx);
            }
        }

        // Copy blocks from other region
        for (x, y, z) in other_bounding_box.iter_coords() {
            if let Some(&idx) = other.get_block_index(x, y, z).as_ref() {
                if idx != 0 { // Skip air blocks
                    let mapped_idx = palette_mapping[&idx];
                    self.set_block_at_index(x, y, z, mapped_idx as PaletteIndex);
                }
            }
        }

        // Merge entities and block entities
        self.merge_entities(other);
        self.merge_block_entities(other);
    }

    fn merge_entities(&mut self, other: &Region) {
        self.entities.extend(other.entities.iter().cloned());
    }

    fn merge_block_entities(&mut self, other: &Region) {
        for (&pos, be) in &other.block_entities {
            self.block_entities.insert(pos, be.clone());
        }
    }

    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    pub fn remove_entity(&mut self, index: usize) -> Option<Entity> {
        if index < self.entities.len() {
            Some(self.entities.remove(index))
        } else {
            None
        }
    }

    pub fn add_block_entity(&mut self, block_entity: BlockEntity) {
        self.block_entities.insert(block_entity.position, block_entity);
    }

    pub fn remove_block_entity(&mut self, position: (i32, i32, i32)) -> Option<BlockEntity> {
        self.block_entities.remove(&position)
    }

    pub fn to_nbt(&self) -> NbtTag {
        let mut tag = NbtCompound::new();
        tag.insert("Name", NbtTag::String(self.name.clone()));
        tag.insert("Position", NbtTag::IntArray(vec![self.position.0, self.position.1, self.position.2]));
        tag.insert("Size", NbtTag::IntArray(vec![self.size.0, self.size.1, self.size.2]));

        // Convert sparse chunks to full block data for NBT
        let mut blocks_tag = NbtCompound::new();

        // Iterate over all possible coordinates in the bounding box
        let bounding_box = self.get_bounding_box();

        // Iterate through all coordinates in the bounding box
        for (x, y, z) in bounding_box.iter_coords() {
            let idx = match self.get_block_index(x, y, z) {
                Some(idx) => idx as i32,
                None => 0 // Air
            };

            // Only include non-air blocks to save space
            if idx != 0 {
                blocks_tag.insert(&format!("{},{},{}", x, y, z), NbtTag::Int(idx));
            }
        }

        tag.insert("Blocks", NbtTag::Compound(blocks_tag));

        // Add palette list
        let mut palette_list = NbtList::new();
        for block in &self.palette {
            palette_list.push(block.to_nbt());
        }
        tag.insert("Palette", NbtTag::List(palette_list));

        // Add entities list
        let mut entities_list = NbtList::new();
        for entity in &self.entities {
            entities_list.push(entity.to_nbt());
        }
        tag.insert("Entities", NbtTag::List(entities_list));

        // Add block entities
        let mut block_entities_tag = NbtCompound::new();
        for ((x, y, z), block_entity) in &self.block_entities {
            block_entities_tag.insert(&format!("{},{},{}", x, y, z), block_entity.to_nbt());
        }
        tag.insert("BlockEntities", NbtTag::Compound(block_entities_tag));

        NbtTag::Compound(tag)
    }

    pub fn from_nbt(nbt: &NbtCompound) -> Result<Self, String> {
        let name = nbt.get::<_, &str>("Name")
            .map_err(|e| format!("Failed to get Region Name: {}", e))?
            .to_string();

        let position = match nbt.get::<_, &NbtTag>("Position") {
            Ok(NbtTag::IntArray(arr)) if arr.len() == 3 => (arr[0], arr[1], arr[2]),
            _ => return Err("Invalid Position tag".to_string()),
        };

        let size = match nbt.get::<_, &NbtTag>("Size") {
            Ok(NbtTag::IntArray(arr)) if arr.len() == 3 => (arr[0], arr[1], arr[2]),
            _ => return Err("Invalid Size tag".to_string()),
        };

        let palette_tag = nbt.get::<_, &NbtList>("Palette")
            .map_err(|e| format!("Failed to get Palette: {}", e))?;

        let mut palette = Vec::new();
        for tag in palette_tag.iter() {
            if let NbtTag::Compound(compound) = tag {
                if let Ok(block_state) = BlockState::from_nbt(compound) {
                    palette.push(block_state);
                }
            }
        }

        // Create the region with the correct size
        let mut region = Region::new(name, position, size);
        region.palette = palette;

        // Rebuild the palette lookup
        region.palette_lookup.clear();
        for (idx, block) in region.palette.iter().enumerate() {
            region.palette_lookup.insert(block.clone(), idx as PaletteIndex);
        }

        // Load blocks
        let blocks_tag = nbt.get::<_, &NbtCompound>("Blocks")
            .map_err(|e| format!("Failed to get Blocks: {}", e))?;

        for (key, value) in blocks_tag.inner() {
            if let NbtTag::Int(index) = value {
                let coords: Vec<i32> = key.split(',')
                    .map(|s| s.parse::<i32>().unwrap_or(0))
                    .collect();
                if coords.len() == 3 {
                    let (x, y, z) = (coords[0], coords[1], coords[2]);
                    region.set_block_at_index(x, y, z, *index as PaletteIndex);
                }
            }
        }

        // Load entities
        let entities_tag = nbt.get::<_, &NbtList>("Entities")
            .map_err(|e| format!("Failed to get Entities: {}", e))?;

        let mut entities = Vec::new();
        for tag in entities_tag.iter() {
            if let NbtTag::Compound(compound) = tag {
                if let Ok(entity) = Entity::from_nbt(compound) {
                    entities.push(entity);
                }
            }
        }
        region.entities = entities;

        // Load block entities
        let block_entities_tag = nbt.get::<_, &NbtCompound>("BlockEntities")
            .map_err(|e| format!("Failed to get BlockEntities: {}", e))?;

        let mut block_entities = StdHashMap::new();
        for (key, value) in block_entities_tag.inner() {
            if let NbtTag::Compound(be_compound) = value {
                let coords: Vec<i32> = key.split(',')
                    .map(|s| s.parse::<i32>().unwrap_or(0))
                    .collect();
                if coords.len() == 3 {
                    let block_entity = BlockEntity::from_nbt(be_compound) ;
                    block_entities.insert((coords[0], coords[1], coords[2]), block_entity);
                }
            }
        }

        region.block_entities = block_entities;

        Ok(region)
    }
    pub fn to_litematic_nbt(&self) -> NbtCompound {
        let mut region_nbt = NbtCompound::new();

        // 1. Position and Size
        region_nbt.insert("Position", NbtTag::IntArray(vec![self.position.0, self.position.1, self.position.2]));
        region_nbt.insert("Size", NbtTag::IntArray(vec![self.size.0, self.size.1, self.size.2]));

        // 2. BlockStatePalette
        let mut palette_list = NbtList::new();
        for block_state in &self.palette {
            palette_list.push(block_state.to_nbt());
        }
        region_nbt.insert("BlockStatePalette", NbtTag::List(palette_list));

        // 3. BlockStates (packed long array)
        let block_states = self.create_packed_block_states();
        region_nbt.insert("BlockStates", NbtTag::LongArray(block_states));

        // 4. Entities
        let mut entities_list = NbtList::new();
        for entity in &self.entities {
            entities_list.push(entity.to_nbt());
        }
        region_nbt.insert("Entities", NbtTag::List(entities_list));

        // 5. TileEntities
        let mut tile_entities_list = NbtList::new();
        for be in self.block_entities.values() {
            tile_entities_list.push(be.to_nbt());
        }
        region_nbt.insert("TileEntities", NbtTag::List(tile_entities_list));

        region_nbt
    }

    pub fn create_packed_block_states(&self) -> Vec<i64> {
        let bits_per_block = self.calculate_bits_per_block();
        let volume = self.volume();
        let expected_len = (volume * bits_per_block + 63) / 64; // Equivalent to ceil(volume * bits_per_block / 64)

        let mut packed_states = vec![0i64; expected_len];
        let mask = (1i64 << bits_per_block) - 1;

        // Iterate through all positions in the region
        let bounding_box = self.get_bounding_box();
        for (index, (x, y, z)) in bounding_box.iter_coords().enumerate() {
            let block_state_idx = self.get_block_index(x, y, z).unwrap_or(0) as i64 & mask;

            let bit_index = index * bits_per_block;
            let start_long_index = bit_index / 64;
            let end_long_index = (bit_index + bits_per_block - 1) / 64;
            let start_offset = bit_index % 64;

            if start_long_index == end_long_index {
                packed_states[start_long_index] |= block_state_idx << start_offset;
            } else {
                packed_states[start_long_index] |= block_state_idx << start_offset;
                packed_states[end_long_index] |= block_state_idx >> (64 - start_offset);
            }
        }

        // Handle negative numbers (convert from unsigned to signed)
        packed_states.iter_mut().for_each(|x| *x = *x as u64 as i64);

        packed_states
    }

    pub fn unpack_block_states(&self, packed_states: &[i64]) -> Vec<usize> {
        let bits_per_block = self.calculate_bits_per_block();
        let mask = (1 << bits_per_block) - 1;
        let volume = self.volume();

        let mut blocks = Vec::with_capacity(volume);

        for index in 0..volume {
            let bit_index = index * bits_per_block;
            let start_long_index = bit_index / 64;
            let start_offset = bit_index % 64;

            let value = if start_offset + bits_per_block <= 64 {
                // Block is entirely within one long
                ((packed_states[start_long_index] as u64) >> start_offset) & (mask as u64)
            } else {
                // Block spans two longs
                let low_bits = ((packed_states[start_long_index] as u64) >> start_offset) & ((1 << (64 - start_offset)) - 1);
                let high_bits = (packed_states[start_long_index + 1] as u64) & ((1 << (bits_per_block - (64 - start_offset))) - 1);
                low_bits | (high_bits << (64 - start_offset))
            };

            blocks.push(value as usize);
        }

        blocks
    }

    pub fn get_palette(&self) -> Vec<BlockState> {
        self.palette.clone()
    }

    pub(crate) fn get_palette_nbt(&self) -> NbtList {
        let mut palette = NbtList::new();
        for block in &self.palette {
            palette.push(block.to_nbt());
        }
        palette
    }

    pub fn count_block_types(&self) -> HashMap<BlockState, usize> {
        let mut block_counts = HashMap::new();

        // Iterate through all blocks in all chunks
        let bounding_box = self.get_bounding_box();
        for (x, y, z) in bounding_box.iter_coords() {
            let idx = match self.get_block_index(x, y, z) {
                Some(idx) => idx,
                None => 0 // Air
            };

            let block_state = &self.palette[idx];
            *block_counts.entry(block_state.clone()).or_insert(0) += 1;
        }

        block_counts
    }

    pub fn count_blocks(&self) -> usize {
        let mut count = 0;

        // Iterate through all chunks
        for chunk in self.chunks.values() {
            // Count non-air blocks in this chunk
            count += chunk.iter().filter(|&&idx| idx != 0).count();
        }

        count
    }

    pub fn get_palette_index(&self, block: &BlockState) -> Option<usize> {
        self.palette_lookup.get(block).map(|&idx| idx as usize)
    }

    // Private helper methods

    fn get_or_insert_in_palette(&mut self, block: BlockState) -> PaletteIndex {
        if let Some(&index) = self.palette_lookup.get(&block) {
            index
        } else {
            let index = self.palette.len() as PaletteIndex;
            self.palette.push(block.clone());
            self.palette_lookup.insert(block, index);
            index
        }
    }

    pub(crate) fn get_chunk_coords_and_index(&self, x: i32, y: i32, z: i32) -> (i32, i32, i32, usize) {
        // Calculate chunk coordinates
        let chunk_x = x.div_euclid(SUB);
        let chunk_y = y.div_euclid(SUB);
        let chunk_z = z.div_euclid(SUB);

        // Calculate local coordinates within the chunk
        let local_x = x.rem_euclid(SUB) as usize;
        let local_y = y.rem_euclid(SUB) as usize;
        let local_z = z.rem_euclid(SUB) as usize;

        // Calculate index within the chunk
        let idx = (local_y * SUB as usize * SUB as usize) + (local_z * SUB as usize) + local_x;

        (chunk_x, chunk_y, chunk_z, idx)
    }

    pub(crate) fn set_block_at_index(&mut self, x: i32, y: i32, z: i32, palette_index: PaletteIndex) {
        let (chunk_x, chunk_y, chunk_z, idx) = self.get_chunk_coords_and_index(x, y, z);
        let chunk_key = (chunk_x, chunk_y, chunk_z);

        // Only allocate a chunk if we're setting a non-air block
        if palette_index == 0 {
            // If we're setting air and the chunk doesn't exist, we don't need to do anything
            if !self.chunks.contains_key(&chunk_key) {
                return;
            }
        }

        // Get or create the chunk
        let chunk = self.chunks.entry(chunk_key).or_insert_with(|| {
            // Initialize a new chunk with all air blocks (index 0)
            Box::new([0; CHUNK_SIZE])
        });

        // Set the block
        chunk[idx] = palette_index;

        // If the entire chunk is now air, remove it to save memory
        if palette_index == 0 && chunk.iter().all(|&idx| idx == 0) {
            self.chunks.remove(&chunk_key);
        }
    }

    // For coordinates to index conversion needed by the old API
    fn coords_to_index(&self, x: i32, y: i32, z: i32) -> usize {
        self.get_bounding_box().coords_to_index(x, y, z)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use hashbrown::HashMap;
    use std::collections::HashMap as StdHashMap;
    use crate::bounding_box::BoundingBox;
    use crate::block_entity::BlockEntity;
    use crate::block_position::BlockPosition;
    use crate::entity::Entity;
    use crate::BlockState;
    use crate::region::Region;

    // Helper functions for tests
    fn create_block_state(name: &str) -> BlockState {
        BlockState::new(name)
    }

    fn create_block_with_property(name: &str, key: &str, value: &str) -> BlockState {
        BlockState::new(name).with_prop(key, value)
    }

    // BlockState tests
    #[test]
    fn test_block_state_creation() {
        let block = create_block_state("minecraft:stone");
        assert_eq!(block.name.as_ref(), "minecraft:stone");
        assert!(block.properties.is_empty());
    }

    #[test]
    fn test_block_state_with_properties() {
        let block = create_block_with_property("minecraft:stone", "variant", "granite");
        assert_eq!(block.name.as_ref(), "minecraft:stone");
        assert_eq!(block.properties.len(), 1);
        assert_eq!(block.get_property("variant").unwrap().as_ref(), "granite");
    }

    #[test]
    fn test_block_state_display() {
        let block = create_block_state("minecraft:stone");
        assert_eq!(block.to_string(), "minecraft:stone");

        let block_with_prop = create_block_with_property("minecraft:stone", "variant", "granite");
        assert_eq!(block_with_prop.to_string(), "minecraft:stone[variant=granite]");

        let mut block_with_props = create_block_with_property("minecraft:stone", "variant", "granite");
        block_with_props.set_property("color", "red");
        // The order of properties should be deterministic (sorted by key)
        assert_eq!(block_with_props.to_string(), "minecraft:stone[color=red,variant=granite]");
    }

    #[test]
    fn test_block_state_air() {
        let air = BlockState::air();
        assert_eq!(air.name.as_ref(), "minecraft:air");
        assert!(air.properties.is_empty());
    }

    // Region tests
    #[test]
    fn test_region_creation() {
        let region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        assert_eq!(region.name, "Test");
        assert_eq!(region.position, (0, 0, 0));
        assert_eq!(region.size, (2, 2, 2));
        assert_eq!(region.palette.len(), 1);
        assert_eq!(region.palette[0].name.as_ref(), "minecraft:air");
    }

    #[test]
    fn test_set_and_get_block() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        let stone = create_block_state("minecraft:stone");

        assert!(region.set_block(0, 0, 0, stone.clone()));
        assert_eq!(region.get_block(0, 0, 0).unwrap().name.as_ref(), "minecraft:stone");
        assert_eq!(region.get_block(1, 1, 1).unwrap().name.as_ref(), "minecraft:air");
        assert_eq!(region.get_block(2, 2, 2), None);
    }

    #[test]
    fn test_pack_block_states_to_long_array() {
        // Create blocks array from 1 to 16
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (16, 1, 1));

        // Add blocks to the palette (0 is already air)
        for i in 1..=16 {
            let block = create_block_state(&format!("minecraft:wool{}", i));
            region.set_block(i-1, 0, 0, block);
        }

        // Create packed states
        let packed_states = region.create_packed_block_states();

        // 16 blocks with 5 bits each (since we have 17 palette entries including air)
        // needs 2 longs
        assert_eq!(packed_states.len(), 2);

        // The expected values for this specific test case
        assert_eq!(packed_states, vec![-3013672028691362751, 33756]);

        // Unpack and check
        let unpacked_blocks = region.unpack_block_states(&packed_states);

        // Verify the first 16 positions match
        for i in 0..16 {
            assert_eq!(unpacked_blocks[i], i + 1);
        }
    }

    #[test]
    fn test_expand_to_fit() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        let stone = create_block_state("minecraft:stone");

        region.set_block(0, 0, 0, stone.clone());
        let new_size = (3, 3, 3);
        region.expand_to_fit(new_size.0, new_size.1, new_size.2);

        // Check if the original block is preserved
        assert_eq!(region.get_block(0, 0, 0).unwrap().name.as_ref(), "minecraft:stone");

        // Check if the new coordinates are in the region and contain air
        assert_eq!(region.get_block(3, 3, 3).unwrap().name.as_ref(), "minecraft:air");
    }

    #[test]
    fn test_entities() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        let entity = Entity::new("minecraft:creeper".to_string(), (0.5, 0.0, 0.5));

        region.add_entity(entity.clone());
        assert_eq!(region.entities.len(), 1);

        let removed = region.remove_entity(0);
        assert_eq!(removed, Some(entity));
        assert_eq!(region.entities.len(), 0);
    }

    #[test]
    fn test_block_entities() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        let pos = (0, 0, 0);
        let block_entity = BlockEntity::new("minecraft:chest".to_string(), pos);

        region.add_block_entity(block_entity.clone());
        assert_eq!(region.block_entities.len(), 1);

        let removed = region.remove_block_entity(pos);
        assert_eq!(removed, Some(block_entity));
        assert_eq!(region.block_entities.len(), 0);
    }

    #[test]
    fn test_to_and_from_nbt() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        let stone = create_block_state("minecraft:stone");
        region.set_block(0, 0, 0, stone.clone());

        let nbt = region.to_nbt();
        if let quartz_nbt::NbtTag::Compound(compound) = nbt {
            let deserialized_region = Region::from_nbt(&compound).unwrap();

            assert_eq!(region.name, deserialized_region.name);
            assert_eq!(region.position, deserialized_region.position);
            assert_eq!(region.size, deserialized_region.size);
            assert_eq!(
                region.get_block(0, 0, 0).unwrap().name.as_ref(),
                deserialized_region.get_block(0, 0, 0).unwrap().name.as_ref()
            );
        } else {
            panic!("Expected NbtTag::Compound");
        }
    }

    #[test]
    fn test_to_litematic_nbt() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        let stone = create_block_state("minecraft:stone");
        region.set_block(0, 0, 0, stone.clone());

        let nbt = region.to_litematic_nbt();

        assert!(nbt.contains_key("Position"));
        assert!(nbt.contains_key("Size"));
        assert!(nbt.contains_key("BlockStatePalette"));
        assert!(nbt.contains_key("BlockStates"));
        assert!(nbt.contains_key("Entities"));
        assert!(nbt.contains_key("TileEntities"));
    }

    #[test]
    fn test_count_blocks() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        let stone = create_block_state("minecraft:stone");

        assert_eq!(region.count_blocks(), 0);

        region.set_block(0, 0, 0, stone.clone());
        region.set_block(1, 1, 1, stone.clone());

        assert_eq!(region.count_blocks(), 2);
    }

    #[test]
    fn test_region_merge() {
        let mut region1 = Region::new("Test1".to_string(), (0, 0, 0), (2, 2, 2));
        let mut region2 = Region::new("Test2".to_string(), (2, 2, 2), (2, 2, 2));
        let stone = create_block_state("minecraft:stone");

        region1.set_block(0, 0, 0, stone.clone());
        region2.set_block(2, 2, 2, stone.clone());

        region1.merge(&region2);

        assert_eq!(region1.size, (4, 4, 4));
        assert_eq!(region1.get_block(0, 0, 0).unwrap().name.as_ref(), "minecraft:stone");
        assert_eq!(region1.get_block(2, 2, 2).unwrap().name.as_ref(), "minecraft:stone");
    }

    #[test]
    fn test_region_merge_different_palettes() {
        let mut region1 = Region::new("Test1".to_string(), (0, 0, 0), (2, 2, 2));
        let mut region2 = Region::new("Test2".to_string(), (2, 2, 2), (2, 2, 2));
        let stone = create_block_state("minecraft:stone");
        let dirt = create_block_state("minecraft:dirt");

        region1.set_block(0, 0, 0, stone.clone());
        region2.set_block(2, 2, 2, dirt.clone());

        region1.merge(&region2);

        assert_eq!(region1.size, (4, 4, 4));
        assert_eq!(region1.get_block(0, 0, 0).unwrap().name.as_ref(), "minecraft:stone");
        assert_eq!(region1.get_block(2, 2, 2).unwrap().name.as_ref(), "minecraft:dirt");
    }

    #[test]
    fn test_region_merge_different_overlapping_palettes() {
        let mut region1 = Region::new("Test1".to_string(), (0, 0, 0), (2, 2, 2));
        let mut region2 = Region::new("Test2".to_string(), (1, 1, 1), (2, 2, 2));
        let stone = create_block_state("minecraft:stone");
        let dirt = create_block_state("minecraft:dirt");

        region1.set_block(0, 0, 0, stone.clone());
        region1.set_block(1, 1, 1, dirt.clone());

        region2.set_block(2, 2, 2, dirt.clone());

        region1.merge(&region2);

        assert_eq!(region1.size, (3, 3, 3));
        assert_eq!(region1.get_block(0, 0, 0).unwrap().name.as_ref(), "minecraft:stone");
        assert_eq!(region1.get_block(1, 1, 1).unwrap().name.as_ref(), "minecraft:dirt");
        assert_eq!(region1.get_block(2, 2, 2).unwrap().name.as_ref(), "minecraft:dirt");
    }

    #[test]
    fn test_expand_to_fit_single_block() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        let stone = create_block_state("minecraft:stone");

        // Place a block at the farthest corner to trigger resizing
        region.set_block(3, 3, 3, stone.clone());

        assert_eq!(region.position, (0, 0, 0));
        assert_eq!(region.get_block(3, 3, 3).unwrap().name.as_ref(), "minecraft:stone");
        assert_eq!(region.get_block(0, 0, 0).unwrap().name.as_ref(), "minecraft:air");
    }

    #[test]
    fn test_expand_to_fit_negative_coordinates() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        let dirt = create_block_state("minecraft:dirt");

        // Place a block at a negative coordinate to trigger resizing
        region.set_block(-1, -1, -1, dirt.clone());

        assert_eq!(region.position, (-1, -1, -1)); // Expect region to shift
        assert_eq!(region.get_block(-1, -1, -1).unwrap().name.as_ref(), "minecraft:dirt");
        assert_eq!(region.get_block(0, 0, 0).unwrap().name.as_ref(), "minecraft:air");
    }

    #[test]
    fn test_expand_to_fit_large_positive_coordinates() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        let stone = create_block_state("minecraft:stone");

        // Place a block far away to trigger significant resizing
        region.set_block(10, 10, 10, stone.clone());

        assert_eq!(region.position, (0, 0, 0));
        assert_eq!(region.get_block(10, 10, 10).unwrap().name.as_ref(), "minecraft:stone");
    }

    #[test]
    fn test_expand_to_fit_corner_to_corner() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        let stone = create_block_state("minecraft:stone");
        let dirt = create_block_state("minecraft:dirt");

        // Place a block at one corner
        region.set_block(0, 0, 0, stone.clone());

        // Place another block far from the first to trigger resizing
        region.set_block(4, 4, 4, dirt.clone());

        assert_eq!(region.get_block(0, 0, 0).unwrap().name.as_ref(), "minecraft:stone");
        assert_eq!(region.get_block(4, 4, 4).unwrap().name.as_ref(), "minecraft:dirt");
    }

    #[test]
    fn test_expand_to_fit_multiple_expansions() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        let stone = create_block_state("minecraft:stone");

        // Perform multiple expansions
        region.set_block(3, 3, 3, stone.clone());
        region.set_block(7, 7, 7, stone.clone());
        region.set_block(-2, -2, -2, stone.clone());

        assert_eq!(region.position, (-2, -2, -2));  // Position should shift
        assert_eq!(region.get_block(3, 3, 3).unwrap().name.as_ref(), "minecraft:stone");
        assert_eq!(region.get_block(7, 7, 7).unwrap().name.as_ref(), "minecraft:stone");
        assert_eq!(region.get_block(-2, -2, -2).unwrap().name.as_ref(), "minecraft:stone");
    }

    #[test]
    fn test_expand_to_fit_with_existing_blocks() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (3, 3, 3));
        let stone = create_block_state("minecraft:stone");
        let dirt = create_block_state("minecraft:dirt");

        // Place blocks in the initial region
        region.set_block(0, 0, 0, stone.clone());
        region.set_block(2, 2, 2, dirt.clone());

        // Trigger expansion
        region.set_block(5, 5, 5, stone.clone());

        assert_eq!(region.get_block(0, 0, 0).unwrap().name.as_ref(), "minecraft:stone");
        assert_eq!(region.get_block(2, 2, 2).unwrap().name.as_ref(), "minecraft:dirt");
        assert_eq!(region.get_block(5, 5, 5).unwrap().name.as_ref(), "minecraft:stone");
    }

    #[test]
    fn test_incremental_expansion_in_x() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        let stone = create_block_state("minecraft:stone");

        for x in 0..32 {
            region.set_block(x, 0, 0, stone.clone());
            assert_eq!(region.get_block(x, 0, 0).unwrap().name.as_ref(), "minecraft:stone");
        }
    }

    #[test]
    fn test_incremental_expansion_in_y() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        let stone = create_block_state("minecraft:stone");

        for y in 0..32 {
            region.set_block(0, y, 0, stone.clone());
            assert_eq!(region.get_block(0, y, 0).unwrap().name.as_ref(), "minecraft:stone");
        }
    }

    #[test]
    fn test_incremental_expansion_in_z() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        let stone = create_block_state("minecraft:stone");

        for z in 0..32 {
            region.set_block(0, 0, z, stone.clone());
            assert_eq!(region.get_block(0, 0, z).unwrap().name.as_ref(), "minecraft:stone");
        }
    }

    #[test]
    fn test_incremental_expansion_in_x_y_z() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        let stone = create_block_state("minecraft:stone");

        for i in 0..32 {
            region.set_block(i, i, i, stone.clone());
            assert_eq!(region.get_block(i, i, i).unwrap().name.as_ref(), "minecraft:stone");
        }
    }

    #[test]
    fn test_checkerboard_expansion() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        let stone = create_block_state("minecraft:stone");
        let dirt = create_block_state("minecraft:dirt");

        // Only create an 8³ checkerboard to keep test time reasonable
        for x in 0..8 {
            for y in 0..8 {
                for z in 0..8 {
                    if (x + y + z) % 2 == 0 {
                        region.set_block(x, y, z, stone.clone());
                    } else {
                        region.set_block(x, y, z, dirt.clone());
                    }
                }
            }
        }

        for x in 0..8 {
            for y in 0..8 {
                for z in 0..8 {
                    let expected_name = if (x + y + z) % 2 == 0 {
                        "minecraft:stone"
                    } else {
                        "minecraft:dirt"
                    };
                    assert_eq!(region.get_block(x, y, z).unwrap().name.as_ref(), expected_name);
                }
            }
        }
    }

    #[test]
    fn test_bounding_box() {
        let region = Region::new("Test".to_string(), (1, 0, 1), (-2, 2, -2));
        let bounding_box = region.get_bounding_box();

        assert_eq!(bounding_box.min, (0, 0, 0));
        assert_eq!(bounding_box.max, (1, 1, 1));

        let region = Region::new("Test".to_string(), (1, 0, 1), (-3, 3, -3));
        let bounding_box = region.get_bounding_box();

        assert_eq!(bounding_box.min, (-1, 0, -1));
        assert_eq!(bounding_box.max, (1, 2, 1));
    }

    #[test]
    fn test_coords_to_index() {
        let region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));

        // Get the volume
        let volume = region.volume();

        // Test all coordinates in the region
        for i in 0..volume {
            let coords = region.index_to_coords(i);
            let bb = region.get_bounding_box();
            let index = bb.coords_to_index(coords.0, coords.1, coords.2);
            assert_eq!(index, i);
        }
    }

    #[test]
    fn test_merge_negative_size() {
        let mut region1 = Region::new("Test1".to_string(), (0, 0, 0), (-2, -2, -2));
        let mut region2 = Region::new("Test2".to_string(), (-2, -2, -2), (-2, -2, -2));
        let stone = create_block_state("minecraft:stone");

        region1.set_block(0, 0, 0, stone.clone());
        region2.set_block(-2, -2, -2, stone.clone());

        region1.merge(&region2);

        // Check bounding box
        let bb = region1.get_bounding_box();
        assert_eq!(bb.min, (-3, -3, -3));
        assert_eq!(bb.max, (0, 0, 0));

        // Check blocks were preserved
        assert_eq!(region1.get_block(0, 0, 0).unwrap().name.as_ref(), "minecraft:stone");
        assert_eq!(region1.get_block(-2, -2, -2).unwrap().name.as_ref(), "minecraft:stone");
    }

    #[test]
    fn test_expand_to_fit_preserve_blocks() {
        let mut region = Region::new("Test".to_string(), (1, 0, 1), (-2, 2, -2));
        let stone = create_block_state("minecraft:stone");
        let diamond = create_block_state("minecraft:diamond_block");

        // Set some initial blocks
        region.set_block(1, 0, 1, stone.clone());
        region.set_block(0, 1, 0, stone.clone());

        // Expand the region by setting a block outside the current bounds
        region.set_block(1, 2, 1, diamond.clone());

        // Check if the original blocks are preserved
        assert_eq!(region.get_block(1, 0, 1).unwrap().name.as_ref(), "minecraft:stone");
        assert_eq!(region.get_block(0, 1, 0).unwrap().name.as_ref(), "minecraft:stone");

        // Check if the new block is set correctly
        assert_eq!(region.get_block(1, 2, 1).unwrap().name.as_ref(), "minecraft:diamond_block");
    }

    #[test]
    fn test_sparse_storage() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (100, 100, 100));
        let stone = create_block_state("minecraft:stone");

        // Only set a few blocks in a large region
        region.set_block(0, 0, 0, stone.clone());
        region.set_block(99, 99, 99, stone.clone());

        // Check that we can still get the blocks
        assert_eq!(region.get_block(0, 0, 0).unwrap().name.as_ref(), "minecraft:stone");
        assert_eq!(region.get_block(99, 99, 99).unwrap().name.as_ref(), "minecraft:stone");
        assert_eq!(region.get_block(50, 50, 50).unwrap().name.as_ref(), "minecraft:air");

        // Count should be accurate
        assert_eq!(region.count_blocks(), 2);
    }

    #[test]
    fn test_chunk_allocation() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (100, 100, 100));
        let stone = create_block_state("minecraft:stone");

        // Set and immediately remove a block
        region.set_block(1, 1, 1, stone.clone());
        region.set_block(1, 1, 1, BlockState::air());

        // Count should be 0
        assert_eq!(region.count_blocks(), 0);
    }
}