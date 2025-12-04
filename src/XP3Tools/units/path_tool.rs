use walkdir::{WalkDir, DirEntry};

use std::path::{Path, PathBuf};


pub fn get_all_files_walkdir(path: &Path) -> Vec<PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.into_path())
        .collect()
}
