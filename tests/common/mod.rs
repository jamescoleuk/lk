use std::process::{Command, Output};

/// Runs the compiled lk binary in the tests directory, with an isolated
/// config dir so the user's own `~/.config/lk` doesn't affect the results.
pub fn run_lk(args: &[&str]) -> Output {
    let config_dir = tempfile::tempdir().expect("failed to create temp config dir");
    Command::new(env!("CARGO_BIN_EXE_lk"))
        .args(args)
        .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/tests"))
        .env("LK_CONFIG_DIR", config_dir.path())
        .output()
        .expect("failed to execute lk")
}

pub fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}
