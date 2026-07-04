use crate::utils::clean_resource;

/// Removes all containers (running or stopped). Returns count removed.
pub fn clean() -> usize {
    clean_resource("containers", "docker ps -aq", &["docker", "rm", "-f"])
}
