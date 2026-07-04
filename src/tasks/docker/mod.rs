mod containers;
mod full;
mod images;
mod volumes;

use crate::utils::resolve_scope;

/// Cleans docker resources scoped by flags.
/// Containers are always removed; images/volumes are opt-in; `full` implies both
/// plus heavy steps (system prune -a, macOS Docker restart, buildx cache).
pub fn run(images: bool, volumes: bool, full: bool) {
    println!("🧽 Docker cleaning...");
    let (remove_images, remove_volumes) = resolve_scope(images, volumes, full);

    let c = containers::clean();
    let i = if remove_images { images::clean() } else { 0 };
    let v = if remove_volumes { volumes::clean() } else { 0 };

    if full {
        full::clean();
    }

    println!(
        "📋 Summary: {} containers, {} images, {} volumes removed",
        c, i, v
    );
}
