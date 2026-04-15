pub mod input;
pub mod plugin;
pub mod window;

pub use bevy;

pub mod vendor {
    pub use leafwing_input_manager;
}

pub mod prelude {
    pub use bevy::prelude::*;

    pub use crate::input::actions::CoreAction;
}
