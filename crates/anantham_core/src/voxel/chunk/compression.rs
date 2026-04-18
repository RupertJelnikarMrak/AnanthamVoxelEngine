//! Garbage Collector implementation for chunks.
//!
//! As the world gets modified, chunks' number of unique blocks within can fall,
//! therefore reducing it's palette size. We lazly scan for it and reduce them
//! in memory where possible.

use super::data::{CHUNK_VOLUME, ChunkData};
use super::local::write_paletted4_unchecked;
use crate::voxel::block::data::BlockState;

#[inline(always)]
pub fn read_paletted4(indices: &[u8], index: usize) -> usize {
    let byte = indices[index / 2];
    if index.is_multiple_of(2) {
        (byte >> 4) as usize
    } else {
        (byte & 0x0F) as usize
    }
}

/// Runs on the async thread pool. Scans the chunk array and returns a smaller
/// memory tier if the unique block count has dropped sufficiently.
pub fn attempt_compression(data: ChunkData) -> Option<ChunkData> {
    match data {
        ChunkData::Homogenous(_) => None,

        ChunkData::Paletted4 { palette, indices } => {
            let (used_count, new_palette, _) =
                analyze_usage(&palette, |i| read_paletted4(&indices[..], i));

            if used_count <= 1 {
                let state = new_palette.first().copied().unwrap_or(BlockState::AIR);
                Some(ChunkData::Homogenous(state))
            } else {
                None
            }
        }

        ChunkData::Paletted8 { palette, indices } => {
            let (used_count, new_palette, mapping) =
                analyze_usage(&palette, |i| indices[i] as usize);

            if used_count <= 1 {
                let state = new_palette.first().copied().unwrap_or(BlockState::AIR);
                Some(ChunkData::Homogenous(state))
            } else if used_count <= 16 {
                let mut new_indices: Box<[u8; CHUNK_VOLUME / 2]> = vec![0u8; CHUNK_VOLUME / 2]
                    .into_boxed_slice()
                    .try_into()
                    .unwrap();
                for i in 0..CHUNK_VOLUME {
                    let old_p_idx = indices[i] as usize;
                    let new_p_idx = mapping[old_p_idx] as u8;
                    unsafe { write_paletted4_unchecked(&mut new_indices[..], i, new_p_idx) }
                }
                Some(ChunkData::Paletted4 {
                    palette: new_palette,
                    indices: new_indices,
                })
            } else {
                None
            }
        }

        ChunkData::Paletted16 { palette, indices } => {
            let (used_count, new_palette, mapping) =
                analyze_usage(&palette, |i| indices[i] as usize);

            if used_count <= 1 {
                let state = new_palette.first().copied().unwrap_or(BlockState::AIR);
                Some(ChunkData::Homogenous(state))
            } else if used_count <= 16 {
                let mut new_indices: Box<[u8; CHUNK_VOLUME / 2]> = vec![0u8; CHUNK_VOLUME / 2]
                    .into_boxed_slice()
                    .try_into()
                    .unwrap();
                for i in 0..CHUNK_VOLUME {
                    let old_p_idx = indices[i] as usize;
                    let new_p_idx = mapping[old_p_idx] as u8;
                    unsafe { write_paletted4_unchecked(&mut new_indices[..], i, new_p_idx) }
                }
                Some(ChunkData::Paletted4 {
                    palette: new_palette,
                    indices: new_indices,
                })
            } else if used_count <= 256 {
                let mut new_indices: Box<[u8; CHUNK_VOLUME]> = vec![0u8; CHUNK_VOLUME]
                    .into_boxed_slice()
                    .try_into()
                    .unwrap();
                for i in 0..CHUNK_VOLUME {
                    let old_p_idx = indices[i] as usize;
                    new_indices[i] = mapping[old_p_idx] as u8;
                }
                Some(ChunkData::Paletted8 {
                    palette: new_palette,
                    indices: new_indices,
                })
            } else {
                None
            }
        }
    }
}

/// Helper to count unique blocks and build a mapping from the old palette to the new compact palette.
fn analyze_usage<F>(palette: &[BlockState], mut get_idx: F) -> (usize, Vec<BlockState>, Vec<usize>)
where
    F: FnMut(usize) -> usize,
{
    let mut used = vec![false; palette.len()];

    // 1. Mark used indices
    for i in 0..CHUNK_VOLUME {
        used[get_idx(i)] = true;
    }

    let mut new_palette = Vec::with_capacity(16); // Guess small to avoid over-allocating
    let mut mapping = vec![0; palette.len()];

    // 2. Build the new compact palette and the translation map
    for (old_idx, &is_used) in used.iter().enumerate() {
        if is_used {
            mapping[old_idx] = new_palette.len();
            new_palette.push(palette[old_idx]);
        }
    }

    (new_palette.len(), new_palette, mapping)
}
