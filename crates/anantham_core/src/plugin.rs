//! The master plugin registry for the core engine.
//!
//! This module bundles all the individual domain plugins (like voxel, spatial,
//! and render bridging) into a single entry point.
//!
//! **Note for Modders:** This file follows the required architecture expected
//! from all Anantham Plugins, official or community made.

use crate::input::InputPlugin as AnanthamInputPlugin;
use crate::window::{ANANTHAM_WINDOW_TITLE, AnanthamWindowPlugin};
use bevy::prelude::*;

pub struct AnanthamCorePlugin;

impl Plugin for AnanthamCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: ANANTHAM_WINDOW_TITLE.to_string(),
                ..default()
            }),
            ..default()
        }));
        app.add_plugins((AnanthamWindowPlugin, AnanthamInputPlugin));
    }
}
