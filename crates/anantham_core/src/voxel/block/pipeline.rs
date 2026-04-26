use super::registry::BlockRegistry;
use super::vfs::BlockDataVfs;
use crate::state::AppState;
use crate::voxel::meshing::MeshingAttributes;
use crate::voxel::meshing::registry::MeshingRegistry;
use bevy::prelude::*;
use serde::Deserialize;
use std::fs;

fn is_visible_default() -> bool {
    true
}

#[derive(Deserialize)]
pub struct BlockMeta {
    pub texture_id: String,
    #[serde(default)]
    pub is_fractional: bool,
    #[serde(default = "is_visible_default")]
    pub is_visible: bool,
    #[serde(default)]
    pub is_transparent: bool,
}

pub fn register_blocks_from_vfs(
    mut block_registry: ResMut<BlockRegistry>,
    meshing_registry: ResMut<MeshingRegistry>,
    vfs: Res<BlockDataVfs>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    info!("Starting Block Registry Phase...");

    block_registry.register_block("anantham", "air", false);

    vfs.build_index();

    let vfs_read = vfs.inner.read().unwrap();

    let mut batch = Vec::with_capacity(256);

    for ((namespace, name), meta_path) in &vfs_read.discovered_blocks {
        let ron_str = fs::read_to_string(meta_path).expect("Failed to read meta.ron");

        if let Ok(meta) = ron::from_str::<BlockMeta>(&ron_str) {
            let block = block_registry.register_block(namespace, name, meta.is_fractional);

            batch.clear();

            for offset in 0..block.state_count {
                let fractional_mask = offset as u8;

                batch.push(MeshingAttributes::new(
                    meta.is_visible,
                    meta.is_transparent,
                    fractional_mask,
                    0,
                ));
            }
            meshing_registry.set_batch(block.base_id, &batch);
            info!("Discovered & Registered Block: {}:{}", namespace, name);
        } else {
            warn!("Syntax error in meta.ron for {}:{}", namespace, name);
        }
    }

    info!(
        "Block Discovery Complete. Total block states allocated: {}",
        crate::voxel::block::registry::REGISTERED_STATE_COUNT
            .load(std::sync::atomic::Ordering::Relaxed)
    );

    next_state.set(AppState::InGame);
}
