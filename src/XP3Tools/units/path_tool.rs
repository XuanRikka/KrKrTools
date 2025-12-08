use walkdir::WalkDir;

use std::path::{Path, PathBuf};

pub fn get_all_files_walkdir<P: AsRef<Path>>(path: P) -> Vec<PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.into_path().strip_prefix("./").unwrap().to_path_buf())
        .collect()
}


pub fn path_str_handle(path: String) -> String {
    let path = path.replace("\\", "/");
    path.strip_prefix("./")
        .unwrap_or(&path)
        .to_string()
}
