use crate::script::Function;
use crate::script::Script;
use crate::ui::print_complete_header;
use anyhow::Result;
use nanoid::nanoid;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use tempfile::TempDir;

pub struct BashFile {
    // This isn't read but if the TempDir goes out-of-scope it might get deleted by the operating system.
    #[allow(dead_code)]
    dir: TempDir,
    full_path: PathBuf,
    script: Script,
    function: Function,
    params: Vec<String>,
}

impl BashFile {
    pub fn new(script: Script, function: Function, params: Vec<String>) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let file_name = format!("./~lk_{}", nanoid!(10));
        let full_path = dir.path().join(&file_name);
        Self {
            dir,
            full_path,
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
            .open(&self.full_path)?;

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
        let script_path = self.script.working_dir_absolute();
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

        Command::new(&self.full_path)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap()
            .wait()?;
        Ok(())
    }
}
