//! This module acts as the translation layer between raw physical inputs
//! (keyboard, mouse, gamepad) and semantic game actions (e.g., `Interact`, `MoveCamera`).
//!
//! # Architecture
//! The engine strictly decouples **Input Definition** from **Input Execution**:
//! * **Definition (`input` module):** Registers the `CoreAction` enum and maps physical keystrokes
//!   to those semantic actions.
//! * **Execution (Domain modules):** Other systems (like the `window` or `spatial` modules) query
//!   the resulting `ActionState` resource to perform logic.
//!
//! # Roadmap: The Modding Keybind Problem
//! In a heavily modded ecosystem, raw keybinds inevitably conflict.
//!
//! Future iterations should drop the hardcoded defaults in favor of parsing an efficient
//! configuration file at startup. This abstraction should natively support complex modding
//! requirements like input chords, leader-key sequences, and context-sensitive radial menus
//! without requiring any alterations to the core gameplay systems or downstream Tier 1 Plugins.

pub mod actions;
pub mod prelude {
    pub use super::actions::CoreAction;
    pub use leafwing_input_manager::prelude::*;
}

use actions::{CoreAction, default_input_map};
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<CoreAction>::default());

        app.init_resource::<ActionState<CoreAction>>();

        app.insert_resource(default_input_map());
    }
}
