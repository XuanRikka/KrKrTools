use std::env;
use walkdir::WalkDir;

use std::path::{Path, PathBuf};

pub fn get_all_files_walkdir<P: AsRef<Path>>(path: P) -> Vec<PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.into_path().to_path_buf())
        .collect()
}


pub fn path_str_handle(path: String) -> String {
    let path = path.replace("\\", "/");
    path.strip_prefix("./")
        .unwrap_or(&path)
        .to_string()
}


pub fn absolute_to_relative(base: &Path, path: &Path) -> PathBuf {
    let base_abs = match base.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            panic!("无法解析基准目录 {:?}: {}", base.display(), e);
        }
    };

    let path_abs = match path.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            panic!("无法解析路径 {:?}: {}", path.display(), e);
        }
    };
    
    match path_abs.strip_prefix(&base_abs) {
        Ok(rel) => rel.to_path_buf(),
        Err(_) => {
            panic!(
                "路径 {:?} 不在基准目录 {:?} 下，无法生成相对路径",
                path.display(),
                base.display()
            );
        }
    }
}


pub fn get_cwd() -> PathBuf {
    env::current_dir().expect("无法获取当前工作目录")
}
