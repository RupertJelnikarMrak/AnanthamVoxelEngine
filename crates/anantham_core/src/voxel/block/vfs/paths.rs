use directories::ProjectDirs;
use std::path::PathBuf;

pub struct VfsPaths;

impl VfsPaths {
    pub fn internal_core() -> PathBuf {
        if cfg!(debug_assertions) {
            std::env::current_dir()
                .expect("Failed to get the current working directory")
                .join("data/anantham")
        } else {
            std::env::current_exe()
                .expect("Failed to get executable path")
                .parent()
                .expect("Executable has no parent directory")
                .join("data/anantham")
        }
    }

    pub fn external_user_data() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("com", "Anantham", "AVE") {
            proj_dirs.data_dir().join("data")
        } else {
            std::env::current_dir().unwrap().join("user_data")
        }
    }
}
