use std::rc::Rc;
use crate::block_position::BlockPosition;
use crate::bounding_box::BoundingBox;
use crate::{BlockState, UniversalSchematic};

// First, define a struct to represent our lazy iterator on the Rust side
pub struct ChunksIterator {
    pub(crate) schematic: Rc<UniversalSchematic>,
    bbox: BoundingBox,
    pub(crate) chunk_width: i32,
    pub(crate) chunk_height: i32,
    pub(crate) chunk_length: i32,

    // Current position in the iteration
    current_chunk_x: i32,
    current_chunk_y: i32,
    current_chunk_z: i32,
    chunks_processed: bool,
}

impl ChunksIterator {
    pub fn new(schematic: Rc<UniversalSchematic>, chunk_width: i32, chunk_height: i32, chunk_length: i32) -> Self {
        let bbox = schematic.get_bounding_box();

        // Calculate the minimum chunk coordinates based on bounding box
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

        ChunksIterator {
            schematic,
            bbox,
            chunk_width,
            chunk_height,
            chunk_length,
            current_chunk_x: min_chunk_x,
            current_chunk_y: min_chunk_y,
            current_chunk_z: min_chunk_z,
            chunks_processed: false,
        }
    }

    // Get the next chunk in the iteration
    pub fn next_chunk(&mut self) -> Option<(i32, i32, i32, Vec<(BlockPosition, BlockState)>)> {
        if self.chunks_processed {
            return None;
        }

        // Calculate chunk bounds
        let chunk_min_x = self.current_chunk_x * self.chunk_width;
        let chunk_min_y = self.current_chunk_y * self.chunk_height;
        let chunk_min_z = self.current_chunk_z * self.chunk_length;

        let chunk_max_x = chunk_min_x + self.chunk_width - 1;
        let chunk_max_y = chunk_min_y + self.chunk_height - 1;
        let chunk_max_z = chunk_min_z + self.chunk_length - 1;

        // Check if this chunk could contain blocks (intersects with bounding box)
        if chunk_min_x > self.bbox.max.0 || chunk_max_x < self.bbox.min.0 ||
            chunk_min_y > self.bbox.max.1 || chunk_max_y < self.bbox.min.1 ||
            chunk_min_z > self.bbox.max.2 || chunk_max_z < self.bbox.min.2 {
            // Skip this chunk and move to the next one
            self.advance_position();
            return self.next_chunk();
        }

        // Collect blocks in this chunk (only those that exist)
        let mut blocks = Vec::new();

        // Define chunk bounds clamped to the schematic bounding box
        let min_x = std::cmp::max(chunk_min_x, self.bbox.min.0);
        let min_y = std::cmp::max(chunk_min_y, self.bbox.min.1);
        let min_z = std::cmp::max(chunk_min_z, self.bbox.min.2);

        let max_x = std::cmp::min(chunk_max_x, self.bbox.max.0);
        let max_y = std::cmp::min(chunk_max_y, self.bbox.max.1);
        let max_z = std::cmp::min(chunk_max_z, self.bbox.max.2);

        // Only iterate through the intersection of chunk and bounding box
        for x in min_x..=max_x {
            for y in min_y..=max_y {
                for z in min_z..=max_z {
                    if let Some(block) = self.schematic.get_block(x, y, z) {
                        // Skip air blocks (typically index 0 in most implementations)
                        if !block.name.contains("air") {
                            blocks.push((BlockPosition { x, y, z }, block.clone()));
                        }
                    }
                }
            }
        }

        // Save current position
        let current_pos = (self.current_chunk_x, self.current_chunk_y, self.current_chunk_z);

        // Advance to next position for future calls
        self.advance_position();

        // Return the chunk data if it has blocks
        if !blocks.is_empty() {
            Some((current_pos.0, current_pos.1, current_pos.2, blocks))
        } else {
            // Skip empty chunks
            self.next_chunk()
        }
    }

    // Helper to advance to the next chunk position
    fn advance_position(&mut self) {
        // Calculate max chunk coordinates based on bounding box
        let max_chunk_x = (self.bbox.max.0 + self.chunk_width - 1) / self.chunk_width;
        let max_chunk_y = (self.bbox.max.1 + self.chunk_height - 1) / self.chunk_height;
        let max_chunk_z = (self.bbox.max.2 + self.chunk_length - 1) / self.chunk_length;

        // Advance Z first, then Y, then X
        self.current_chunk_z += 1;
        if self.current_chunk_z > max_chunk_z {
            self.current_chunk_z = if self.bbox.min.2 < 0 {
                (self.bbox.min.2 - self.chunk_length + 1) / self.chunk_length
            } else {
                self.bbox.min.2 / self.chunk_length
            };

            self.current_chunk_y += 1;

            if self.current_chunk_y > max_chunk_y {
                self.current_chunk_y = if self.bbox.min.1 < 0 {
                    (self.bbox.min.1 - self.chunk_height + 1) / self.chunk_height
                } else {
                    self.bbox.min.1 / self.chunk_height
                };

                self.current_chunk_x += 1;

                if self.current_chunk_x > max_chunk_x {
                    // We've processed all chunks
                    self.chunks_processed = true;
                }
            }
        }
    }
}