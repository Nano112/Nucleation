use std::fmt;
use std::sync::Arc;
use hashbrown::HashMap;
use std::collections::HashMap as StdHashMap;
use quartz_nbt::{NbtCompound, NbtTag};
use serde::{Deserialize, Serialize, Serializer, Deserializer};
use serde::ser::SerializeMap;
use serde::de::{Visitor, MapAccess};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

// We can't derive Hash and Serialize/Deserialize directly for the struct due to hashbrown::HashMap
// and Arc<str>, so we'll implement them manually
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlockState {
    pub name: Arc<str>,
    pub properties: HashMap<Arc<str>, Arc<str>>,
}



// Manually implement Hash for BlockState
impl Hash for BlockState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);

        // Sort keys for consistent hashing
        let mut keys: Vec<&Arc<str>> = self.properties.keys().collect();
        keys.sort_by(|a, b| a.to_string().cmp(&b.to_string()));

        for key in keys {
            key.hash(state);
            if let Some(value) = self.properties.get(key) {
                value.hash(state);
            }
        }
    }
}

// Implement Serialize for BlockState
impl Serialize for BlockState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("name", &*self.name)?;

        // Convert hashbrown::HashMap to a standard HashMap for serialization
        let props: StdHashMap<&str, &str> = self.properties
            .iter()
            .map(|(k, v)| (k.as_ref(), v.as_ref()))
            .collect();

        map.serialize_entry("properties", &props)?;
        map.end()
    }
}

// Implement Deserialize for BlockState
impl<'de> Deserialize<'de> for BlockState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct BlockStateVisitor(PhantomData<BlockState>);

        impl<'de> Visitor<'de> for BlockStateVisitor {
            type Value = BlockState;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a BlockState structure")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut name = None;
                let mut properties = HashMap::new();

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "name" => {
                            let name_str: String = map.next_value()?;
                            name = Some(Arc::from(name_str));
                        }
                        "properties" => {
                            let props_map: StdHashMap<String, String> = map.next_value()?;
                            for (k, v) in props_map {
                                properties.insert(Arc::from(k), Arc::from(v));
                            }
                        }
                        _ => {}
                    }
                }

                let name = name.ok_or_else(|| serde::de::Error::missing_field("name"))?;

                Ok(BlockState {
                    name,
                    properties,
                })
            }
        }

        deserializer.deserialize_map(BlockStateVisitor(PhantomData))
    }
}

impl fmt::Display for BlockState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if !self.properties.is_empty() {
            write!(f, "[")?;
            let mut first = true;
            // Sort properties for consistent output
            let mut props: Vec<_> = self.properties.iter().collect();
            props.sort_by(|a, b| a.0.cmp(b.0));

            for (key, value) in props {
                if !first {
                    write!(f, ",")?;
                }
                write!(f, "{}={}", key, value)?;
                first = false;
            }
            write!(f, "]")?;
        }
        Ok(())
    }
}


impl BlockState {
        pub fn new<S: Into<Arc<str>>>(name: S) -> Self {
            BlockState {
                name: name.into(),
                properties: HashMap::new(),
            }
        }

        pub fn add_prop(&mut self, key: &str, value: &str) {
            self.properties
                .insert(Arc::from(key), Arc::from(value));
        }

        pub fn with_str_props(self, props: &[(&'static str, &'static str)]) -> Self {
            let map = props
                .iter()
                .map(|&(k, v)| (Arc::<str>::from(k), Arc::<str>::from(v)))
                .collect();
            self.with_properties(map)
        }

        pub fn with_prop<K: Into<Arc<str>>, V: Into<Arc<str>>>(mut self, key: K, value: V) -> Self {
            self.properties.insert(key.into(), value.into());
            self
        }

        pub fn with_properties(mut self, properties: HashMap<Arc<str>, Arc<str>>) -> Self {
            self.properties = properties;
            self
        }

        pub fn set_property<K: Into<Arc<str>>, V: Into<Arc<str>>>(&mut self, key: K, value: V) {
            self.properties.insert(key.into(), value.into());
        }

        pub fn remove_property<K: AsRef<str>>(&mut self, key: K) {
            self.properties.remove(key.as_ref());
        }

        pub fn get_property<K: AsRef<str>>(&self, key: K) -> Option<&Arc<str>> {
            self.properties.get(key.as_ref())
        }

        pub fn get_name(&self) -> &Arc<str> {
            &self.name
        }

        pub fn air() -> Self {
            static AIR_NAME: &str = "minecraft:air";
            Self::new(AIR_NAME)
        }

        pub fn to_nbt(&self) -> NbtTag {
            let mut compound = NbtCompound::new();
            compound.insert("Name", self.name.to_string());

            if !self.properties.is_empty() {
                let mut properties = NbtCompound::new();
                for (key, value) in &self.properties {
                    properties.insert(key.to_string(), value.to_string());
                }
                compound.insert("Properties", NbtTag::Compound(properties));
            }

            NbtTag::Compound(compound)
        }

        pub fn from_nbt(compound: &NbtCompound) -> Result<Self, String> {
            let name = compound
                .get::<_, &str>("Name")
                .map_err(|e| format!("Failed to get Name: {}", e))?;

            let mut block_state = BlockState::new(name);

            if let Ok(props) = compound.get::<_, &NbtCompound>("Properties") {
                for (key, value) in props.inner() {
                    if let NbtTag::String(value_str) = value {
                        // Clone the strings to avoid reference issues
                        let key_string = key.to_string();
                        let value_string = value_str.to_string();
                        block_state.set_property(key_string, value_string);
                    }
                }
            }

            Ok(block_state)
        }
    }