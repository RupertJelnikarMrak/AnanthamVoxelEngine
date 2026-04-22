use super::{BlockPropertyAsset, PropertyRegistry};
use crate::voxel::block::BlockRegistry;
use bevy::prelude::*;
use std::path::Path;

/// Helper to extract "stone" from "data/anantham/blocks/stone/mesh.ron"
fn extract_block_id(asset_path: &bevy::asset::AssetPath) -> Option<String> {
    let path = Path::new(asset_path.path());

    let block_name = path.parent()?.file_name()?.to_str()?;
    let namespace = path.parent()?.parent()?.file_name()?.to_str()?;

    Some(format!("{}:{}", namespace, block_name))
}

/// Runs once during Init. Matches loaded assets to the BlockRegistry and populates arrays.
pub fn populate_property_arrays<A: BlockPropertyAsset>(
    assets: Res<Assets<A>>,
    asset_server: Res<AssetServer>,
    block_registry: Res<BlockRegistry>,
    property_registry: Res<PropertyRegistry<A::RuntimeData>>,
) {
    for (id, asset) in assets.iter() {
        if let Some(path) = asset_server.get_path(id)
            && let Some(block_name) = extract_block_id(&path)
            && let Some(block) = block_registry.get_block_by_name(&block_name)
        {
            let runtime_data = asset.to_runtime();

            for offset in 0..block.state_count {
                property_registry.set(block.base_id + offset, runtime_data.clone());
            }
        }
    }
}

/// Runs continuously. Updates the array dynamically if a file is edited.
pub fn hot_reload_property<A: BlockPropertyAsset>(
    mut messages: MessageReader<AssetEvent<A>>,
    assets: Res<Assets<A>>,
    asset_server: Res<AssetServer>,
    block_registry: Res<BlockRegistry>,
    property_registry: Res<PropertyRegistry<A::RuntimeData>>,
) {
    for message in messages.read() {
        if let AssetEvent::Modified { id } = message
            && let Some(new_data) = assets.get(*id)
            && let Some(path) = asset_server.get_path(*id)
            && let Some(block_name) = extract_block_id(&path)
            && let Some(block) = block_registry.get_block_by_name(&block_name)
        {
            let runtime_data = new_data.to_runtime();

            // Instantly overwrite without reallocating
            for offset in 0..block.state_count {
                property_registry.set(block.base_id + offset, runtime_data.clone());
            }
            info!("Hot-reloaded property for block: {}", block_name);
        }
    }
}
