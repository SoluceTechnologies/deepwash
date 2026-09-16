use std::path::{Path, PathBuf};

pub fn default_names() -> Vec<String> {
    [
        "node_modules", ".next", "target", "dist", "build", ".cache",
        "__pycache__", ".venv", ".turbo", ".gradle",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

pub fn is_target(name: &str, names: &[String]) -> bool {
    names.iter().any(|n| n == name)
}

pub struct SystemTarget {
    pub path: PathBuf,
    pub empty_contents: bool,
}

pub fn system_targets(home: &Path, os: &str) -> Vec<SystemTarget> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    match os {
        "macos" => candidates.push(home.join(".Trash")),
        _ => candidates.push(home.join(".local/share/Trash")),
    }
    candidates.push(home.join(".cache"));

    candidates
        .into_iter()
        .filter(|p| p.exists())
        .map(|path| SystemTarget { path, empty_contents: true })
        .collect()
}
