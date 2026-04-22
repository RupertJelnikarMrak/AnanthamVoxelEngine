pub mod loader;
pub mod registry;
pub mod systems;

use bevy::prelude::*;
use serde::de::DeserializeOwned;

pub use registry::PropertyRegistry;

/// Any Block Property file (.ron) must implement this trait.
/// It defines both the asset data (from disk) and how it maps to the runtime memory.
pub trait BlockPropertyAsset: Asset + Clone + DeserializeOwned {
    /// The runtime type that will be stored in the flat array for fast access.
    type RuntimeData: Clone + Default + Send + Sync + 'static;

    /// Converts the parsed disk data into the optimized runtime data.
    fn to_runtime(&self) -> Self::RuntimeData;
}

/// A SystemSet to ensure block IDs are discovered BEFORE properties try to populate.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum BlockInitSet {
    Discover, // Runs first: scans folders, assigns BlockStates, runs padders
    Populate, // Runs second: reads assets, populates the padded property arrays
}

pub trait AppBlockPropertyExt {
    /// Registers a new block property.
    /// Automatically handles asset loading, array padding, initialization, and hot-reloading.
    fn register_block_property<A: BlockPropertyAsset>(
        &mut self,
        file_extension: &'static str,
    ) -> &mut Self;
}

impl AppBlockPropertyExt for App {
    fn register_block_property<A: BlockPropertyAsset>(
        &mut self,
        file_extension: &'static str,
    ) -> &mut Self {
        use crate::state::EngineState;
        use std::sync::Arc;

        // 1. Initialize the Runtime Registry
        let registry = PropertyRegistry::<A::RuntimeData>::default();

        // 2. Create the hook that allows the BlockRegistry to pad this array
        let hook = Box::new(registry::PadderHook {
            data_ref: Arc::clone(&registry.data),
        });

        // 3. Ensure BlockRegistry exists, then register the padder hook
        if !self
            .world()
            .contains_resource::<crate::voxel::block::BlockRegistry>()
        {
            self.init_resource::<crate::voxel::block::BlockRegistry>();
        }
        self.world_mut()
            .resource_mut::<crate::voxel::block::BlockRegistry>()
            .register_property_array(hook);

        // 4. Setup Bevy Assets, Loaders, and Systems
        self.init_asset::<A>()
            .register_asset_loader(loader::BlockPropertyLoader::<A>::new(file_extension))
            .insert_resource(registry)
            // Populate the arrays during the RegisterBlocks state (after discovery)
            .add_systems(
                OnEnter(EngineState::RegisterBlocks),
                systems::populate_property_arrays::<A>.in_set(BlockInitSet::Populate),
            )
            // Listen for hot-reloads during the game
            .add_systems(
                Update,
                systems::hot_reload_property::<A>.run_if(in_state(EngineState::InGame)),
            );

        self
    }
}
