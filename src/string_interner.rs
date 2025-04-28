use serde::{Serialize, Deserialize, Serializer, Deserializer};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
struct StringInterner {
    strings: HashMap<String, Arc<str>>,
}

// Implement Serialize manually
impl Serialize for StringInterner {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Just serialize the keys - we don't need to serialize the Arc<str> values
        // since they're just duplicates of the keys
        let keys: Vec<&String> = self.strings.keys().collect();
        keys.serialize(serializer)
    }
}

// Implement custom Deserialize
impl<'de> Deserialize<'de> for StringInterner {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let keys: Vec<String> = Vec::deserialize(deserializer)?;

        let mut interner = StringInterner {
            strings: HashMap::new(),
        };

        // Rebuild the interner from the keys
        for key in keys {
            let arc_str: Arc<str> = key.as_str().into();
            interner.strings.insert(key, arc_str);
        }

        Ok(interner)
    }
}

impl StringInterner {
    fn new() -> Self {
        StringInterner {
            strings: HashMap::new(),
        }
    }

    fn intern(&mut self, s: &str) -> Arc<str> {
        if let Some(interned) = self.strings.get(s) {
            interned.clone()
        } else {
            let arc_str: Arc<str> = s.into();
            self.strings.insert(s.to_string(), arc_str.clone());
            arc_str
        }
    }
}