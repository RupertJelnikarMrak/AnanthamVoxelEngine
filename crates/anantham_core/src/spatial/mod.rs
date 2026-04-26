pub mod camera;

use crate::state::AppState;
use bevy::prelude::*;

pub struct AnanthamSpatialPlugin;

impl Plugin for AnanthamSpatialPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), camera::spawn_camera);
        app.add_systems(Update, camera::camera_movement_system);
    }
}
