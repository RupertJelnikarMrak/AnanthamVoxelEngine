//! Everything directly window related, behaviour to actions, but mainly just
//! a configuration wrapper for Bevy's window.

use bevy::prelude::*;
pub mod controls;

pub const ANANTHAM_WINDOW_TITLE: &str = "Anantham";

pub struct AnanthamWindowPlugin;

impl Plugin for AnanthamWindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, controls::handle_window_state);
    }
}
