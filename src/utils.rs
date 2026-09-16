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

pub fn parse_size(s: &str) -> Result<u64, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("empty size".to_string());
    }
    let last = s.chars().last().unwrap();
    let (num_part, mult) = match last.to_ascii_uppercase() {
        'K' => (&s[..s.len() - 1], 1_000_f64),
        'M' => (&s[..s.len() - 1], 1_000_000_f64),
        'G' => (&s[..s.len() - 1], 1_000_000_000_f64),
        'T' => (&s[..s.len() - 1], 1_000_000_000_000_f64),
        c if c.is_ascii_digit() => (s, 1_f64),
        _ => return Err(format!("invalid size suffix in '{}'", s)),
    };
    let value: f64 = num_part
        .parse()
        .map_err(|_| format!("invalid size number in '{}'", s))?;
    if value < 0.0 {
        return Err(format!("negative size '{}'", s));
    }
    Ok((value * mult) as u64)
}

pub fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1000.0 && unit < UNITS.len() - 1 {
        value /= 1000.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{} {}", bytes, UNITS[unit])
    } else if value >= 100.0 {
        format!("{:.0} {}", value, UNITS[unit])
    } else {
        let s = format!("{:.1}", value);
        let s = s.strip_suffix(".0").unwrap_or(&s).to_string();
        format!("{} {}", s, UNITS[unit])
    }
}
