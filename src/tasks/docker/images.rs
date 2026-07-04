use crate::utils::clean_resource;

pub fn clean() -> usize {
    clean_resource("images", "docker images -aq", &["docker", "rmi", "-f"])
}
