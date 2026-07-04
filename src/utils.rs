use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

pub fn parse_ids(stdout: &str) -> Vec<&str> {
    stdout
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect()
}

pub fn resolve_scope(images: bool, volumes: bool, full: bool) -> (bool, bool) {
    if full {
        (true, true)
    } else {
        (images, volumes)
    }
}

pub fn clean_resource(label: &str, list_cmd: &str, remove_argv: &[&str]) -> usize {
    let listed = match run_cmd("sh", &["-c", list_cmd]) {
        Ok(out) => out,
        Err(e) => {
            println!("⚠️ Failed to list {}: {}", label, e.trim());
            return 0;
        }
    };

    let ids = parse_ids(&listed);
    if ids.is_empty() {
        println!("⏭️ No {} to remove", label);
        return 0;
    }

    let (cmd, prefix) = remove_argv.split_first().expect("remove_argv is non-empty");
    let mut args: Vec<&str> = prefix.to_vec();
    args.extend_from_slice(&ids);
    match run_cmd(cmd, &args) {
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
        if run_cmd("docker", &["info"]).is_ok() {
            return true;
        }
        sleep(Duration::from_secs(interval));
        waited += interval;
    }
    false
}
