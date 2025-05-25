use crate::utils::{run_cmd, wait_for_docker_ready};
use std::env::consts::OS;
use std::thread::sleep;
use std::time::Duration;

pub fn run(volumes: bool) {
    println!("Docker cleaning...");

    let containers_cmd = run_cmd("sh", &["-c", "docker rm -f $(docker ps -aq)"]);
    match containers_cmd {
        Ok(out) => println!("✅ Containers removed:\n{}", out),
        Err(_) => println!("⚠️ No containers to remove or error occurred"),
    }

    let images_cmd = run_cmd("sh", &["-c", "docker rmi -f $(docker images -aq)"]);
    match images_cmd {
        Ok(out) => println!("✅ Images removed:\n{}", out),
        Err(_) => println!("⚠️ No images to remove or error occurred"),
    }

    if volumes {
        let volumes_cmd = run_cmd("sh", &["-c", "docker volume rm $(docker volume ls -q)"]);
        match volumes_cmd {
            Ok(out) => println!("✅ Volumes removed:\n{}", out),
            Err(_) => println!("⚠️ No volumes to remove or error occurred"),
        }
    }

    let prune_cmd = run_cmd("sh", &["-c", "docker system prune -a -f"]);
    match prune_cmd {
        Ok(out) => println!("✅ System prune done:\n{}", out),
        Err(_) => println!("⚠️ System prune failed or nothing to prune"),
    }

    if OS == "macos" {
        println!("🔄 Restarting Docker before buildx prune...");
        let quit = run_cmd("pkill", &["-f", "Docker Desktop"]);
        match quit {
            Ok(_) => println!("✅ Docker quit successfully."),
            Err(_) => println!("⚠️ Failed to quit Docker. You might need to restart manually."),
        }
        sleep(Duration::from_secs(3));
        let start = run_cmd("open", &["-a", "Docker"]);
        match start {
            Ok(_) => println!("✅ Docker started successfully."),
            Err(_) => println!("⚠️ Failed to start Docker. You might need to start manually."),
        }

        println!("⏳ Waiting for Docker to become ready (up to 60 seconds)...");
        if wait_for_docker_ready(60) {
            println!("✅ Docker is ready.");
            let buildx_prune = run_cmd(
                "sh",
                &[
                    "-c",
                    "docker buildx history rm $(docker buildx history ls | tail -n +2 | awk '{print $1}')  ",
                ],
            );
            match buildx_prune {
                Ok(out) => println!("✅ Buildx cache cleaned:\n{}", out),
                Err(_) => println!("⚠️ No Buildx to prune"),
            }
        } else {
            println!("⚠️ Docker did not become ready in time. Skipping buildx prune.");
        }
    } else {
        println!("ℹ️ Skipping buildx prune: not running on macOS");
    }
}
