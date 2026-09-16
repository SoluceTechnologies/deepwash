use deepwash::tasks::clean::{default_names, scan};
use std::fs;
use std::path::PathBuf;

fn tmp_root(tag: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("dw_scan_{}_{}", tag, std::process::id()));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).unwrap();
    p
}

#[test]
fn scan_finds_targets_and_prunes_nested() {
    let root = tmp_root("basic");
    // a/node_modules/pkg/index.js  (outer match; inner node_modules must NOT be visited)
    fs::create_dir_all(root.join("a/node_modules/pkg/node_modules")).unwrap();
    fs::write(root.join("a/node_modules/pkg/index.js"), b"hello").unwrap();
    // a/src/main.rs (recursed, no match)
    fs::create_dir_all(root.join("a/src")).unwrap();
    fs::write(root.join("a/src/main.rs"), b"fn main() {}").unwrap();
    // b/target
    fs::create_dir_all(root.join("b/target")).unwrap();
    fs::write(root.join("b/target/artifact.bin"), vec![0u8; 100]).unwrap();

    let names = default_names();
    let matches = scan(&root, &names);

    let kinds: Vec<&str> = matches.iter().map(|m| m.kind.as_str()).collect();
    // exactly one node_modules (outer, pruned) and one target
    assert_eq!(kinds.iter().filter(|k| **k == "node_modules").count(), 1);
    assert_eq!(kinds.iter().filter(|k| **k == "target").count(), 1);
    // outer node_modules path is the shallow one
    let nm = matches.iter().find(|m| m.kind == "node_modules").unwrap();
    assert_eq!(nm.path, root.join("a/node_modules"));
    assert!(nm.size >= 5); // includes index.js contents

    fs::remove_dir_all(&root).unwrap();
}

#[test]
#[cfg(unix)]
fn scan_skips_symlinked_dirs() {
    use std::os::unix::fs::symlink;
    let root = tmp_root("symlink");
    fs::create_dir_all(root.join("real/node_modules")).unwrap();
    fs::write(root.join("real/node_modules/f"), vec![0u8; 50]).unwrap();
    // link -> real ; a scan starting at root/links should not follow into real
    fs::create_dir_all(root.join("links")).unwrap();
    symlink(root.join("real"), root.join("links/alias")).unwrap();

    let matches = scan(&root.join("links"), &default_names());
    assert!(matches.is_empty(), "symlinked dir must not be traversed");

    fs::remove_dir_all(&root).unwrap();
}
