//! Collection of all default actions the AnanthamCore defines behaviour for
//! and their default mappings.

use bevy::prelude::*;
use bevy::reflect::Reflect;
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum CoreAction {
    #[actionlike(DualAxis)]
    MoveCamera,
    #[actionlike(DualAxis)]
    LookAround,
    ToggleFullscreen,
    ReleaseMouse,
    Interact,
}

pub fn default_input_map() -> InputMap<CoreAction> {
    let mut map = InputMap::default();

    map.insert(CoreAction::ToggleFullscreen, KeyCode::F11);
    map.insert(CoreAction::ReleaseMouse, KeyCode::Escape);
    map.insert(CoreAction::Interact, MouseButton::Left);

    map.insert_dual_axis(CoreAction::MoveCamera, VirtualDPad::wasd());

    map.insert_dual_axis(CoreAction::LookAround, MouseMove::default());

    map
}
