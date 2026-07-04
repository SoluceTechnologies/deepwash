# deepwash `docker` — flag-scoped cleaning + structured logging

Date: 2026-07-04
Status: Approved

## Problem

The `docker` subcommand always removes containers **and** images, always runs
`docker system prune -a -f` (which deletes unused images regardless of intent),
and on macOS always restarts Docker Desktop and clears buildx cache. There is no
way to remove containers while keeping images. Logging prints a misleading
`⚠️ error occurred` when a target set is simply empty.

## Goals

- Default `deepwash docker` removes containers only, **keeps images**.
- Opt into heavier cleaning via flags.
- Fix the bug where `system prune -a` deleted images in every run.
- Clearer, count-based logging with no false errors and no new dependencies.
- Split the monolithic `tasks/docker.rs` into focused submodules.

## CLI

Replace the single `volumes` flag on `Commands::Docker` with three flags
(`src/cli.rs`):

```rust
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
}
```

`src/main.rs` dispatches `docker::run(images, volumes, full)`.

## Flag model (independent)

- Containers are **always** removed.
- `-i` / `--images` adds image removal.
- `-v` / `--volumes` adds volume removal.
- Flags are independent — any combination is valid. `docker -v` removes
  containers + volumes and keeps images.
- `-f` / `--full` implies `images = true` and `volumes = true`, and additionally
  runs the heavy steps. It overrides the individual flags.

### Behavior matrix

| Command             | Containers | Images | Volumes | Heavy steps |
|---------------------|:----------:|:------:|:-------:|:-----------:|
| `docker`            | ✅         | —      | —       | —           |
| `docker -i`         | ✅         | ✅     | —       | —           |
| `docker -v`         | ✅         | —      | ✅      | —           |
| `docker -i -v`      | ✅         | ✅     | ✅      | —           |
| `docker --full`     | ✅         | ✅     | ✅      | ✅          |

Heavy steps = `docker system prune -a -f`, plus on macOS only: quit Docker
Desktop, reopen, wait for ready, then buildx history cleanup. On non-macOS the
buildx/restart block is skipped (message printed). `system prune -a` is kept in
`--full` even though images were already removed — it mops up build cache,
networks, and dangling data.

## Module layout

Convert `src/tasks/docker.rs` (file) into `src/tasks/docker/` (dir). Each
submodule exposes one function, does one job, and can be reasoned about alone.

```
src/tasks/docker/
├── mod.rs         // run(images, volumes, full): orchestrator + final summary
├── containers.rs  // clean() -> removes containers (always)
├── images.rs      // clean() -> removes images
├── volumes.rs     // clean() -> removes volumes
└── full.rs        // clean() -> system prune -a + macOS restart + buildx cache
```

`mod.rs` flow:

1. `let (images, volumes) = if full { (true, true) } else { (images, volumes) };`
2. `containers::clean()` (always)
3. `if images { images::clean() }`
4. `if volumes { volumes::clean() }`
5. `if full { full::clean() }`
6. Print summary.

## Logging (structured, no new deps)

Add a shared helper in `src/utils.rs` used by the container/image/volume
submodules to make counts accurate and kill false errors. Pattern: **list first,
then remove.**

Proposed helper:

```rust
/// Removes docker resources listed by `list_cmd`, deleted via `remove_prefix`.
/// Returns the number of items removed. Empty list => skip, count 0, no error.
pub fn clean_resource(label: &str, list_cmd: &str, remove_prefix: &str) -> usize
```

Behavior per resource:
- Run `list_cmd` (e.g. `docker ps -aq`). Split stdout into non-empty lines = ids.
- If no ids: print `⏭️  No {label} to remove`, return 0.
- Else: run `{remove_prefix} <ids...>`, print `✅ Removed {N} {label}`, return N.
  On a real command error, print `⚠️ Failed to remove {label}: {stderr}`.

`mod.rs` collects the returned counts and prints a final summary line:
`📋 Summary: {c} containers, {i} images, {v} volumes removed`.

The heavy `full::clean()` steps keep their existing per-step ✅/⚠️ prints (prune,
quit, start, ready, buildx) since they are not count-based.

## Out of scope

- No external logging crate (`log`/`env_logger`).
- No new subcommands beyond `docker`.
- No change to `wait_for_docker_ready` / `run_cmd` signatures (only additions).

## Testing

- `cargo build` and `cargo clippy` clean.
- Manual smoke on macOS:
  - `docker` → containers gone, images remain, no prune, no restart.
  - `docker -i` → images also gone, still no prune/restart.
  - `docker -v` → volumes gone, images remain.
  - `docker --full` (or `-f`) → full nuke incl. prune + macOS restart + buildx.
  - Run against an empty docker → `⏭️ No ... to remove` messages, zero false errors.
