use crate::utils::clean_resource;

/// Removes all volumes. Returns count removed.
pub fn clean() -> usize {
    clean_resource("volumes", "docker volume ls -q", &["docker", "volume", "rm"])
}
