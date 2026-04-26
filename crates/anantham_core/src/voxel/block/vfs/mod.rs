pub mod paths;

use bevy::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use walkdir::WalkDir;

#[derive(Debug)]
pub struct VfsState {
    pub internal_core: PathBuf,
    pub active_packs: Vec<PathBuf>,
    /// Locked after boot: Maps Virtual Path -> Active Physical Path
    pub virtual_index: HashMap<PathBuf, PathBuf>,
    /// Locked after boot: Stores unique (Namespace, BlockName) pairs
    pub discovered_blocks: HashMap<(String, String), PathBuf>,
}

impl Default for VfsState {
    fn default() -> Self {
        Self {
            internal_core: paths::VfsPaths::internal_core(),
            active_packs: Vec::new(),
            virtual_index: HashMap::new(),
            discovered_blocks: HashMap::new(),
        }
    }
}

#[derive(Resource, Clone)]
pub struct BlockDataVfs {
    pub inner: Arc<RwLock<VfsState>>,
}

impl Default for BlockDataVfs {
    fn default() -> Self {
        Self {
            inner: Arc::new(RwLock::new(VfsState::default())),
        }
    }
}

impl BlockDataVfs {
    pub fn build_index(&self) {
        let mut write_guard = self.inner.write().unwrap();
        let mut new_index = HashMap::new();
        let mut new_blocks = HashMap::new();

        let mut search_roots = vec![write_guard.internal_core.clone()];
        search_roots.extend(write_guard.active_packs.iter().cloned());

        for root in search_roots {
            if !root.exists() {
                continue;
            }

            for entry in WalkDir::new(&root).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    let physical_path = entry.path().to_path_buf();

                    if let Ok(virtual_path) = physical_path.strip_prefix(&root) {
                        new_index.insert(virtual_path.to_path_buf(), physical_path.clone());

                        if virtual_path.file_name().and_then(|s| s.to_str()) == Some("meta.ron") {
                            let mut components = physical_path.components().rev();
                            components.next(); // skip meta.ron

                            if let Some(name_comp) = components.next()
                                && let Some(dir_comp) = components.next()
                                && dir_comp.as_os_str() == "blocks"
                                && let Some(ns_comp) = components.next()
                            {
                                let namespace = ns_comp.as_os_str().to_string_lossy().into_owned();
                                let block_name =
                                    name_comp.as_os_str().to_string_lossy().into_owned();

                                new_blocks.insert((namespace, block_name), physical_path.clone());
                            }
                        }
                    }
                }
            }
        }

        write_guard.virtual_index = new_index;
        write_guard.discovered_blocks = new_blocks;
    }
}
