use anyhow::{Context, Result};
use std::{
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
};

struct Shell {
    locations: &'static [&'static str],
    history_file: &'static str,
}

const SHELLS: &[Shell] = &[
    Shell {
        locations: &["/bin/bash", "/usr/local/bin/bash", "/opt/homebrew/bin/bash"],
        history_file: ".bash_history",
    },
    Shell {
        locations: &["/bin/zsh", "/usr/local/bin/zsh", "/opt/homebrew/bin/zsh"],
        history_file: ".zsh_history",
    },
];

/// The user's shell, so we can write executed commands to its history file.
pub struct UserShell {
    history_file: PathBuf,
}

impl UserShell {
    pub fn new() -> Option<Self> {
        // This environment variable is the way we detect the current shell.
        let Ok(shell_path) = std::env::var("SHELL") else {
            log::error!(
                "Unable to get the current value of SHELL, so we won't be updating the history"
            );
            return None;
        };

        let Some(home_dir) = dirs::home_dir() else {
            log::error!("Unable to find your home directory, so we won't be updating the history");
            return None;
        };

        // Get the shell from the shells we know about
        let Some(shell) = SHELLS
            .iter()
            .find(|shell| shell.locations.contains(&shell_path.as_str()))
        else {
            log::error!(
                "Couldn't find a configuration for the shell in this location: {shell_path}"
            );
            return None;
        };

        let history_file = home_dir.join(shell.history_file);
        if Path::new(&history_file).exists() {
            Some(Self { history_file })
        } else {
            log::error!("I expected your history file to be at {history_file:?} but it wasn't.");
            None
        }
    }

    pub fn add_command(&self, command: &str) -> Result<()> {
        log::info!("Adding command to history: {command}");
        log::info!("History file: {:?}", &self.history_file);
        let mut file = OpenOptions::new()
            .append(true)
            .open(&self.history_file)
            .with_context(|| format!("Unable to open history file {:?}", self.history_file))?;
        writeln!(file, "{command}")?;
        Ok(())
    }
}
