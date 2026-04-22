use bevy::prelude::*;
use serde::Deserialize;

use crate::voxel::block::BlockPropertyAsset;

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

#[derive(Asset, TypePath, Clone, Debug, Deserialize)]
pub struct MeshingAssets {
    pub is_visible: bool,
    #[serde(default)]
    pub is_transparent: bool,
    #[serde(default)]
    pub material_id: u16,
}

/// Bridges the disk data to the runtime flat array
impl BlockPropertyAsset for MeshingAssets {
    type RuntimeData = MeshingAttributes;

    fn to_runtime(&self) -> Self::RuntimeData {
        MeshingAttributes {
            is_visible: self.is_visible,
            is_transparent: self.is_transparent,
            material_id: self.material_id,
        }
    }
}
