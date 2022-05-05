use std::{
    env,
    io::{stderr, stdout, Write},
    process::{self, Command},
};

// Usage: your_docker.sh run <image> <command> <arg1> <arg2> ...
fn main() {
    let args: Vec<_> = env::args().collect();
    let command = &args[3];
    let command_args = &args[4..];
    let output = Command::new(command).args(command_args).output().unwrap();

    stdout().write_all(&output.stdout).unwrap();
    stderr().write_all(&output.stderr).unwrap();

    process::exit(output.status.code().unwrap_or(0));
}
