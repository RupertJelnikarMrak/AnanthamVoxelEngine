use bevy::prelude::*;
use std::sync::{Arc, RwLock};

/// Data associated with a block, used to determine how a block gets meshed
#[derive(Clone, Copy, Debug)]
pub struct MeshingAttributes {
    packed_data: u32,
}

impl Default for MeshingAttributes {
    fn default() -> Self {
        Self::AIR
    }
}

impl MeshingAttributes {
    pub const AIR: Self = Self::new(false, true, 0, 0);
    pub const SOLID: Self = Self::new(true, false, 0, 0);

    #[inline(always)]
    pub const fn new(
        is_visible: bool,
        is_transparent: bool,
        fractional_mask: u8,
        material_id: u16,
    ) -> Self {
        let flags = (is_visible as u32) | ((is_transparent as u32) << 1);
        let mask = (fractional_mask as u32) << 8;
        let mat = (material_id as u32) << 16;

        Self {
            packed_data: flags | mask | mat,
        }
    }

    #[inline(always)]
    pub fn is_visible(&self) -> bool {
        (self.packed_data & 0x1) != 0
    }

    #[inline(always)]
    pub fn is_transparent(&self) -> bool {
        (self.packed_data & 0x2) != 0
    }

    #[inline(always)]
    pub fn fractional_mask(&self) -> u8 {
        ((self.packed_data >> 8) & 0xFF) as u8
    }

    #[inline(always)]
    pub fn material_id(&self) -> u16 {
        ((self.packed_data >> 16) & 0xFFFF) as u16
    }
}

#[derive(Resource, Clone, Default)]
pub struct MeshingRegistry {
    pub data: Arc<RwLock<Vec<MeshingAttributes>>>,
}

impl MeshingRegistry {
    pub fn set(&self, state_id: u32, value: MeshingAttributes) {
        let mut write_lock = self.data.write().unwrap();

        let target_len = (state_id as usize) + 1;
        if target_len > write_lock.len() {
            write_lock.resize(target_len, MeshingAttributes::AIR);
        }

        write_lock[state_id as usize] = value;
    }

    /// Writes multiple states starting from a base ID using a single lock and memcpy.
    pub fn set_batch(&self, base_id: u32, values: &[MeshingAttributes]) {
        if values.is_empty() {
            return;
        }

        let mut write_lock = self.data.write().unwrap();

        let start = base_id as usize;
        let end = start + values.len();

        if end > write_lock.len() {
            write_lock.resize(end, MeshingAttributes::AIR);
        }

        // Instantly copies the contiguous memory block
        write_lock[start..end].copy_from_slice(values);
    }
}
