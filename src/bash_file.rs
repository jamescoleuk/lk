use colored::Colorize;

use crate::script::Function;
use anyhow::Result;

use std::process::Command;
use std::process::Stdio;

use crate::script::Script;
use crate::ui::print_complete_header;
use nanoid::nanoid;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;

pub struct BashFile {
    location: String,
    script: Script,
    function: Function,
    params: Vec<String>,
}

impl BashFile {
    pub fn new(script: Script, function: Function, params: Vec<String>) -> Self {
        Self {
            location: format!("./~lk_{}", nanoid!(10)),
            script,
            function,
            params,
        }
    }

    /// lk uses a temporary file in order to execute a function in a script. This temporary file
    /// sources the script we're going to execute and then it can run the function because it'll
    /// have been loaded into the shell. `std::process::Command` has no way to do this. An alternative
    /// would be adding `"$@"` to the end of the scripts but I'd rather avoid this stipulation.
    pub fn write(&self) -> Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .mode(0o700)
            .open(&self.location)?;

        // Write the file header
        let bash_file = r#"#!/usr/bin/env bash
# 
# Temporary lk file used to execute functions in scripts.
# If you see it here you can delete it and/or gitignore it.
"#;
        writeln!(file, "{}", bash_file)?;

        // CD to the scripts dir. This is an assumption we're making here,
        // but we can't avoid making an assumption, and this is safer than
        // assuming that the script can be run from any directory,
        // although that should be possible in a well written-script.
        let script_file_name = self.script.file_name();
        let script_path = self.script.path();
        writeln!(file, "cd {}", script_path)?;

        // Source the script so we can access its functions
        writeln!(file, "source {}", script_file_name)?;

        // Call the function the user asked for
        writeln!(file, "{} {}", self.function.name, self.params.join(" "))?;

        Ok(())
    }

    /// This executes the lk file, and then removes it.
    pub fn execute(&self) -> Result<()> {
        print_complete_header(&self.script, &self.function, &self.params);

        let mut cmd = Command::new(&self.location)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();
        let exit_status = cmd.wait()?;
        match exit_status.code() {
            Some(code) => {
                match std::fs::remove_file(&self.location) {
                    Ok(_) => {
                        // Great, we've tidied up.
                    }
                    Err(e) => {
                        if e.to_string().contains("No such file or directory") {
                            // We don't care about this
                        } else {
                            eprintln!(
                            "Yikes! I couldn't remove my temporary file, '{}'! The error was {}",
                            self.location,
                            e.to_string().red()
                        )
                        }
                    }
                }
                std::process::exit(code)
            }
            None => eprintln!("Your function exited without a status code!"),
        }
        Ok(())
    }
}
