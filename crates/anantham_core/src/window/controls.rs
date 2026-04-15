//! Behaviour logic for actions related to controlling the window.

use crate::input::actions::CoreAction;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow, WindowMode};
use leafwing_input_manager::prelude::ActionState;

pub fn handle_window_state(
    mut window_query: Query<(&mut Window, &mut CursorOptions), With<PrimaryWindow>>,
    action_state: Res<ActionState<CoreAction>>,
) {
    let Ok((mut window, mut cursor_options)) = window_query.single_mut() else {
        return;
    };

    if action_state.just_pressed(&CoreAction::ToggleFullscreen) {
        window.mode = match window.mode {
            WindowMode::Windowed => WindowMode::BorderlessFullscreen(MonitorSelection::Current),
            _ => WindowMode::Windowed,
        };
    }

    if action_state.just_pressed(&CoreAction::ReleaseMouse) {
        cursor_options.grab_mode = CursorGrabMode::None;
        cursor_options.visible = true;
    }

    if action_state.just_pressed(&CoreAction::Interact) {
        cursor_options.grab_mode = CursorGrabMode::Confined;
        cursor_options.visible = false;
    }
}
