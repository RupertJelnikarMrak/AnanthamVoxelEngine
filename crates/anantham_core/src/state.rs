use bevy::prelude::*;

/// The Top-Level state of the application.
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppState {
    #[default]
    Loading,
    InGame,
}
