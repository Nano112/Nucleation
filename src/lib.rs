// src/lib.rs

// Core modules
mod universal_schematic;
mod region;
mod block_state;
mod entity;
pub mod block_entity;
mod formats;
mod print_utils;
mod bounding_box;
mod metadata;
mod block_position;
pub mod utils;
mod item;
mod chunk;

// Feature-specific modules
#[cfg(feature = "wasm")]
mod wasm;
#[cfg(feature = "ffi")]
pub mod ffi;
#[cfg(feature = "python")]
mod python;
#[cfg(feature = "php")]
mod php;
#[cfg(not(target_arch = "wasm32"))]
pub mod blockpedia;
#[cfg(not(target_arch = "wasm32"))]
pub mod region_operations;
pub mod api_definition;
pub mod codegen;

// Public re-exports
pub use universal_schematic::{UniversalSchematic, ContainerInfo};
pub use block_state::BlockState;
pub use region::Region;
pub use entity::Entity;
pub use formats::{litematic, schematic};
pub use print_utils::{format_schematic, format_json_schematic};

// Re-export WASM types when building with WASM feature
#[cfg(feature = "wasm")]
pub use wasm::*;

// Re-export PHP types when building with PHP feature
#[cfg(feature = "php")]
pub use php::*;

// Re-export blockpedia integration types (only for non-WASM targets)
#[cfg(not(target_arch = "wasm32"))]
pub use blockpedia::{ColorAnalysis, BlockpediaError, SchematicTransforms, SchematicColorAnalysis};
#[cfg(not(target_arch = "wasm32"))]
pub use region_operations::{RegionColorAnalysis, BatchOperation, RegionError};

// Re-export blockpedia types for public use (only for non-WASM targets)
#[cfg(not(target_arch = "wasm32"))]
pub use ::blockpedia::{color::ExtendedColorData, transforms::{BlockShape, Rotation, Direction}};
