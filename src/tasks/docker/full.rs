use crate::utils::{run_cmd, wait_for_docker_ready};
use std::env::consts::OS;
use std::thread::sleep;
use std::time::Duration;

pub fn clean() {
    match run_cmd("sh", &["-c", "docker system prune -a -f"]) {
        Ok(out) => println!("✅ System prune done:\n{}", out.trim()),
        Err(_) => println!("⚠️ System prune failed or nothing to prune"),
    }

    if OS != "macos" {
        println!("ℹ️ Skipping buildx prune: not running on macOS");
        return;
    }

    println!("🔄 Restarting Docker before buildx prune...");
    match run_cmd("pkill", &["-f", "Docker Desktop"]) {
        Ok(_) => println!("✅ Docker quit successfully."),
        Err(_) => println!("⚠️ Failed to quit Docker. You might need to restart manually."),
    }
    sleep(Duration::from_secs(3));
    match run_cmd("open", &["-a", "Docker"]) {
        Ok(_) => println!("✅ Docker started successfully."),
        Err(_) => println!("⚠️ Failed to start Docker. You might need to start manually."),
    }

    println!("⏳ Waiting for Docker to become ready (up to 60 seconds)...");
    if !wait_for_docker_ready(60) {
        println!("⚠️ Docker did not become ready in time. Skipping buildx prune.");
        return;
    }
    println!("✅ Docker is ready.");
    let buildx = run_cmd(
        "sh",
        &[
            "-c",
            "docker buildx history rm $(docker buildx history ls | tail -n +2 | awk '{print $1}')",
        ],
    );
    match buildx {
        Ok(out) => println!("✅ Buildx cache cleaned:\n{}", out.trim()),
        Err(_) => println!("⚠️ No Buildx to prune"),
    }
}
