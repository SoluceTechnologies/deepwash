use deepwash::tasks::clean::is_refused_path;
use std::path::{Path, PathBuf};

#[test]
fn refuses_root_home_and_scan_root() {
    let home = PathBuf::from("/Users/someone");
    let scan_root = PathBuf::from("/Users/someone/projects");
    assert!(is_refused_path(Path::new("/"), &home, Some(&scan_root)));
    assert!(is_refused_path(&home, &home, Some(&scan_root)));
    assert!(is_refused_path(&scan_root, &home, Some(&scan_root)));
    // a normal match under the scan root is allowed
    assert!(!is_refused_path(
        Path::new("/Users/someone/projects/app/node_modules"),
        &home,
        Some(&scan_root)
    ));
}
