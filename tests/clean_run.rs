use deepwash::tasks::clean::{run, CleanOpts, default_names};
use std::fs;
use std::path::PathBuf;

fn tmp(tag: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("dw_run_{}_{}", tag, std::process::id()));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).unwrap();
    p
}

#[test]
fn execute_deletes_matches_and_keeps_others() {
    let root = tmp("exec");
    fs::create_dir_all(root.join("app/node_modules/pkg")).unwrap();
    fs::write(root.join("app/node_modules/pkg/blob"), vec![0u8; 1000]).unwrap();
    fs::create_dir_all(root.join("app/src")).unwrap();
    fs::write(root.join("app/src/main.rs"), b"fn main(){}").unwrap();

    run(CleanOpts {
        path: Some(root.clone()),
        execute: true,
        min_size: None,
        system: false,
        names: default_names(),
    });

    assert!(!root.join("app/node_modules").exists(), "node_modules should be deleted");
    assert!(root.join("app/src/main.rs").exists(), "non-match must survive");

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn dry_run_deletes_nothing() {
    let root = tmp("dry");
    fs::create_dir_all(root.join("node_modules")).unwrap();
    fs::write(root.join("node_modules/f"), vec![0u8; 100]).unwrap();

    run(CleanOpts {
        path: Some(root.clone()),
        execute: false,
        min_size: None,
        system: false,
        names: default_names(),
    });

    assert!(root.join("node_modules").exists(), "dry-run must not delete");

    fs::remove_dir_all(&root).unwrap();
}
