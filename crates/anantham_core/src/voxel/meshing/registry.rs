use crate::voxel::block::BlockState;
use bevy::prelude::*;
use std::sync::Arc;

/// Data associated with a block, used to determine how a block gets meshed
#[derive(Clone, Copy, Debug)]
pub struct MeshingAttributes {
    pub is_visible: bool,
    pub is_transparent: bool,
    pub material_id: u16,
}

impl Default for MeshingAttributes {
    fn default() -> Self {
        Self::AIR
    }
}

impl MeshingAttributes {
    /// The standard solid block (Stone, Dirt, Wood).
    pub const SOLID: Self = Self {
        is_visible: true,
        is_transparent: false,
        material_id: 0,
    };

    /// The standard empty block (Air, Void).
    pub const AIR: Self = Self {
        is_visible: false,
        is_transparent: true,
        material_id: 0,
    };

    // --- BUILDER PATTERN ---

    #[inline]
    pub fn with_visible(mut self, visible: bool) -> Self {
        self.is_visible = visible;
        self
    }

    #[inline]
    pub fn with_transparent(mut self, transparent: bool) -> Self {
        self.is_transparent = transparent;
        self
    }

    #[inline]
    pub fn with_material(mut self, id: u16) -> Self {
        self.material_id = id;
        self
    }
}

/// A centralized registry holding flat arrays of meshing attributes indexed directly by BlockState (u32)
#[derive(Resource, Default, Clone)]
pub struct MeshingRegistry {
    pub meshing_data: Arc<Vec<MeshingAttributes>>,
}

impl MeshingRegistry {
    /// Safely sets meshing data, resizing the internal array if necessary.
    pub fn set_meshing(&mut self, state: BlockState, attributes: MeshingAttributes) {
        let data = Arc::make_mut(&mut self.meshing_data);

        let id = state.0 as usize;
        if id >= data.len() {
            data.resize(id + 1, MeshingAttributes::default());
        }
        data[id] = attributes;
    }

    /// Safe getter. Returns default attributes (AIR/Empty) if the BlockState is out of bounds.
    /// Useful for UI, debug tools, or handling corrupted chunk saves safely.
    #[inline]
    pub fn get_meshing(&self, state: BlockState) -> MeshingAttributes {
        self.meshing_data
            .get(state.0 as usize)
            .copied()
            .unwrap_or_default()
    }

    /// Super-fast getter for the meshing hot-loop.
    ///
    /// # Safety
    /// The caller must ensure that `state.0` is strictly less than the length of `meshing_data`.
    /// Passing an unregistered or out-of-bounds BlockState will cause Undefined Behavior.
    #[inline(always)]
    pub unsafe fn get_meshing_unchecked(&self, state: BlockState) -> &MeshingAttributes {
        debug_assert!(
            (state.0 as usize) < self.meshing_data.len(),
            "BlockState {} out of bounds for MeshingRegistry!",
            state.0
        );

        unsafe { self.meshing_data.get_unchecked(state.0 as usize) }
    }
}
