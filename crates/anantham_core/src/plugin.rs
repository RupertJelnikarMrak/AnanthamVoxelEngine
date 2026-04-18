//! The master plugin registry for the core engine.
//!
//! This module bundles all the individual domain plugins (like voxel, spatial,
//! and render bridging) into a single entry point.
//!
//! **Note for Modders:** This file follows the required architecture expected
//! from all Anantham Plugins, official or community made.

use crate::input::AnanthamInputPlugin;
use crate::render_bridge::RenderBridgePlugin;
use crate::voxel::VoxelCorePlugin;
use crate::window::{ANANTHAM_WINDOW_TITLE, AnanthamWindowPlugin};
use bevy::prelude::*;
use bevy::window::WindowResolution;
use std::time::Duration;

pub struct AnanthamCorePlugin;

impl Plugin for AnanthamCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: ANANTHAM_WINDOW_TITLE.to_string(),
                resolution: WindowResolution::new(1280, 720),
                ..default()
            }),
            ..default()
        }));
        app.add_plugins((
            AnanthamWindowPlugin,
            AnanthamInputPlugin,
            RenderBridgePlugin,
            VoxelCorePlugin,
        ));

        // Set tick speed to 50ms / 20 times a second.
        app.insert_resource(Time::<Fixed>::from_duration(Duration::from_millis(50)));
    }
}
