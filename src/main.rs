use std::{env, fs, path::Path, process};

use tempfile::TempDir;
use unshare::{Command, Namespace, Stdio};

fn setup_tmp_dir<T: AsRef<Path>>(command: T) -> (TempDir, String) {
    let command_path = command.as_ref().canonicalize().unwrap();

    // create tmp directory
    let temp_dir = TempDir::new().unwrap();

    // Copy command into the tmp directory
    let target_path = temp_dir.path().join(command_path.file_name().unwrap());
    fs::copy(command_path, &target_path).unwrap();

    // Prepare new command
    let mut command = String::from("/");
    command.push_str(target_path.file_name().unwrap().to_str().unwrap());

    (temp_dir, command)
}

// Usage: your_docker.sh run <image> <command> <arg1> <arg2> ...
fn main() {
    let args: Vec<_> = env::args().collect();
    let command = &args[3];
    let command_args = &args[4..];

    let (directory, command) = setup_tmp_dir(command);

    let status = Command::new(command)
        .args(command_args)
        .chroot_dir(directory.path())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .unshare([&Namespace::Pid])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    // stdout().write_all(&output.stdout).unwrap();
    // stderr().write_all(&output.stderr).unwrap();

    process::exit(status.code().unwrap_or(0));
}
