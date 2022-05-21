mod network;

use std::{env, process};

use anyhow::Result;
use tempfile::TempDir;
use unshare::{Command, Namespace, Stdio};

use network::fetch_image;

// Usage: your_docker.sh run <image> <command> <arg1> <arg2> ...
// #[tokio::main]
fn main() -> Result<()> {
    // println!("Start");
    let args: Vec<_> = env::args().collect();
    let container = &args[2];
    let command = &args[3];
    let command_args = &args[4..];

    let temp_dir = TempDir::new()?;

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(fetch_image(container, temp_dir.path()))?;
    // fetch_image(container, temp_dir.path()).await?;

    let status = Command::new(command)
        .args(command_args)
        .chroot_dir(temp_dir.path())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .unshare([&Namespace::Pid])
        .spawn()
        .unwrap()
        .wait()?;

    process::exit(status.code().unwrap_or(0));
}
