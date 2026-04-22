use bevy::prelude::*;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum EngineState {
    #[default]
    Boot,
    LoadAssets,     // AssetServer is reading asset files from disk
    RegisterBlocks, // Parsing RONs, allocationd IDs, padding PropertyRegistries
    InGame,         // Game loop runs
}
