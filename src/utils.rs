use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

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