use std::sync::atomic::Ordering;

use super::data::{CHUNK_VOLUME, Chunk, ChunkData};
use crate::voxel::block::REGISTERED_STATE_COUNT;
use crate::voxel::block::data::BlockState;
use crate::voxel::world::math::local_to_index;
use bevy::prelude::*;

#[derive(Debug)]
pub enum ChunkError {
    /// A local coordinate was out of chunk bounds.
    OutOfLocalBounds,
    /// The block is not in the registry.
    UnregisteredBlockSet,
}

impl Chunk {
    pub fn get_block(&self, local: IVec3) -> Result<BlockState, ChunkError> {
        if (local.x as u32) >= 32 || (local.y as u32) >= 32 || (local.z as u32) >= 32 {
            return Err(ChunkError::OutOfLocalBounds);
        }
        // SAFETY: Bounds are checked right above.
        Ok(unsafe { self.get_block_unchecked(local_to_index(local)) })
    }

    /// Blazing fast, completely unchecked access.
    ///
    /// # Safety
    /// The caller should guarantee that `index` is strictly between `0` and `32767` (`CHUNK_VOLUME - 1`).
    #[inline(always)]
    pub unsafe fn get_block_unchecked(&self, index: usize) -> BlockState {
        debug_assert!(index < CHUNK_VOLUME, "Chunk index out of bounds: {}", index);

        unsafe {
            match &self.data {
                ChunkData::Homogenous(state) => *state,
                ChunkData::Paletted4 { palette, indices } => {
                    let byte = *indices.get_unchecked(index >> 1);
                    let palette_index = if (index & 1) == 0 {
                        byte >> 4
                    } else {
                        byte & 0x0F
                    };
                    *palette.get_unchecked(palette_index as usize)
                }
                ChunkData::Paletted8 { palette, indices } => {
                    *palette.get_unchecked(*indices.get_unchecked(index) as usize)
                }
                ChunkData::Paletted16 { palette, indices } => {
                    *palette.get_unchecked(*indices.get_unchecked(index) as usize)
                }
            }
        }
    }

    pub fn set_block(&mut self, local: IVec3, state: BlockState) -> Result<(), ChunkError> {
        if (local.x as u32) >= 32 || (local.y as u32) >= 32 || (local.z as u32) >= 32 {
            return Err(ChunkError::OutOfLocalBounds);
        }
        if state.0 >= REGISTERED_STATE_COUNT.load(Ordering::Acquire) {
            return Err(ChunkError::UnregisteredBlockSet);
        }

        let index = local_to_index(local);

        // SAFETY: Bounds are manually checked at the start of the function
        unsafe {
            // If there's nothing to change we do not call `set_block_unchecked` as it would mark
            // the chunk as dirty for no reason
            if self.get_block_unchecked(index) == state {
                return Ok(());
            }
            self.set_block_unchecked(index, state);
        }

        Ok(())
    }

    /// Safely applies a single block modification inside this chunk.
    ///
    /// # Safety
    /// The caller should guarantee that `index` is strictly between `0` and `32767` (`CHUNK_VOLUME - 1`)
    /// and that the block id is actually registered
    #[inline]
    pub unsafe fn set_block_unchecked(&mut self, index: usize, state: BlockState) {
        debug_assert!(index < CHUNK_VOLUME, "Chunk index out of bounds: {}", index);

        let mut next_data = None;

        unsafe {
            match &mut self.data {
                ChunkData::Homogenous(current_state) => {
                    let new_palette = vec![*current_state, state];
                    let mut new_indices: Box<[u8; CHUNK_VOLUME / 2]> = vec![0u8; CHUNK_VOLUME / 2]
                        .into_boxed_slice()
                        .try_into()
                        .unwrap();

                    write_paletted4_unchecked(&mut new_indices[..], index, 1);

                    next_data = Some(ChunkData::Paletted4 {
                        palette: new_palette,
                        indices: new_indices,
                    });
                }

                ChunkData::Paletted4 { palette, indices } => {
                    if let Some(p_idx) = palette.iter().position(|&b| b == state) {
                        write_paletted4_unchecked(&mut indices[..], index, p_idx as u8);
                    } else if palette.len() < 16 {
                        let p_idx = palette.len() as u8;
                        palette.push(state);
                        write_paletted4_unchecked(&mut indices[..], index, p_idx);
                    } else {
                        next_data = Some(upgrade_4_to_8(palette, &indices[..], index, state));
                    }
                }

                ChunkData::Paletted8 { palette, indices } => {
                    if let Some(p_idx) = palette.iter().position(|&b| b == state) {
                        *indices.get_unchecked_mut(index) = p_idx as u8;
                    } else if palette.len() < 256 {
                        let p_idx = palette.len() as u8;
                        palette.push(state);
                        *indices.get_unchecked_mut(index) = p_idx;
                    } else {
                        next_data = Some(upgrade_8_to_16(palette, &indices[..], index, state));
                    }
                }

                ChunkData::Paletted16 { palette, indices } => {
                    if let Some(p_idx) = palette.iter().position(|&b| b == state) {
                        *indices.get_unchecked_mut(index) = p_idx as u16;
                    } else {
                        let p_idx = palette.len() as u16;
                        palette.push(state);
                        *indices.get_unchecked_mut(index) = p_idx;
                    }
                }
            }
        }

        if let Some(new_data) = next_data {
            self.data = new_data;
        }
    }

    /// Safely applies multiple block modifications inside this chunk using a maximum
    /// of one memory allocation.
    ///
    /// # Safety
    /// The caller should guarantee that all `usize` indices in the `deltas` slice are strictly between `0` and `32767` (`CHUNK_VOLUME - 1`).
    /// and that the block ids is actually registered
    pub unsafe fn set_block_batch_unchecked(&mut self, deltas: &[(usize, BlockState)]) {
        if deltas.is_empty() {
            return;
        }

        let mut unique_states = match &self.data {
            ChunkData::Homogenous(state) => vec![*state],
            ChunkData::Paletted4 { palette, .. }
            | ChunkData::Paletted8 { palette, .. }
            | ChunkData::Paletted16 { palette, .. } => palette.clone(),
        };

        for &(_, state) in deltas {
            if !unique_states.contains(&state) {
                unique_states.push(state);
            }
        }

        let required_size = unique_states.len();

        let needs_upgrade = match &self.data {
            ChunkData::Homogenous(_) => required_size > 1,
            ChunkData::Paletted4 { .. } => required_size > 16,
            ChunkData::Paletted8 { .. } => required_size > 256,
            ChunkData::Paletted16 { .. } => false,
        };

        unsafe {
            if !needs_upgrade {
                match &mut self.data {
                    ChunkData::Homogenous(_) => {}
                    ChunkData::Paletted4 { palette, indices } => {
                        for state in &unique_states {
                            if !palette.contains(state) {
                                palette.push(*state);
                            }
                        }
                        for &(idx, state) in deltas {
                            let p_idx = palette.iter().position(|&s| s == state).unwrap() as u8;
                            write_paletted4_unchecked(&mut indices[..], idx, p_idx);
                        }
                    }
                    ChunkData::Paletted8 { palette, indices } => {
                        for state in &unique_states {
                            if !palette.contains(state) {
                                palette.push(*state);
                            }
                        }
                        for &(idx, state) in deltas {
                            let p_idx = palette.iter().position(|&s| s == state).unwrap() as u8;
                            *indices.get_unchecked_mut(idx) = p_idx;
                        }
                    }
                    ChunkData::Paletted16 { palette, indices } => {
                        for state in &unique_states {
                            if !palette.contains(state) {
                                palette.push(*state);
                            }
                        }
                        for &(idx, state) in deltas {
                            let p_idx = palette.iter().position(|&s| s == state).unwrap() as u16;
                            *indices.get_unchecked_mut(idx) = p_idx;
                        }
                    }
                }
                return;
            }

            let new_palette: Vec<BlockState> = unique_states.into_iter().collect();
            let get_p_idx =
                |state: BlockState| new_palette.iter().position(|&s| s == state).unwrap();

            self.data = match required_size {
                0..=16 => {
                    let mut new_indices: Box<[u8; CHUNK_VOLUME / 2]> = vec![0u8; CHUNK_VOLUME / 2]
                        .into_boxed_slice()
                        .try_into()
                        .unwrap();

                    match &self.data {
                        ChunkData::Homogenous(state) => {
                            let p_idx = get_p_idx(*state) as u8;
                            if p_idx != 0 {
                                for i in 0..CHUNK_VOLUME {
                                    write_paletted4_unchecked(&mut new_indices[..], i, p_idx);
                                }
                            }
                        }
                        _ => unreachable!(),
                    }

                    for &(idx, state) in deltas {
                        write_paletted4_unchecked(
                            &mut new_indices[..],
                            idx,
                            get_p_idx(state) as u8,
                        );
                    }
                    ChunkData::Paletted4 {
                        palette: new_palette,
                        indices: new_indices,
                    }
                }

                17..=256 => {
                    let mut new_indices: Box<[u8; CHUNK_VOLUME]> = vec![0u8; CHUNK_VOLUME]
                        .into_boxed_slice()
                        .try_into()
                        .unwrap();

                    match &self.data {
                        ChunkData::Homogenous(state) => new_indices.fill(get_p_idx(*state) as u8),
                        ChunkData::Paletted4 { palette, indices } => {
                            for i in 0..CHUNK_VOLUME {
                                let byte = *indices.get_unchecked(i >> 1);
                                let old_p_idx = if (i & 1) == 0 { byte >> 4 } else { byte & 0x0F };
                                *new_indices.get_unchecked_mut(i) =
                                    get_p_idx(*palette.get_unchecked(old_p_idx as usize)) as u8;
                            }
                        }
                        _ => unreachable!(),
                    }

                    for &(idx, state) in deltas {
                        *new_indices.get_unchecked_mut(idx) = get_p_idx(state) as u8;
                    }
                    ChunkData::Paletted8 {
                        palette: new_palette,
                        indices: new_indices,
                    }
                }

                _ => {
                    let mut new_indices: Box<[u16; CHUNK_VOLUME]> = vec![0u16; CHUNK_VOLUME]
                        .into_boxed_slice()
                        .try_into()
                        .unwrap();

                    match &self.data {
                        ChunkData::Homogenous(state) => new_indices.fill(get_p_idx(*state) as u16),
                        ChunkData::Paletted4 { palette, indices } => {
                            for i in 0..CHUNK_VOLUME {
                                let byte = *indices.get_unchecked(i >> 1);
                                let old_p_idx = if (i & 1) == 0 { byte >> 4 } else { byte & 0x0F };
                                *new_indices.get_unchecked_mut(i) =
                                    get_p_idx(*palette.get_unchecked(old_p_idx as usize)) as u16;
                            }
                        }
                        ChunkData::Paletted8 { palette, indices } => {
                            for i in 0..CHUNK_VOLUME {
                                *new_indices.get_unchecked_mut(i) = get_p_idx(
                                    *palette.get_unchecked(*indices.get_unchecked(i) as usize),
                                )
                                    as u16;
                            }
                        }
                        _ => unreachable!(),
                    }

                    for &(idx, state) in deltas {
                        *new_indices.get_unchecked_mut(idx) = get_p_idx(state) as u16;
                    }
                    ChunkData::Paletted16 {
                        palette: new_palette,
                        indices: new_indices,
                    }
                }
            };
        }
    }
}

/// Bitwise insertion for 4-bit chunk palette compression.
///
/// # Safety
/// The caller should guarantee that `index` is strictly between `0` and `32767`
/// (`CHUNK_VOLUME - 1`), meaning `index >> 1` will not exceed the bounds of the
/// 16KB `indices` array.
#[inline(always)]
pub unsafe fn write_paletted4_unchecked(indices: &mut [u8], index: usize, val: u8) {
    let byte_index = index >> 1;
    unsafe {
        let byte = indices.get_unchecked_mut(byte_index);
        if (index & 1) == 0 {
            *byte = (*byte & 0x0F) | (val << 4);
        } else {
            *byte = (*byte & 0xF0) | val;
        }
    }
}

/// Internal helper to dynamically upgrade a Paletted4 chunk to Paletted8.
///
/// # Safety
/// The caller should guarantee `target_idx` is strictly bounded `0..32767`.
unsafe fn upgrade_4_to_8(
    old_palette: &[BlockState],
    old_indices: &[u8],
    target_idx: usize,
    new_state: BlockState,
) -> ChunkData {
    let mut new_palette = old_palette.to_vec();
    new_palette.push(new_state);
    let new_state_idx = (new_palette.len() - 1) as u8;

    let mut new_indices: Box<[u8; CHUNK_VOLUME]> = vec![0u8; CHUNK_VOLUME]
        .into_boxed_slice()
        .try_into()
        .unwrap();

    unsafe {
        for i in 0..CHUNK_VOLUME {
            if i == target_idx {
                *new_indices.get_unchecked_mut(i) = new_state_idx;
            } else {
                let byte = *old_indices.get_unchecked(i >> 1);
                *new_indices.get_unchecked_mut(i) =
                    if (i & 1) == 0 { byte >> 4 } else { byte & 0x0F };
            }
        }
    }

    ChunkData::Paletted8 {
        palette: new_palette,
        indices: new_indices,
    }
}

/// Internal helper to dynamically upgrade a Paletted8 chunk to Paletted16.
///
/// # Safety
/// The caller should guarantee `target_idx` is strictly bounded `0..32767`.
unsafe fn upgrade_8_to_16(
    old_palette: &[BlockState],
    old_indices: &[u8],
    target_idx: usize,
    new_state: BlockState,
) -> ChunkData {
    let mut new_palette = old_palette.to_vec();
    new_palette.push(new_state);
    let new_state_idx = (new_palette.len() - 1) as u16;

    let mut new_indices: Box<[u16; CHUNK_VOLUME]> = vec![0u16; CHUNK_VOLUME]
        .into_boxed_slice()
        .try_into()
        .unwrap();

    unsafe {
        for i in 0..CHUNK_VOLUME {
            if i == target_idx {
                *new_indices.get_unchecked_mut(i) = new_state_idx;
            } else {
                *new_indices.get_unchecked_mut(i) = *old_indices.get_unchecked(i) as u16;
            }
        }
    }

    ChunkData::Paletted16 {
        palette: new_palette,
        indices: new_indices,
    }
}
