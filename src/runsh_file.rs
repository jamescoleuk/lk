// mod executables;
// mod script;

use crate::script::Function;
use anyhow::Result;

use std::process::Command;
use std::process::Stdio;

use crate::script::Script;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;

/// Runsh uses a temporary file in order to execute a function in a script. This temporary file
/// sources the script we're going to execute and then it can run the function because it'll
/// have been loaded into the shell. `std::process::Command` has no way to do this. An alternative
/// would be adding `"$@"` to the end of the scripts but I'd rather avoid this stipulation.
// fn write_runsh_file(validated_request: ValidatedRequest) -> Result<()> {
pub fn write_runsh_file(script: &Script, function: &Function) -> Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .mode(0o700)
        .open("~runsh")?;
    let runsh_file = r#"#!/usr/bin/env bash
# 
# Temporary runsh file used to execute functions in scripts.
# If you see it here you can delete it and/or gitignore it.

"#;
    writeln!(
        file,
        "{} source {} && {}",
        runsh_file,
        script.path(),
        function.name
    )?;
    Ok(())
}

/// This executes the runsh file, and then removes it.
pub fn execute_runsh_file() -> Result<()> {
    let mut cmd = Command::new("./~runsh")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();

    let exit_status = cmd.wait()?;
    match exit_status.code() {
        Some(code) => {
            match std::fs::remove_file("./~runsh") {
                Ok(_) => {
                    // Great, we've tidied up.
                }
                Err(e) => {
                    eprintln!(
                        "Yikes! I couldn't remove my temporary file, './~runsh'! The error was {}",
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
