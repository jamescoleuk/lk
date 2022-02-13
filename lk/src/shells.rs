use std::{fs::OpenOptions, io::Write, path::Path};

use anyhow::Result;

#[derive(Clone)]
pub struct Shell {
    locations: Vec<String>,
    pub history_file: String,
}

pub struct UserShell {
    // shell: Shell,
    // home_dir: String,
    history_file: String,
}

impl UserShell {
    pub fn new() -> Option<Self> {
        let shells: Vec<Shell> = vec![
            Shell {
                locations: vec!["/usr/local/bin/bash".to_string()],
                history_file: ".bash_history".to_string(),
            },
            Shell {
                locations: vec!["/bin/zsh".to_string(), "/usr/local/bin/zsh".to_string()],
                history_file: ".zsh_history".to_string(),
            },
            Shell {
                locations: vec!["/usr/local/bin/fish".to_string()],
                history_file: ".config/fish/fish_history".to_string(),
            },
        ];

        // This environment variable is the way we detect the current shell.
        if let Ok(shell_path) = std::env::var("SHELL") {
            if let Ok(home_path) = std::env::var("HOME") {
                // Get the shell from the current shells
                let shell = shells.iter().find(|shell| {
                    shell
                        .locations
                        .iter()
                        .any(|location| location == &shell_path)
                });

                if let Some(shell) = shell {
                    let history_file = format!("{}/{}", home_path, &shell.history_file);
                    if Path::new(&history_file).exists() {
                        Some(Self { history_file })
                    } else {
                        log::error!(
                            "I expected your history file to be at '{}' but it wasn't.",
                            history_file
                        );
                        None
                    }
                } else {
                    log::error!(
                        "Couldn't find a configuration for the shell in this location: {}",
                        shell_path
                    );
                    None
                }
            } else {
                log::error!(
                    "Unable to get the current value of HOME, so we won't be updating the history"
                );
                None
            }
        } else {
            log::error!(
                "Unable to get the current value of SHELL, so we won't be updating the history"
            );
            None
        }
    }

    pub fn add_command(&self, command: String) -> Result<()> {
        log::info!("Adding command to history: {}", command);
        // let history_file: String = self.history_file();
        log::info!("History file: {}", &self.history_file);
        // TODO Don't fail if the file doesn't exist
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&self.history_file)
            .unwrap();
        writeln!(file, "{}", command)?;
        Ok(())
    }
}
