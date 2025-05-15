use std::io::{BufReader, Cursor, Read};

use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use quartz_nbt::{NbtCompound, NbtList, NbtTag};
use quartz_nbt::io::{read_nbt, Flavor};
use crate::{BlockState, UniversalSchematic};
use crate::block_entity::BlockEntity;
use crate::entity::Entity;
use crate::region::Region;

#[cfg(feature = "wasm")]
use wasm_bindgen::JsValue;

#[cfg(feature = "wasm")]
use web_sys::console;


pub fn is_schematic(data: &[u8]) -> bool {
    // Decompress the data
    let reader = BufReader::with_capacity(1 << 20, data); // 1 MiB buf
    let mut gz = GzDecoder::new(reader);
    let (root, _) = match read_nbt(&mut gz, Flavor::Uncompressed) {
        Ok(result) => result,
        Err(_) => {
            #[cfg(feature = "wasm")]
            let _: Result<(), JsValue> = Err(JsValue::from_str("Failed to read NBT data"));
            return false;
        }
    };


    //things should be under Schematic tag if not treat root as the schematic
    let root = root.get::<_, &NbtCompound>("Schematic").unwrap_or(&root);

    // get tge version of the schematic
    let version = root.get::<_, i32>("Version");
    #[cfg(feature = "wasm")]
    console::log_1(&format!("Schematic Version: {:?}", version).into());
    if version.is_err() {
        return root.get::<_, &NbtCompound>("Blocks").is_ok();
    }


    // Check if it's a v3 schematic (which has a Blocks compound)
    if version.unwrap() == 3 {
        #[cfg(feature = "wasm")]
        console::log_1(&format!("Detected v3 schematic").into());
        return root.get::<_, &NbtCompound>("Blocks").is_ok();
    }

    // Otherwise check for v2 format
    root.get::<_, i32>("DataVersion").is_ok() &&
    root.get::<_, i16>("Width").is_ok() &&
    root.get::<_, i16>("Height").is_ok() &&
    root.get::<_, i16>("Length").is_ok() &&
    root.get::<_, &Vec<i8>>("BlockData").is_ok()
}

pub fn to_schematic(schematic: &UniversalSchematic) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut root = NbtCompound::new();

    root.insert("Version", NbtTag::Int(2)); // Schematic format version 2
    root.insert("DataVersion", NbtTag::Int(schematic.metadata.mc_version.unwrap_or(1343)));

    let bounding_box = schematic.get_bounding_box();
    let (width, height, length) = bounding_box.get_dimensions();

    root.insert("Width", NbtTag::Short((width as i16).abs() ));
    root.insert("Height", NbtTag::Short((height as i16).abs()));
    root.insert("Length", NbtTag::Short((length as i16).abs()));

    root.insert("Size", NbtTag::IntArray(vec![width as i32, height as i32, length as i32]));


    let offset = vec![0, 0, 0];
    root.insert("Offset", NbtTag::IntArray(offset));


    let merged_region = schematic.get_merged_region();
    
    root.insert("Palette", convert_palette(&merged_region.palette).0);
    root.insert("PaletteMax", convert_palette(&merged_region.palette).1 + 1);

    let block_data: Vec<u8> = merged_region.blocks.iter()
        .flat_map(|&block_id| encode_varint(block_id as u32))
        .collect();


    //attempt to decode the block data for debug since it's buggy
    let mut reader = Cursor::new(block_data.clone());
    let mut decoded_data = Vec::new();
    while reader.position() < block_data.len() as u64 {
        let value = decode_varint(&mut reader)?;
        decoded_data.push(value);
    }
    // println!("Decoded Data: {:?}", decoded_data);
    // println!("Decoded Data Length: {:?}", decoded_data.len());

    root.insert("BlockData", NbtTag::ByteArray(block_data.iter().map(|&x| x as i8).collect()));

    let mut block_entities = NbtList::new();
    for region in schematic.regions.values() {
        block_entities.extend(convert_block_entities(region).iter().cloned());
    }
    root.insert("BlockEntities", NbtTag::List(block_entities));

    let mut entities = NbtList::new();
    for region in schematic.regions.values() {
        entities.extend(convert_entities(region).iter().cloned());
    }
    root.insert("Entities", NbtTag::List(entities));

    root.insert("Metadata", schematic.metadata.to_nbt());


    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    quartz_nbt::io::write_nbt(&mut encoder, Option::from("Schematic"), &root, quartz_nbt::io::Flavor::Uncompressed)?;
    Ok(encoder.finish()?)
}




pub fn from_schematic(data: &[u8]) -> Result<UniversalSchematic, Box<dyn std::error::Error>> {
    let reader   = BufReader::with_capacity(1 << 20, data);   // 1 MiB buf
    let mut gz   = GzDecoder::new(reader);
    let (root, _) = read_nbt(&mut gz, Flavor::Uncompressed)?;

    let schem = root.get::<_, &NbtCompound>("Schematic").unwrap_or(&root);
    let schem_version = schem.get::<_, i32>("Version")?;

    let name = if let Some(metadata) = schem.get::<_, &NbtCompound>("Metadata").ok() {
        metadata.get::<_, &str>("Name").ok().map(|s| s.to_string())
    } else {
        None
    }.unwrap_or_else(|| "Unnamed".to_string());

    let mc_version = schem.get::<_, i32>("DataVersion").ok();

    let mut schematic = UniversalSchematic::new(name);
    schematic.metadata.mc_version = mc_version;

    let width = schem.get::<_, i16>("Width")? as u32;
    let height = schem.get::<_, i16>("Height")? as u32;
    let length = schem.get::<_, i16>("Length")? as u32;

    let block_container=
    if schem_version == 2 {
        schem
    } else {
        schem.get::<_, &NbtCompound>("Blocks")?
    };

    let block_palette = parse_block_palette(&block_container)?;

    let block_data = parse_block_data(&block_container, width, height, length)?;


    let mut region = Region::new("Main".to_string(), (0, 0, 0), (width as i32, height as i32, length as i32));
    region.palette = block_palette;

    region.blocks = block_data.iter().map(|&x| x as usize).collect();

    let block_entities = parse_block_entities(&block_container)?;
    for block_entity in block_entities {
        region.add_block_entity(block_entity);
    }

    let entities = parse_entities(&schem)?;
    for entity in entities {
        region.add_entity(entity);
    }

    schematic.add_region(region);
    Ok(schematic)
}



fn convert_block_entities(region: &Region) -> NbtList {
    let mut block_entities = NbtList::new();

    for (_, block_entity) in &region.block_entities {
        block_entities.push(block_entity.to_nbt());
    }

    block_entities
}

fn convert_entities(region: &Region) -> NbtList {
    let mut entities = NbtList::new();

    for entity in &region.entities {
        entities.push(entity.to_nbt());
    }

    entities
}

fn parse_block_palette(region_tag: &NbtCompound) -> Result<Vec<BlockState>, Box<dyn std::error::Error>> {
    let palette_compound = region_tag.get::<_, &NbtCompound>("Palette")?;
    let palette_max = region_tag.get::<_, i32>("PaletteMax") // V2
        .unwrap_or(palette_compound.len() as i32) as usize; // V3
    let mut palette = vec![BlockState::new("minecraft:air".to_string()); palette_max + 1];

    for (block_state_str, value) in palette_compound.inner() {
        if let NbtTag::Int(id) = value {
            let block_state = parse_block_state(block_state_str);
            palette[*id as usize] = block_state;
        }
    }

    Ok(palette)
}

fn parse_block_state(input: &str) -> BlockState {
    if let Some((name, properties_str)) = input.split_once('[') {
        let name = name.to_string();
        let properties = properties_str
            .trim_end_matches(']')
            .split(',')
            .filter_map(|prop| {
                let mut parts = prop.splitn(2, '=');
                Some((
                    parts.next()?.trim().to_string(),
                    parts.next()?.trim().to_string(),
                ))
            })
            .collect();
        BlockState { name, properties }
    } else {
        BlockState::new(input.to_string())
    }
}

fn convert_palette(palette: &Vec<BlockState>) -> (NbtCompound, i32) {
    let mut nbt_palette = NbtCompound::new();
    let mut max_id = 0;
    
    // Always ensure air is at index 0
    nbt_palette.insert("minecraft:air", NbtTag::Int(0));
    
    let mut next_id = 1; // Start at 1 since air is at 0
    
    for (id, block_state) in palette.iter().enumerate() {
        if block_state.name == "minecraft:air" {
            continue; // Skip air blocks as we already added it at index 0
        }
        
        let key = if block_state.properties.is_empty() {
            block_state.name.clone()
        } else {
            format!("{}[{}]", block_state.name,
                    block_state.properties.iter()
                        .map(|(k, v)| format!("{}={}", k, v))
                        .collect::<Vec<_>>()
                        .join(","))
        };
        
        nbt_palette.insert(&key, NbtTag::Int(next_id));
        max_id = max_id.max(next_id);
        next_id += 1;
    }

    (nbt_palette, max_id as i32)
}

pub fn encode_varint(value: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut val = value;
    loop {
        let mut byte = (val & 0b0111_1111) as u8;
        val >>= 7;
        if val != 0 {
            byte |= 0b1000_0000;
        }
        bytes.push(byte);
        if val == 0 {
            break;
        }
    }
    bytes
}

fn decode_varint<R: Read>(reader: &mut R) -> Result<u32, Box<dyn std::error::Error>> {
    let mut result = 0u32;
    let mut shift = 0;
    loop {
        let mut byte = [0u8; 1];
        reader.read_exact(&mut byte)?;
        result |= ((byte[0] & 0b0111_1111) as u32) << shift;
        if byte[0] & 0b1000_0000 == 0 {
            return Ok(result);
        }
        shift += 7;
        if shift >= 32 {
            return Err("Varint is too long".into());
        }
    }
}

fn parse_block_data(
    region_tag: &NbtCompound,
    width: u32,
    height: u32,
    length: u32,
) -> Result<Vec<u32>, Box<dyn std::error::Error>> {
    // V2 = BlockData, V3 = Data
    let block_data_i8 = region_tag
        .get::<_, &Vec<i8>>("BlockData")
        .or(region_tag.get::<_, &Vec<i8>>("Data"))?;

    let mut block_data_u8: &[u8] = unsafe {
        std::slice::from_raw_parts(block_data_i8.as_ptr() as *const u8,
                                   block_data_i8.len())
    };

    // ---------- fast var-int decode ----------
    #[inline]
    fn read_varint(slice: &mut &[u8]) -> Option<u32> {
        let mut out = 0u32;
        let mut shift = 0;
        while !slice.is_empty() {
            let byte = slice[0];
            *slice = &slice[1..];
            out |= ((byte & 0x7F) as u32) << shift;
            if byte & 0x80 == 0 {
                return Some(out);
            }
            shift += 7;
        }
        None
    }

    let expected_length = (width * height * length) as usize;
    let mut block_data: Vec<u32> = Vec::with_capacity(expected_length);

    while let Some(id) = read_varint(&mut block_data_u8) {
        block_data.push(id);
    }

    if block_data.len() != expected_length {
        return Err(format!(
            "Block data length mismatch: expected {}, got {}",
            expected_length,
            block_data.len()
        ).into());
    }

    Ok(block_data)
}



fn parse_block_entities(region_tag: &NbtCompound) -> Result<Vec<BlockEntity>, Box<dyn std::error::Error>> {
    let block_entities_list = region_tag.get::<_, &NbtList>("BlockEntities")?;
    let mut block_entities = Vec::new();

    for tag in block_entities_list.iter() {
        if let NbtTag::Compound(compound) = tag {
            let block_entity = BlockEntity::from_nbt(compound);
            block_entities.push(block_entity);
        }
    }

    Ok(block_entities)
}

fn parse_entities(region_tag: &NbtCompound) -> Result<Vec<Entity>, Box<dyn std::error::Error>> {
    if !region_tag.contains_key("Entities") {
        return Ok(Vec::new());
    }
    let entities_list = region_tag.get::<_, &NbtList>("Entities")?;
    let mut entities = Vec::new();

    for tag in entities_list.iter() {
        if let NbtTag::Compound(compound) = tag {
            entities.push(Entity::from_nbt(compound)?);
        }
    }

    Ok(entities)
}



#[cfg(test)]
mod tests {
    use std::fs;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;

    use crate::{BlockState, UniversalSchematic};
    use crate::litematic::{from_litematic, to_litematic};

    use super::*;

    #[test]
    fn test_schematic_file_generation() {
        // Create a test schematic
        let mut schematic = UniversalSchematic::new("Test Schematic".to_string());
        let stone = BlockState::new("minecraft:stone".to_string());
        let dirt = BlockState::new("minecraft:dirt".to_string());

        for x in 0..5 {
            for y in 0..5 {
                for z in 0..5 {
                    if (x + y + z) % 2 == 0 {
                        schematic.set_block(x, y, z, stone.clone());
                    } else {
                        schematic.set_block(x, y, z, dirt.clone());
                    }
                }
            }
        }

        // Convert the schematic to .schem format
        let schem_data = to_schematic(&schematic).expect("Failed to convert schematic");

        // Save the .schem file
        let mut file = File::create("test_schematic.schem").expect("Failed to create file");
        file.write_all(&schem_data).expect("Failed to write to file");

        // Read the .schem file back
        let loaded_schem_data = std::fs::read("test_schematic.schem").expect("Failed to read file");

        // Parse the loaded .schem data
        let loaded_schematic = from_schematic(&loaded_schem_data).expect("Failed to parse schematic");

        // Compare the original and loaded schematics
        assert_eq!(schematic.metadata.name, loaded_schematic.metadata.name);
        assert_eq!(schematic.regions.len(), loaded_schematic.regions.len());

        let original_region = schematic.regions.get("Main").unwrap();
        let loaded_region = loaded_schematic.regions.get("Main").unwrap();

        assert_eq!(original_region.entities.len(), loaded_region.entities.len());
        assert_eq!(original_region.block_entities.len(), loaded_region.block_entities.len());

        // Clean up the generated file
        //std::fs::remove_file("test_schematic.schem").expect("Failed to remove file");
    }

    #[test]
    fn test_varint_encoding_decoding() {
        let test_cases = vec![
            0u32,
            1u32,
            127u32,
            128u32,
            255u32,
            256u32,
            65535u32,
            65536u32,
            4294967295u32,
        ];

        for &value in &test_cases {
            let encoded = encode_varint(value);

            let mut cursor = Cursor::new(encoded);
            let decoded = decode_varint(&mut cursor).unwrap();

            assert_eq!(value, decoded, "Encoding and decoding failed for value: {}", value);
        }
    }

    #[test]
    fn test_parse_block_data() {
        let mut nbt = NbtCompound::new();
        let block_data = vec![0, 1, 2, 1, 0, 2, 1, 0]; // 8 blocks
        let encoded_block_data: Vec<u8> = block_data.iter()
            .flat_map(|&v| encode_varint(v))
            .collect();

        nbt.insert("BlockData", NbtTag::ByteArray(encoded_block_data.iter().map(|&x| x as i8).collect()));

        let parsed_data = parse_block_data(&nbt, 2, 2, 2).expect("Failed to parse block data");
        assert_eq!(parsed_data, vec![0, 1, 2, 1, 0, 2, 1, 0]);
    }

    #[test]
    fn test_convert_palette() {
        let palette = vec![
            BlockState::new("minecraft:stone".to_string()),
            BlockState::new("minecraft:dirt".to_string()),
            BlockState {
                name: "minecraft:wool".to_string(),
                properties: [("color".to_string(), "red".to_string())].into_iter().collect(),
            },
        ];

        let (nbt_palette, max_id) = convert_palette(&palette);

        // Air is always at index 0, and other blocks follow
        assert_eq!(max_id, 3); // Now we have 4 entries: air, stone, dirt, wool
        assert_eq!(nbt_palette.get::<_, i32>("minecraft:air").unwrap(), 0);
        assert_eq!(nbt_palette.get::<_, i32>("minecraft:stone").unwrap(), 1);
        assert_eq!(nbt_palette.get::<_, i32>("minecraft:dirt").unwrap(), 2);
        assert_eq!(nbt_palette.get::<_, i32>("minecraft:wool[color=red]").unwrap(), 3);
    }


    #[test]
    fn test_import_new_chest_test_schem() {
        let name = "new_chest_test";
        let input_path_str = format!("tests/samples/{}.schem", name);
        let schem_path = Path::new(&input_path_str);
        assert!(schem_path.exists(), "Sample .schem file not found");
        let schem_data = fs::read(schem_path).expect(format!("Failed to read {}", input_path_str).as_str());

        let mut schematic = from_schematic(&schem_data).expect("Failed to parse schematic");
        assert_eq!(schematic.metadata.name, Some("Unnamed".to_string()));
    }

    #[test]
    fn test_conversion() {
        let output_dir_path = Path::new("tests/output");
        if !output_dir_path.exists() {
            fs::create_dir_all(output_dir_path)
                .expect("Failed to create output directory 'tests/output'");
        }
        let schem_name = "tests/samples/cutecounter.schem";
        let output_litematic_name = "tests/output/cutecounter.litematic";
        let output_schematic_name = "tests/output/cutecounter.schem";

        //load the schem as a UniversalSchematic
        let schem_data = fs::read(schem_name).expect("Failed to read schem file");
        let schematic = from_schematic(&schem_data).expect("Failed to parse schematic");

        //convert the UniversalSchematic to a Litematic
        let litematic_output_data = to_litematic(&schematic).expect("Failed to convert to litematic");
        let mut litematic_output_file = File::create(output_litematic_name).expect("Failed to create litematic file");
        litematic_output_file.write_all(&litematic_output_data).expect("Failed to write litematic file");

        //load back from the litematic file
        let litematic_data = fs::read(output_litematic_name).expect("Failed to read litematic file");
        let schematic_from_litematic = from_litematic(&litematic_data).expect("Failed to parse litematic");

        //convert the Litematic back to a UniversalSchematic
        let schematic_output_data = to_schematic(&schematic_from_litematic).expect("Failed to convert to schematic");
        let mut schematic_output_file = File::create(output_schematic_name).expect("Failed to create schematic file");
        schematic_output_file.write_all(&schematic_output_data).expect("Failed to write schematic file");
    }
}