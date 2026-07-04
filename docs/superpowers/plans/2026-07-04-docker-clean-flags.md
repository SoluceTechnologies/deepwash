# Docker Flag-Scoped Cleaning + Structured Logging Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `deepwash docker` remove only containers by default (keep images), with `-i` (images), `-v` (volumes), and `-f/--full` (everything + heavy steps) flags, plus count-based logging with no false errors.

**Architecture:** Split `src/tasks/docker.rs` into a `docker/` submodule dir (one job per file). Add pure, unit-tested helpers in `src/utils.rs` (`parse_ids`, `resolve_scope`) so the shell-driven code has testable seams. Each resource submodule uses a shared `clean_resource` helper that lists ids first, then removes — giving real counts and skipping empty sets cleanly.

**Tech Stack:** Rust 2024 edition, clap 4 (derive), std only. No new dependencies.

**Spec:** `docs/superpowers/specs/2026-07-04-docker-clean-flags-design.md`

---

## File Structure

- `src/cli.rs` — replace single `volumes` flag with `images` / `volumes` / `full` on `Commands::Docker`.
- `src/main.rs` — dispatch `docker::run(images, volumes, full)`.
- `src/utils.rs` — add pure helpers `parse_ids`, `resolve_scope`, and shell helper `clean_resource`; add `#[cfg(test)]` unit tests.
- `src/tasks/docker.rs` — **deleted**, replaced by dir below.
- `src/tasks/docker/mod.rs` — `run(images, volumes, full)` orchestrator + final summary.
- `src/tasks/docker/containers.rs` — `clean() -> usize`.
- `src/tasks/docker/images.rs` — `clean() -> usize`.
- `src/tasks/docker/volumes.rs` — `clean() -> usize`.
- `src/tasks/docker/full.rs` — `clean()` heavy steps (prune + macOS restart + buildx).

---

## Task 1: Pure helpers in utils.rs (parse_ids + resolve_scope)

**Files:**
- Modify: `src/utils.rs`

- [ ] **Step 1: Write the failing tests**

Append to `src/utils.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ids_splits_nonempty_lines() {
        assert_eq!(parse_ids("a\nb\nc\n"), vec!["a", "b", "c"]);
    }

    #[test]
    fn parse_ids_ignores_blank_and_whitespace_lines() {
        assert_eq!(parse_ids("a\n\n  \nb\n"), vec!["a", "b"]);
    }

    #[test]
    fn parse_ids_empty_input_is_empty() {
        assert!(parse_ids("").is_empty());
        assert!(parse_ids("\n  \n").is_empty());
    }

    #[test]
    fn resolve_scope_full_forces_images_and_volumes() {
        assert_eq!(resolve_scope(false, false, true), (true, true));
    }

    #[test]
    fn resolve_scope_without_full_passes_flags_through() {
        assert_eq!(resolve_scope(false, false, false), (false, false));
        assert_eq!(resolve_scope(true, false, false), (true, false));
        assert_eq!(resolve_scope(false, true, false), (false, true));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test`
Expected: FAIL — `cannot find function parse_ids` / `resolve_scope`.

- [ ] **Step 3: Implement the helpers**

Add near the top of `src/utils.rs` (after existing `use` lines):

```rust
/// Splits command stdout into non-empty, trimmed resource ids.
pub fn parse_ids(stdout: &str) -> Vec<&str> {
    stdout
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect()
}

/// Resolves effective (images, volumes) scope. `full` forces both on.
pub fn resolve_scope(images: bool, volumes: bool, full: bool) -> (bool, bool) {
    if full {
        (true, true)
    } else {
        (images, volumes)
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test`
Expected: PASS (5 tests).

- [ ] **Step 5: Commit**

```bash
git add src/utils.rs
git commit -m "feat: add parse_ids and resolve_scope helpers"
```

---

## Task 2: clean_resource shell helper in utils.rs

**Files:**
- Modify: `src/utils.rs`

- [ ] **Step 1: Implement the helper**

`clean_resource` shells out to docker so it is not unit-tested; it is exercised by manual smoke in Task 7. Add to `src/utils.rs`:

```rust
/// Lists docker resources via `list_cmd`, then removes them with `remove_prefix`.
/// Returns count removed. Empty list => skip cleanly (count 0, no error print).
///
/// `label` is a human plural, e.g. "containers".
/// `list_cmd` is a full `sh -c` string, e.g. "docker ps -aq".
/// `remove_prefix` is the removal command minus ids, e.g. "docker rm -f".
pub fn clean_resource(label: &str, list_cmd: &str, remove_prefix: &str) -> usize {
    let listed = match run_cmd("sh", &["-c", list_cmd]) {
        Ok(out) => out,
        Err(e) => {
            println!("⚠️ Failed to list {}: {}", label, e.trim());
            return 0;
        }
    };

    let ids = parse_ids(&listed);
    if ids.is_empty() {
        println!("⏭️  No {} to remove", label);
        return 0;
    }

    let remove_cmd = format!("{} {}", remove_prefix, ids.join(" "));
    match run_cmd("sh", &["-c", &remove_cmd]) {
        Ok(_) => {
            println!("✅ Removed {} {}", ids.len(), label);
            ids.len()
        }
        Err(e) => {
            println!("⚠️ Failed to remove {}: {}", label, e.trim());
            0
        }
    }
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo build`
Expected: builds (warnings about unused `clean_resource` are OK until Task 4 wires it).

- [ ] **Step 3: Commit**

```bash
git add src/utils.rs
git commit -m "feat: add clean_resource list-then-remove helper"
```

---

## Task 3: Update CLI flags

**Files:**
- Modify: `src/cli.rs`

- [ ] **Step 1: Replace the Docker variant**

In `src/cli.rs`, replace the `Docker { ... }` variant inside `enum Commands` with:

```rust
    /// Remove Docker containers (keeps images by default)
    Docker {
        /// Also remove images
        #[arg(short = 'i', long = "images")]
        images: bool,
        /// Also remove volumes
        #[arg(short = 'v', long = "volumes")]
        volumes: bool,
        /// Full clean: images + volumes + system prune -a + buildx cache (+ macOS Docker restart)
        #[arg(short = 'f', long = "full")]
        full: bool,
    },
```

- [ ] **Step 2: Verify it compiles (main.rs will still break — expected)**

Run: `cargo build`
Expected: FAIL — `main.rs` still matches old `Docker { volumes }` pattern. Fixed in Task 4.

- [ ] **Step 3: Commit**

```bash
git add src/cli.rs
git commit -m "feat: add -i/-v/-f flags to docker subcommand"
```

---

## Task 4: Docker submodule dir — mod.rs orchestrator + resource submodules

**Files:**
- Delete: `src/tasks/docker.rs`
- Create: `src/tasks/docker/mod.rs`
- Create: `src/tasks/docker/containers.rs`
- Create: `src/tasks/docker/images.rs`
- Create: `src/tasks/docker/volumes.rs`
- Create: `src/tasks/docker/full.rs`

- [ ] **Step 1: Delete the old file**

```bash
git rm src/tasks/docker.rs
```

- [ ] **Step 2: Create `src/tasks/docker/containers.rs`**

```rust
use crate::utils::clean_resource;

/// Removes all containers (running or stopped). Returns count removed.
pub fn clean() -> usize {
    clean_resource("containers", "docker ps -aq", "docker rm -f")
}
```

- [ ] **Step 3: Create `src/tasks/docker/images.rs`**

```rust
use crate::utils::clean_resource;

/// Removes all images. Returns count removed.
pub fn clean() -> usize {
    clean_resource("images", "docker images -aq", "docker rmi -f")
}
```

- [ ] **Step 4: Create `src/tasks/docker/volumes.rs`**

```rust
use crate::utils::clean_resource;

/// Removes all volumes. Returns count removed.
pub fn clean() -> usize {
    clean_resource("volumes", "docker volume ls -q", "docker volume rm")
}
```

- [ ] **Step 5: Create `src/tasks/docker/full.rs`**

```rust
use crate::utils::{run_cmd, wait_for_docker_ready};
use std::env::consts::OS;
use std::thread::sleep;
use std::time::Duration;

/// Heavy cleanup: system prune -a, plus macOS Docker restart + buildx cache.
pub fn clean() {
    match run_cmd("sh", &["-c", "docker system prune -a -f"]) {
        Ok(out) => println!("✅ System prune done:\n{}", out.trim()),
        Err(_) => println!("⚠️ System prune failed or nothing to prune"),
    }

    if OS != "macos" {
        println!("ℹ️ Skipping buildx prune: not running on macOS");
        return;
    }

    println!("🔄 Restarting Docker before buildx prune...");
    match run_cmd("pkill", &["-f", "Docker Desktop"]) {
        Ok(_) => println!("✅ Docker quit successfully."),
        Err(_) => println!("⚠️ Failed to quit Docker. You might need to restart manually."),
    }
    sleep(Duration::from_secs(3));
    match run_cmd("open", &["-a", "Docker"]) {
        Ok(_) => println!("✅ Docker started successfully."),
        Err(_) => println!("⚠️ Failed to start Docker. You might need to start manually."),
    }

    println!("⏳ Waiting for Docker to become ready (up to 60 seconds)...");
    if !wait_for_docker_ready(60) {
        println!("⚠️ Docker did not become ready in time. Skipping buildx prune.");
        return;
    }
    println!("✅ Docker is ready.");
    let buildx = run_cmd(
        "sh",
        &[
            "-c",
            "docker buildx history rm $(docker buildx history ls | tail -n +2 | awk '{print $1}')",
        ],
    );
    match buildx {
        Ok(out) => println!("✅ Buildx cache cleaned:\n{}", out.trim()),
        Err(_) => println!("⚠️ No Buildx to prune"),
    }
}
```

- [ ] **Step 6: Create `src/tasks/docker/mod.rs`**

```rust
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
    let (images, volumes) = resolve_scope(images, volumes, full);

    let c = containers::clean();
    let i = if images { images::clean() } else { 0 };
    let v = if volumes { volumes::clean() } else { 0 };

    if full {
        full::clean();
    }

    println!(
        "📋 Summary: {} containers, {} images, {} volumes removed",
        c, i, v
    );
}
```

- [ ] **Step 7: Verify (main.rs still broken — expected)**

Run: `cargo build`
Expected: FAIL only on `main.rs` old pattern match. Fixed in Task 5.

- [ ] **Step 8: Commit**

```bash
git add src/tasks/
git commit -m "refactor: split docker task into scoped submodules"
```

---

## Task 5: Wire dispatch in main.rs

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Update the match arm**

In `src/main.rs`, replace the `Docker` match arm:

```rust
        Some(Commands::Docker { images, volumes, full }) => docker::run(images, volumes, full),
```

- [ ] **Step 2: Verify full build passes**

Run: `cargo build`
Expected: PASS, no errors.

- [ ] **Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat: dispatch docker run with images/volumes/full flags"
```

---

## Task 6: Lint + test gate

**Files:** none (verification only)

- [ ] **Step 1: Run clippy**

Run: `cargo clippy -- -D warnings`
Expected: no warnings. Fix any (e.g. unused imports) inline and re-run.

- [ ] **Step 2: Run tests**

Run: `cargo test`
Expected: PASS (5 unit tests from Task 1).

- [ ] **Step 3: Verify help text**

Run: `cargo run -- docker --help`
Expected: shows `-i, --images`, `-v, --volumes`, `-f, --full` with descriptions.

- [ ] **Step 4: Commit (only if fixes were needed)**

```bash
git add -A
git commit -m "chore: clippy clean for docker refactor"
```

---

## Task 7: Manual smoke test (requires Docker; macOS)

**Files:** none (manual verification)

> Requires a real Docker daemon. Run against throwaway containers/images. These
> steps are destructive to local Docker state — run only where that is safe.

- [ ] **Step 1: Default keeps images**

Run: `cargo run -- docker`
Expected: `⏭️  No containers to remove` (or `✅ Removed N containers`), NO images/volumes/prune output, NO macOS restart. `docker images` afterwards still lists images.

- [ ] **Step 2: Images flag**

Run: `cargo run -- docker -i`
Expected: containers + images lines, still no prune / no restart.

- [ ] **Step 3: Volumes flag independence**

Run: `cargo run -- docker -v`
Expected: containers + volumes lines, images preserved.

- [ ] **Step 4: Full**

Run: `cargo run -- docker --full`
Expected: containers + images + volumes + `✅ System prune done` + macOS restart sequence + buildx cache line. `-f` behaves identically to `--full`.

- [ ] **Step 5: Empty-state no false errors**

With nothing to clean, run: `cargo run -- docker`
Expected: `⏭️  No containers to remove`, no `⚠️` error lines, summary `0 containers, 0 images, 0 volumes removed`.

---

## Self-Review Notes

- **Spec coverage:** CLI flags (Task 3), independent flag model + `--full` implies both (Task 1 `resolve_scope`, Task 4 `mod.rs`), heavy steps in `--full` only (Task 4 `full.rs`), module split (Task 4), structured logging / no false errors (Task 2 `clean_resource`, Task 4 summary), `system prune -a` kept in full (Task 4 `full.rs`). All covered.
- **Types consistent:** `clean() -> usize` for resource modules, `clean()` unit for full, `resolve_scope(bool,bool,bool) -> (bool,bool)`, `clean_resource(&str,&str,&str) -> usize`, `parse_ids(&str) -> Vec<&str>` — used consistently across tasks.
- **No placeholders:** every code step shows full code.
