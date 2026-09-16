mod report;
mod scan;
mod targets;

pub use report::{filter_sort, print_report, total_size};
pub use scan::{dir_size, scan, Match};
pub use targets::{default_names, is_target, system_targets, SystemTarget};

use crate::utils::format_size;
use std::fs;
use std::path::{Path, PathBuf};

pub struct CleanOpts {
    pub path: Option<PathBuf>,
    pub execute: bool,
    pub min_size: Option<u64>,
    pub system: bool,
    pub names: Vec<String>,
}

/// True if `p` equals `/`, `home`, or the scan root itself (top-level guard).
pub fn is_refused_path(p: &Path, home: &Path, scan_root: Option<&Path>) -> bool {
    p == Path::new("/")
        || p == home
        || scan_root.map(|r| r == p).unwrap_or(false)
}

fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"))
}

/// Remove a match: whole dir, or (for system targets) its contents only.
/// Returns freed bytes on success.
fn remove_match(path: &Path, empty_contents: bool, size: u64) -> Result<u64, String> {
    let result = if empty_contents {
        let mut ok = true;
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let p = entry.path();
                let r = if p.is_dir() {
                    fs::remove_dir_all(&p)
                } else {
                    fs::remove_file(&p)
                };
                if r.is_err() {
                    ok = false;
                }
            }
        }
        if ok { Ok(()) } else { Err("some entries could not be removed".to_string()) }
    } else {
        fs::remove_dir_all(path).map_err(|e| e.to_string())
    };
    result.map(|_| size)
}

pub fn run(opts: CleanOpts) {
    let home = home_dir();
    let mut all: Vec<(PathBuf, u64, String, bool)> = Vec::new(); // path,size,kind,empty_contents

    // Project artifacts under PATH
    let scan_root: Option<PathBuf> = opts.path.clone();
    if let Some(ref root) = scan_root {
        println!("🧹 Scanning {} ...", root.display());
        let mut matches = scan(root, &opts.names);
        matches = filter_sort(matches, opts.min_size);
        print_report(&matches);
        for m in matches {
            all.push((m.path, m.size, m.kind, false));
        }
    }

    // System targets
    if opts.system {
        println!("🧹 Scanning system targets ...");
        for st in system_targets(&home, std::env::consts::OS) {
            let size = dir_size(&st.path);
            println!("  {:>9}   {:<14} {}", format_size(size), "system", st.path.display());
            all.push((st.path, size, "system".to_string(), st.empty_contents));
        }
    }

    let total: u64 = all.iter().map(|(_, s, _, _)| s).sum();
    println!("📋 {} items, {} reclaimable", all.len(), format_size(total));

    if !opts.execute {
        println!("ℹ️ Dry-run. Pass --execute to delete.");
        return;
    }

    let mut freed = 0u64;
    let mut count = 0usize;
    for (path, size, _kind, empty_contents) in all {
        if is_refused_path(&path, &home, scan_root.as_deref()) {
            println!("⚠️ Refusing to delete protected path: {}", path.display());
            continue;
        }
        if path.symlink_metadata().map(|m| m.file_type().is_symlink()).unwrap_or(false) {
            println!("⚠️ Skipping symlink: {}", path.display());
            continue;
        }
        match remove_match(&path, empty_contents, size) {
            Ok(f) => {
                freed += f;
                count += 1;
            }
            Err(e) => println!("⚠️ Failed to remove {}: {}", path.display(), e),
        }
    }
    println!("✅ Pruned {} items, freed {}", count, format_size(freed));
}
