use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use quartz_nbt::{NbtCompound, NbtTag};
use serde::{Deserialize, Serialize};
use blockpedia::{
    BlockState as BlockpediaBlockState, 
    get_block,
    color::ExtendedColorData,
    transforms::BlockShape,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockState {
    pub name: String,
    pub properties: HashMap<String, String>,
}

impl fmt::Display for BlockState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if !self.properties.is_empty() {
            write!(f, "[")?;
            for (i, (key, value)) in self.properties.iter().enumerate() {
                if i > 0 {
                    write!(f, ",")?;
                }
                write!(f, "{}={}", key, value)?;
            }
            write!(f, "]")?;
        }
        Ok(())
    }
}

impl Hash for BlockState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        for (k, v) in &self.properties {
            k.hash(state);
            v.hash(state);
        }
    }
}

impl BlockState {
    pub fn new(name: String) -> Self {
        BlockState {
            name,
            properties: HashMap::new(),
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn with_property(mut self, key: String, value: String) -> Self {
        self.properties.insert(key, value);
        self
    }

    pub fn with_properties(mut self, properties: HashMap<String, String>) -> Self {
        self.properties = properties;
        self
    }

    pub fn set_property(&mut self, key: String, value: String) {
        self.properties.insert(key, value);
    }

    pub fn remove_property(&mut self, key: &str) {
        self.properties.remove(key);
    }

    pub fn get_property(&self, key: &str) -> Option<&String> {
        self.properties.get(key)
    }
    pub fn to_nbt(&self) -> NbtTag {
        let mut compound = NbtCompound::new();
        compound.insert("Name", self.name.clone());

        if !self.properties.is_empty() {
            let mut properties = NbtCompound::new();
            for (key, value) in &self.properties {
                properties.insert(key, value.clone());
            }
            compound.insert("Properties", properties);
        }

        NbtTag::Compound(compound)
    }

    pub fn from_nbt(compound: &NbtCompound) -> Result<Self, String> {
        let name = compound
            .get::<_, &String>("Name")
            .map_err(|e| format!("Failed to get Name: {}", e))?
            .clone();

        let mut properties = HashMap::new();
        if let Ok(props) = compound.get::<_, &NbtCompound>("Properties") {
            for (key, value) in props.inner() {
                if let NbtTag::String(value_str) = value {
                    properties.insert(key.clone(), value_str.clone());
                }
            }
        }

        Ok(BlockState { name, properties })
    }

    // === Blockpedia Integration ===

    /// Get the blockpedia BlockFacts for this block
    pub fn get_facts(&self) -> Option<&'static blockpedia::BlockFacts> {
        get_block(&self.name)
    }

    /// Get the color information for this block
    pub fn get_color(&self) -> Option<ExtendedColorData> {
        self.get_facts()
            .and_then(|facts| facts.extras.color)
            .map(|color| color.to_extended())
    }

    /// Check if this block has a specific property
    pub fn has_property(&self, property: &str) -> bool {
        self.properties.contains_key(property)
    }

    /// Convert to blockpedia BlockState for transformations
    pub fn to_blockpedia(&self) -> Result<BlockpediaBlockState, String> {
        let mut blockpedia_block = BlockpediaBlockState::new(&self.name)
            .map_err(|e| e.to_string())?;

        for (key, value) in &self.properties {
            blockpedia_block = blockpedia_block.with(key, value)
                .map_err(|e| e.to_string())?;
        }

        Ok(blockpedia_block)
    }

    /// Create from blockpedia BlockState
    pub fn from_blockpedia(blockpedia_block: &BlockpediaBlockState) -> Self {
        let block_string = blockpedia_block.to_string();
        
        if let Some(bracket_pos) = block_string.find('[') {
            let name = block_string[..bracket_pos].to_string();
            let properties_str = &block_string[bracket_pos + 1..block_string.len() - 1];
            
            let mut properties = HashMap::new();
            if !properties_str.is_empty() {
                for prop_pair in properties_str.split(',') {
                    let parts: Vec<&str> = prop_pair.split('=').collect();
                    if parts.len() == 2 {
                        properties.insert(parts[0].trim().to_string(), parts[1].trim().to_string());
                    }
                }
            }
            
            BlockState { name, properties }
        } else {
            BlockState {
                name: block_string,
                properties: HashMap::new(),
            }
        }
    }

    // === Block Transformations ===

    /// Rotate this block state by 90 degrees clockwise
    pub fn rotate_clockwise(&self) -> Result<BlockState, String> {
        let blockpedia_block = self.to_blockpedia()?;
        let rotated = blockpedia_block.rotate_clockwise()
            .map_err(|e| e.to_string())?;
        Ok(Self::from_blockpedia(&rotated))
    }

    /// Rotate this block state by 180 degrees
    pub fn rotate_180(&self) -> Result<BlockState, String> {
        let blockpedia_block = self.to_blockpedia()?;
        let rotated = blockpedia_block.rotate_180()
            .map_err(|e| e.to_string())?;
        Ok(Self::from_blockpedia(&rotated))
    }

    /// Rotate this block state by 270 degrees clockwise (90 counter-clockwise)
    pub fn rotate_counter_clockwise(&self) -> Result<BlockState, String> {
        let blockpedia_block = self.to_blockpedia()?;
        let rotated = blockpedia_block.rotate_counter_clockwise()
            .map_err(|e| e.to_string())?;
        Ok(Self::from_blockpedia(&rotated))
    }

    /// Get a material variant of this block (e.g., oak_stairs -> stone_stairs)
    pub fn with_material(&self, material: &str) -> Result<BlockState, String> {
        let blockpedia_block = self.to_blockpedia()?;
        let transformed = blockpedia_block.with_material(material)
            .map_err(|e| e.to_string())?;
        Ok(Self::from_blockpedia(&transformed))
    }

    /// Get a shape variant of this block (e.g., stone -> stone_stairs)
    pub fn with_shape(&self, shape: BlockShape) -> Result<BlockState, String> {
        let blockpedia_block = self.to_blockpedia()?;
        let transformed = blockpedia_block.with_shape(shape)
            .map_err(|e| e.to_string())?;
        Ok(Self::from_blockpedia(&transformed))
    }

    /// Find all available material variants for this block's shape
    pub fn available_materials(&self) -> Result<Vec<String>, String> {
        let blockpedia_block = self.to_blockpedia()?;
        blockpedia_block.available_materials()
            .map_err(|e| e.to_string())
    }

    /// Find all available shape variants for this block's material
    pub fn available_shapes(&self) -> Result<Vec<BlockShape>, String> {
        let blockpedia_block = self.to_blockpedia()?;
        blockpedia_block.available_shapes()
            .map_err(|e| e.to_string())
    }

    /// Check if this block matches a color within a given tolerance
    pub fn matches_color(&self, target_color: &ExtendedColorData, tolerance: f32) -> bool {
        if let Some(color) = self.get_color() {
            color.distance_oklab(target_color) <= tolerance
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BlockState;

    #[test]
    fn test_block_state_creation() {
        let block = BlockState::new("minecraft:stone".to_string())
            .with_property("variant".to_string(), "granite".to_string());

        assert_eq!(block.name, "minecraft:stone");
        assert_eq!(block.properties.get("variant"), Some(&"granite".to_string()));
    }
}
