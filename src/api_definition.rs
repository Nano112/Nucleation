//! Single source of truth for all API definitions
//! 
//! This module defines the core API interface that is automatically
//! translated to all binding formats (WASM, Python, FFI).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metadata for an API method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiMethod {
    /// Method name
    pub name: String,
    /// Method documentation
    pub docs: String,
    /// Input parameters
    pub params: Vec<ApiParam>,
    /// Return type
    pub return_type: ApiType,
    /// Whether this method is async
    pub is_async: bool,
    /// Whether this method is a constructor
    pub is_constructor: bool,
    /// Whether this method is a property getter
    pub is_getter: bool,
    /// Whether this method is a property setter
    pub is_setter: bool,
    /// Whether this method is static
    pub is_static: bool,
    /// Error type if any
    pub error_type: Option<String>,
}

/// Parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiParam {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: ApiType,
    /// Whether this parameter is optional
    pub optional: bool,
    /// Default value if any
    pub default: Option<String>,
    /// Parameter documentation
    pub docs: String,
}

/// Type definitions for cross-platform compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiType {
    // Primitive types
    String,
    I32,
    F32,
    Bool,
    Bytes,
    
    // Container types
    Vec(Box<ApiType>),
    HashMap(Box<ApiType>, Box<ApiType>),
    Option(Box<ApiType>),
    Result(Box<ApiType>, Box<ApiType>),
    
    // Custom types
    Custom(String),
    
    // Void type
    Void,
}

/// Class/struct definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiClass {
    /// Class name
    pub name: String,
    /// Class documentation
    pub docs: String,
    /// Methods available on this class
    pub methods: Vec<ApiMethod>,
    /// Properties of this class
    pub properties: Vec<ApiProperty>,
    /// Whether this class is copyable
    pub copyable: bool,
}

/// Property definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiProperty {
    /// Property name
    pub name: String,
    /// Property type
    pub property_type: ApiType,
    /// Whether this property is readonly
    pub readonly: bool,
    /// Property documentation
    pub docs: String,
}

/// Free function definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiFunction {
    /// Function name
    pub name: String,
    /// Function documentation
    pub docs: String,
    /// Input parameters
    pub params: Vec<ApiParam>,
    /// Return type
    pub return_type: ApiType,
    /// Whether this function is async
    pub is_async: bool,
    /// Error type if any
    pub error_type: Option<String>,
}

/// Complete API definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiDefinition {
    /// Library name
    pub name: String,
    /// Library version
    pub version: String,
    /// Library documentation
    pub docs: String,
    /// All classes in the API
    pub classes: Vec<ApiClass>,
    /// All free functions in the API
    pub functions: Vec<ApiFunction>,
}

/// Define the complete Nucleation API
pub fn nucleation_api() -> ApiDefinition {
    ApiDefinition {
        name: "nucleation".to_string(),
        version: "0.1.55".to_string(),
        docs: "A high-performance Minecraft schematic parser and utility library".to_string(),
        classes: vec![
            block_state_class(),
            schematic_class(),
        ],
        functions: vec![
            load_schematic_function(),
            save_schematic_function(),
            debug_schematic_function(),
            debug_json_schematic_function(),
        ],
    }
}

fn block_state_class() -> ApiClass {
    ApiClass {
        name: "BlockState".to_string(),
        docs: "Represents a Minecraft block with its properties".to_string(),
        copyable: true,
        properties: vec![
            ApiProperty {
                name: "name".to_string(),
                property_type: ApiType::String,
                readonly: true,
                docs: "The block's resource name (e.g., 'minecraft:stone')".to_string(),
            },
            ApiProperty {
                name: "properties".to_string(),
                property_type: ApiType::HashMap(Box::new(ApiType::String), Box::new(ApiType::String)),
                readonly: true,
                docs: "The block's state properties".to_string(),
            },
        ],
        methods: vec![
            ApiMethod {
                name: "new".to_string(),
                docs: "Create a new BlockState with the given name".to_string(),
                params: vec![
                    ApiParam {
                        name: "name".to_string(),
                        param_type: ApiType::String,
                        optional: false,
                        default: None,
                        docs: "Block resource name".to_string(),
                    },
                ],
                return_type: ApiType::Custom("BlockState".to_string()),
                is_async: false,
                is_constructor: true,
                is_getter: false,
                is_setter: false,
                is_static: false,
                error_type: None,
            },
            ApiMethod {
                name: "with_property".to_string(),
                docs: "Create a new BlockState with an additional property".to_string(),
                params: vec![
                    ApiParam {
                        name: "key".to_string(),
                        param_type: ApiType::String,
                        optional: false,
                        default: None,
                        docs: "Property key".to_string(),
                    },
                    ApiParam {
                        name: "value".to_string(),
                        param_type: ApiType::String,
                        optional: false,
                        default: None,
                        docs: "Property value".to_string(),
                    },
                ],
                return_type: ApiType::Custom("BlockState".to_string()),
                is_async: false,
                is_constructor: false,
                is_getter: false,
                is_setter: false,
                is_static: false,
                error_type: None,
            },
        ],
    }
}

fn schematic_class() -> ApiClass {
    ApiClass {
        name: "Schematic".to_string(),
        docs: "A Minecraft schematic containing blocks and metadata".to_string(),
        copyable: false,
        properties: vec![
            ApiProperty {
                name: "dimensions".to_string(),
                property_type: ApiType::Vec(Box::new(ApiType::I32)),
                readonly: true,
                docs: "The schematic's dimensions [width, height, length]".to_string(),
            },
            ApiProperty {
                name: "block_count".to_string(),
                property_type: ApiType::I32,
                readonly: true,
                docs: "Total number of non-air blocks".to_string(),
            },
            ApiProperty {
                name: "volume".to_string(),
                property_type: ApiType::I32,
                readonly: true,
                docs: "Total volume of the schematic".to_string(),
            },
            ApiProperty {
                name: "region_names".to_string(),
                property_type: ApiType::Vec(Box::new(ApiType::String)),
                readonly: true,
                docs: "Names of all regions in the schematic".to_string(),
            },
        ],
        methods: vec![
            ApiMethod {
                name: "new".to_string(),
                docs: "Create a new empty schematic".to_string(),
                params: vec![
                    ApiParam {
                        name: "name".to_string(),
                        param_type: ApiType::Option(Box::new(ApiType::String)),
                        optional: true,
                        default: Some("\"Default\"".to_string()),
                        docs: "Schematic name".to_string(),
                    },
                ],
                return_type: ApiType::Custom("Schematic".to_string()),
                is_async: false,
                is_constructor: true,
                is_getter: false,
                is_setter: false,
                is_static: false,
                error_type: None,
            },
            ApiMethod {
                name: "from_data".to_string(),
                docs: "Load schematic from byte data, auto-detecting format".to_string(),
                params: vec![
                    ApiParam {
                        name: "data".to_string(),
                        param_type: ApiType::Bytes,
                        optional: false,
                        default: None,
                        docs: "Raw schematic file data".to_string(),
                    },
                ],
                return_type: ApiType::Result(Box::new(ApiType::Void), Box::new(ApiType::String)),
                is_async: false,
                is_constructor: false,
                is_getter: false,
                is_setter: false,
                is_static: false,
                error_type: Some("String".to_string()),
            },
            ApiMethod {
                name: "from_litematic".to_string(),
                docs: "Load schematic from Litematic format data".to_string(),
                params: vec![
                    ApiParam {
                        name: "data".to_string(),
                        param_type: ApiType::Bytes,
                        optional: false,
                        default: None,
                        docs: "Litematic file data".to_string(),
                    },
                ],
                return_type: ApiType::Result(Box::new(ApiType::Void), Box::new(ApiType::String)),
                is_async: false,
                is_constructor: false,
                is_getter: false,
                is_setter: false,
                is_static: false,
                error_type: Some("String".to_string()),
            },
            ApiMethod {
                name: "to_litematic".to_string(),
                docs: "Convert schematic to Litematic format".to_string(),
                params: vec![],
                return_type: ApiType::Result(Box::new(ApiType::Bytes), Box::new(ApiType::String)),
                is_async: false,
                is_constructor: false,
                is_getter: false,
                is_setter: false,
                is_static: false,
                error_type: Some("String".to_string()),
            },
            ApiMethod {
                name: "from_schematic".to_string(),
                docs: "Load schematic from classic .schematic format data".to_string(),
                params: vec![
                    ApiParam {
                        name: "data".to_string(),
                        param_type: ApiType::Bytes,
                        optional: false,
                        default: None,
                        docs: "Schematic file data".to_string(),
                    },
                ],
                return_type: ApiType::Result(Box::new(ApiType::Void), Box::new(ApiType::String)),
                is_async: false,
                is_constructor: false,
                is_getter: false,
                is_setter: false,
                is_static: false,
                error_type: Some("String".to_string()),
            },
            ApiMethod {
                name: "to_schematic".to_string(),
                docs: "Convert schematic to classic .schematic format".to_string(),
                params: vec![],
                return_type: ApiType::Result(Box::new(ApiType::Bytes), Box::new(ApiType::String)),
                is_async: false,
                is_constructor: false,
                is_getter: false,
                is_setter: false,
                is_static: false,
                error_type: Some("String".to_string()),
            },
            ApiMethod {
                name: "set_block".to_string(),
                docs: "Set a block at the specified position".to_string(),
                params: vec![
                    ApiParam {
                        name: "x".to_string(),
                        param_type: ApiType::I32,
                        optional: false,
                        default: None,
                        docs: "X coordinate".to_string(),
                    },
                    ApiParam {
                        name: "y".to_string(),
                        param_type: ApiType::I32,
                        optional: false,
                        default: None,
                        docs: "Y coordinate".to_string(),
                    },
                    ApiParam {
                        name: "z".to_string(),
                        param_type: ApiType::I32,
                        optional: false,
                        default: None,
                        docs: "Z coordinate".to_string(),
                    },
                    ApiParam {
                        name: "block_name".to_string(),
                        param_type: ApiType::String,
                        optional: false,
                        default: None,
                        docs: "Block resource name".to_string(),
                    },
                ],
                return_type: ApiType::Void,
                is_async: false,
                is_constructor: false,
                is_getter: false,
                is_setter: false,
                is_static: false,
                error_type: None,
            },
            ApiMethod {
                name: "set_block_with_properties".to_string(),
                docs: "Set a block at the specified position with properties".to_string(),
                params: vec![
                    ApiParam {
                        name: "x".to_string(),
                        param_type: ApiType::I32,
                        optional: false,
                        default: None,
                        docs: "X coordinate".to_string(),
                    },
                    ApiParam {
                        name: "y".to_string(),
                        param_type: ApiType::I32,
                        optional: false,
                        default: None,
                        docs: "Y coordinate".to_string(),
                    },
                    ApiParam {
                        name: "z".to_string(),
                        param_type: ApiType::I32,
                        optional: false,
                        default: None,
                        docs: "Z coordinate".to_string(),
                    },
                    ApiParam {
                        name: "block_name".to_string(),
                        param_type: ApiType::String,
                        optional: false,
                        default: None,
                        docs: "Block resource name".to_string(),
                    },
                    ApiParam {
                        name: "properties".to_string(),
                        param_type: ApiType::HashMap(Box::new(ApiType::String), Box::new(ApiType::String)),
                        optional: false,
                        default: None,
                        docs: "Block state properties".to_string(),
                    },
                ],
                return_type: ApiType::Void,
                is_async: false,
                is_constructor: false,
                is_getter: false,
                is_setter: false,
                is_static: false,
                error_type: None,
            },
            ApiMethod {
                name: "get_block".to_string(),
                docs: "Get block at the specified position".to_string(),
                params: vec![
                    ApiParam {
                        name: "x".to_string(),
                        param_type: ApiType::I32,
                        optional: false,
                        default: None,
                        docs: "X coordinate".to_string(),
                    },
                    ApiParam {
                        name: "y".to_string(),
                        param_type: ApiType::I32,
                        optional: false,
                        default: None,
                        docs: "Y coordinate".to_string(),
                    },
                    ApiParam {
                        name: "z".to_string(),
                        param_type: ApiType::I32,
                        optional: false,
                        default: None,
                        docs: "Z coordinate".to_string(),
                    },
                ],
                return_type: ApiType::Option(Box::new(ApiType::Custom("BlockState".to_string()))),
                is_async: false,
                is_constructor: false,
                is_getter: false,
                is_setter: false,
                is_static: false,
                error_type: None,
            },
        ],
    }
}

fn load_schematic_function() -> ApiFunction {
    ApiFunction {
        name: "load_schematic".to_string(),
        docs: "Load a schematic from file path".to_string(),
        params: vec![
            ApiParam {
                name: "path".to_string(),
                param_type: ApiType::String,
                optional: false,
                default: None,
                docs: "File path to load from".to_string(),
            },
        ],
        return_type: ApiType::Result(Box::new(ApiType::Custom("Schematic".to_string())), Box::new(ApiType::String)),
        is_async: false,
        error_type: Some("String".to_string()),
    }
}

fn save_schematic_function() -> ApiFunction {
    ApiFunction {
        name: "save_schematic".to_string(),
        docs: "Save a schematic to file path".to_string(),
        params: vec![
            ApiParam {
                name: "schematic".to_string(),
                param_type: ApiType::Custom("Schematic".to_string()),
                optional: false,
                default: None,
                docs: "Schematic to save".to_string(),
            },
            ApiParam {
                name: "path".to_string(),
                param_type: ApiType::String,
                optional: false,
                default: None,
                docs: "File path to save to".to_string(),
            },
            ApiParam {
                name: "format".to_string(),
                param_type: ApiType::String,
                optional: true,
                default: Some("\"auto\"".to_string()),
                docs: "Format to save as (auto, litematic, schematic)".to_string(),
            },
        ],
        return_type: ApiType::Result(Box::new(ApiType::Void), Box::new(ApiType::String)),
        is_async: false,
        error_type: Some("String".to_string()),
    }
}

fn debug_schematic_function() -> ApiFunction {
    ApiFunction {
        name: "debug_schematic".to_string(),
        docs: "Get debug information for a schematic".to_string(),
        params: vec![
            ApiParam {
                name: "schematic".to_string(),
                param_type: ApiType::Custom("Schematic".to_string()),
                optional: false,
                default: None,
                docs: "Schematic to debug".to_string(),
            },
        ],
        return_type: ApiType::String,
        is_async: false,
        error_type: None,
    }
}

fn debug_json_schematic_function() -> ApiFunction {
    ApiFunction {
        name: "debug_json_schematic".to_string(),
        docs: "Get debug information for a schematic in JSON format".to_string(),
        params: vec![
            ApiParam {
                name: "schematic".to_string(),
                param_type: ApiType::Custom("Schematic".to_string()),
                optional: false,
                default: None,
                docs: "Schematic to debug".to_string(),
            },
        ],
        return_type: ApiType::String,
        is_async: false,
        error_type: None,
    }
}
