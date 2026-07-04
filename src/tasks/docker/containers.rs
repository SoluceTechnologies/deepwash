use crate::utils::clean_resource;

pub fn clean() -> usize {
    clean_resource("containers", "docker ps -aq", &["docker", "rm", "-f"])
}
