use std::path::{Path, PathBuf};

/// Built-in artifact directory names pruned by default.
pub fn default_names() -> Vec<String> {
    [
        "node_modules", ".next", "target", "dist", "build", ".cache",
        "__pycache__", ".venv", ".turbo", ".gradle",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

/// True if `name` is in the target set.
pub fn is_target(name: &str, names: &[String]) -> bool {
    names.iter().any(|n| n == name)
}

/// A system location cleaned by `--system`.
pub struct SystemTarget {
    pub path: PathBuf,
    /// If true, remove the directory's contents but keep the directory itself.
    pub empty_contents: bool,
}

/// Per-OS trash + cache targets under `home`. Only existing paths are returned.
/// `os` is `std::env::consts::OS` ("macos", "linux", ...).
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
