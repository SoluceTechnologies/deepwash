use crate::utils::clean_resource;

pub fn clean() -> usize {
    clean_resource("volumes", "docker volume ls -q", &["docker", "volume", "rm"])
}
