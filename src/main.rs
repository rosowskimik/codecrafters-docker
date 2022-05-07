use std::{
    env,
    ffi::CString,
    fs,
    io::{stderr, stdout, Write},
    path::Path,
    process::{self, Command},
};

use tempfile::TempDir;

fn setup_env<T: AsRef<Path>>(command: T) -> String {
    let command_path = command.as_ref();

    // create tmp directory
    let tmp_dir = TempDir::new().unwrap();

    // Get full executable path
    let full_path = env::var_os("PATH")
        .and_then(|paths| {
            env::split_paths(&paths).find_map(|dir| {
                let full_path = dir.join(command_path);
                if full_path.is_file() {
                    Some(full_path)
                } else {
                    None
                }
            })
        })
        .unwrap();

    // Set copy target
    let target_path = tmp_dir.path().join(command_path.file_name().unwrap());
    // Copy it into the tmp directory
    fs::copy(full_path, &target_path).unwrap();

    // Chroot into tmp dir && cd /
    let c_path = tmp_dir.path().to_str().unwrap();
    let c_path = CString::new(c_path).unwrap();
    #[cfg(target_family = "unix")]
    unsafe {
        libc::chroot(c_path.as_ptr());
    }
    env::set_current_dir("/").unwrap();

    fs::create_dir("/dev").unwrap();
    let f = std::fs::File::create("/dev/null").unwrap();
    drop(f);

    // String::from(target_path.file_name().unwrap().to_str().unwrap())
    let mut out = String::from("/");
    out.push_str(target_path.file_name().unwrap().to_str().unwrap());
    out
}

// Usage: your_docker.sh run <image> <command> <arg1> <arg2> ...
fn main() {
    let args: Vec<_> = env::args().collect();
    let command = &args[3];
    let command_args = &args[4..];

    let command = setup_env(command);

    let output = Command::new(command).args(command_args).output().unwrap();

    stdout().write_all(&output.stdout).unwrap();
    stderr().write_all(&output.stderr).unwrap();

    process::exit(output.status.code().unwrap_or(0));
}
