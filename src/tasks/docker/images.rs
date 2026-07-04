use crate::utils::clean_resource;

/// Removes all images. Returns count removed.
pub fn clean() -> usize {
    clean_resource("images", "docker images -aq", "docker rmi -f")
}
