use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

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

pub fn run_cmd(cmd: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run command: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}


pub fn wait_for_docker_ready(timeout_secs: u64) -> bool {
    let mut waited = 0;
    let interval = 2;
    while waited < timeout_secs {
        if let Ok(_) = run_cmd("docker", &["info"]) {
            return true;
        }
        sleep(Duration::from_secs(interval));
        waited += interval;
    }
    false
}

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