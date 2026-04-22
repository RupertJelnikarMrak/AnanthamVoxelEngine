use super::registry::BlockRegistry;
use bevy::prelude::*;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
pub struct BlockMeta {
    pub name: String,
    pub description: String,
    pub texture_id: String,
    #[serde(default)]
    pub is_divisible: bool,
    #[serde(default)]
    pub custom_states: u32,
}

pub fn discover_blocks_from_disk(mut registry: ResMut<BlockRegistry>) {
    info!("Starting Block Discovery Phase...");

    registry.register_block("core", "air", "Air", 1, false);

    let data_dir = std::path::Path::new("data");

    if !data_dir.exists() {
        warn!("Data directory not found at {:?}", data_dir);
        return;
    }

    for namespace_entry in std::fs::read_dir(data_dir).expect("Failed to read data directory") {
        let namespace_entry = namespace_entry.unwrap();
        let namespace_path = namespace_entry.path();

        if namespace_path.is_dir() {
            let namespace = namespace_path.file_name().unwrap().to_str().unwrap();
            let blocks_dir = namespace_path.join("blocks");

            if blocks_dir.exists() && blocks_dir.is_dir() {
                for block_entry in
                    fs::read_dir(&blocks_dir).expect("Failed to read blocks directory")
                {
                    let block_entry = block_entry.unwrap();
                    let block_path = block_entry.path();

                    if block_path.is_dir() {
                        let name = block_path.file_name().unwrap().to_str().unwrap();
                        let meta_path = block_path.join("meta.ron");

                        if meta_path.exists() {
                            let ron_str = std::fs::read_to_string(&meta_path)
                                .expect("Failed to read meta.ron");
                            match ron::from_str::<BlockMeta>(&ron_str) {
                                Ok(meta) => {
                                    let state_count = if meta.is_divisible {
                                        256 + meta.custom_states
                                    } else {
                                        meta.custom_states.max(1)
                                    };

                                    registry.register_block(
                                        namespace,
                                        name,
                                        &meta.name,
                                        state_count,
                                        meta.is_divisible,
                                    );

                                    debug!("Discovered & Registered Block: {}:{}", namespace, name);
                                }
                                Err(e) => warn!(
                                    "Syntax error in meta.ron for {}:{}: {}",
                                    namespace, name, e
                                ),
                            }
                        } else {
                            warn!(
                                "Block folder '{}:{}' is missing a meta.ron file!",
                                namespace, name
                            );
                        }
                    }
                }
            }
        }
    }

    info!(
        "Block Discovery Complete. Total block states allocated: {}",
        crate::voxel::block::registry::REGISTERED_STATE_COUNT
            .load(std::sync::atomic::Ordering::Relaxed)
    );
}
