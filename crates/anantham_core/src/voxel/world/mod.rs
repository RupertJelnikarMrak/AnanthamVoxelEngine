pub mod access;
pub mod generation;
pub mod math;

pub use access::{ChunkMap, WorldAccessError, WorldVoxelAccessor};
pub use math::{global_to_local, local_to_index};

use crate::state::AppState;
pub use bevy::prelude::*;

pub struct VoxelWorldPlugin;

impl Plugin for VoxelWorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<access::ChunkMap>();

        app.add_systems(
            OnEnter(AppState::InGame),
            (
                generation::spawn_test_world,
                generation::place_test_blocks.after(generation::spawn_test_world),
            ),
        );
    }
}
