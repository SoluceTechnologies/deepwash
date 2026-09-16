use deepwash::tasks::clean::{default_names, is_target, system_targets};
use std::fs;
use std::path::PathBuf;

#[test]
fn defaults_include_common_artifacts() {
    let names = default_names();
    for n in ["node_modules", ".next", "target", "dist", "build", ".cache",
              "__pycache__", ".venv", ".turbo", ".gradle"] {
        assert!(names.iter().any(|x| x == n), "missing {}", n);
    }
}

#[test]
fn is_target_matches_defaults_and_extras() {
    let mut names = default_names();
    names.push("mycache".to_string());
    assert!(is_target("node_modules", &names));
    assert!(is_target("mycache", &names));
    assert!(!is_target("src", &names));
}

#[test]
fn system_targets_are_os_specific_and_existing_only() {
    // Build a fake home with only a Trash dir present.
    let home = std::env::temp_dir().join(format!("dw_home_{}", std::process::id()));
    let _ = fs::remove_dir_all(&home);
    let mac_trash = home.join(".Trash");
    fs::create_dir_all(&mac_trash).unwrap();

    let mac = system_targets(&home, "macos");
    assert!(mac.iter().any(|t| t.path == mac_trash));
    // ~/.cache not created → excluded
    assert!(!mac.iter().any(|t| t.path == home.join(".cache")));

    let linux = system_targets(&home, "linux");
    // linux trash path not created → excluded
    assert!(!linux.iter().any(|t| t.path == PathBuf::from(&home).join(".local/share/Trash")));

    fs::remove_dir_all(&home).unwrap();
}
