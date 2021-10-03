use colored::Colorize;

use crate::script::Function;
use anyhow::Result;

use std::process::Command;
use std::process::Stdio;

use crate::script::Script;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;

static RN_FILE: &str = "./~rn";

/// rn uses a temporary file in order to execute a function in a script. This temporary file
/// sources the script we're going to execute and then it can run the function because it'll
/// have been loaded into the shell. `std::process::Command` has no way to do this. An alternative
/// would be adding `"$@"` to the end of the scripts but I'd rather avoid this stipulation.
pub fn write_rn_file(script: &Script, function: &Function) -> Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .mode(0o700)
        .open(RN_FILE)?;
    let rn_file = r#"#!/usr/bin/env bash
# 
# Temporary rn file used to execute functions in scripts.
# If you see it here you can delete it and/or gitignore it.

"#;
    writeln!(
        file,
        "{} source {} && {}",
        rn_file,
        script.path(),
        function.name
    )?;
    Ok(())
}

/// This executes the rn file, and then removes it.
pub fn execute_rn_file(script: &Script, function: &Function) -> Result<()> {
    println!(
        "{}{}{}{}",
        "rn: ".on_blue(),
        script.path.as_os_str().to_string_lossy().on_blue(),
        " -> ".on_blue(),
        function.name.on_blue()
    );
    let mut cmd = Command::new(RN_FILE)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();

    let exit_status = cmd.wait()?;
    match exit_status.code() {
        Some(code) => {
            match std::fs::remove_file(RN_FILE) {
                Ok(_) => {
                    // Great, we've tidied up.
                }
                Err(e) => {
                    eprintln!(
                        "Yikes! I couldn't remove my temporary file, '{}'! The error was {}",
                        RN_FILE,
                        e.to_string()
                    )
                }
            }
            std::process::exit(code)
        }
        None => eprintln!("Your function exited without a status code!"),
    }
    Ok(())
}
