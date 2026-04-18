//! Data definitions for Chunk
//!
//! A chunk represents a 32x32x32 region of the world.
//! A world has an almost unlimited amount of potential chunks, thousands of which need to be
//! actively loaded in memory as the program runs.
//! As such we store a local block pallete of blocks used in that chunk and use less bytes per
//! block state saving considerable memory.
//!
//! Each chunk has 4 potential states:
//! - Homogenous: Chunk features a single block, we only need to store that. (Air, some underground
//!   chunks...)
//! - Paletted4: 2-16 unique blocks, we only need 4 bits per block to store
//!   it's local scope id.
//! - Paletted8: 17-256 unique blocks, fits within a byte.
//! - Paletted16: 257-32,768 unique blocks, a chunk only has this many blocks, fits within 16 bytes

use crate::voxel::block::data::BlockState;
use bevy::prelude::*;

pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

#[derive(Clone)]
pub enum ChunkData {
    Homogenous(BlockState),

    Paletted4 {
        palette: Vec<BlockState>,
        indices: Box<[u8; CHUNK_VOLUME / 2]>,
    },

    Paletted8 {
        palette: Vec<BlockState>,
        indices: Box<[u8; CHUNK_VOLUME]>,
    },

    Paletted16 {
        palette: Vec<BlockState>,
        indices: Box<[u16; CHUNK_VOLUME]>,
    },
}

#[derive(Component, Clone)]
pub struct Chunk {
    pub data: ChunkData,
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            data: ChunkData::Homogenous(BlockState::AIR),
        }
    }
}
