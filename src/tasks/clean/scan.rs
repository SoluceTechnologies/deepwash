use crate::tasks::clean::is_target;
use std::fs;
use std::path::{Path, PathBuf};

/// A matched artifact directory.
pub struct Match {
    pub path: PathBuf,
    pub size: u64,
    pub kind: String,
}

/// Recursive byte total of a directory. Skips symlinks; tolerates errors.
pub fn dir_size(path: &Path) -> u64 {
    let mut total = 0;
    let entries = match fs::read_dir(path) {
        Ok(e) => e,
        Err(_) => return 0,
    };
    for entry in entries.flatten() {
        let meta = match entry.path().symlink_metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        if meta.file_type().is_symlink() {
            continue;
        }
        if meta.is_dir() {
            total += dir_size(&entry.path());
        } else {
            total += meta.len();
        }
    }
    total
}

/// Walk `root`, recording target directories without descending into them.
pub fn scan(root: &Path, names: &[String]) -> Vec<Match> {
    let mut matches = Vec::new();
    walk(root, names, &mut matches);
    matches
}

fn walk(dir: &Path, names: &[String], matches: &mut Vec<Match>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return, // unreadable dir: skip silently
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let meta = match path.symlink_metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        if meta.file_type().is_symlink() || !meta.is_dir() {
            continue;
        }
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        if is_target(&name, names) {
            matches.push(Match {
                size: dir_size(&path),
                kind: name,
                path,
            });
            // prune: do not descend
        } else {
            walk(&path, names, matches);
        }
    }
}
