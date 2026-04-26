pub mod data;
pub mod pipeline;
pub mod registry;
pub mod shape;
pub mod vfs;

pub use data::{Block, BlockState};
pub use registry::{BlockRegistry, REGISTERED_STATE_COUNT};

use bevy::prelude::*;

use crate::state::AppState;

pub struct VoxelBlockPlugin;

impl Plugin for VoxelBlockPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BlockRegistry>();
        app.init_resource::<vfs::BlockDataVfs>();

        app.add_systems(
            OnEnter(AppState::Loading),
            pipeline::register_blocks_from_vfs,
        );
    }
}
