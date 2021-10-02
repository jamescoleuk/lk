use anyhow::{anyhow, Result};
use content_inspector::{inspect, ContentType};
use std::{os::unix::fs::PermissionsExt, path::PathBuf};
use walkdir::{DirEntry, WalkDir};

pub struct Executable {
    pub short_name: String,
    pub path: PathBuf,
}

pub struct Executables {
    // root: String,
    executables: Vec<Executable>,
}

impl Executables {
    pub fn new(root: &str) -> Self {
        // TODO: Load this from .gitignore/other ignore files
        let ignored = vec!["target", ".github", ".vscode", ".git"];
        let walker = WalkDir::new(root).into_iter();
        let mut executables: Vec<Executable> = Vec::new();
        for result in walker.filter_entry(|e| (!is_ignored(e, &ignored))) {
            let entry = match result {
                Ok(entry) => entry,
                Err(_) => panic!("Couldn't read dir!"),
            };
            if !entry.file_type().is_dir() && is_executable(&entry) {
                executables.push(Executable {
                    short_name: entry.file_name().to_string_lossy().to_string(),
                    path: entry.into_path(),
                })
            }
        }
        Self {
            // root: root.to_string(),
            executables,
        }
    }

    pub fn get(&self, name: &str) -> Option<&Executable> {
        self.executables
            .iter()
            .find(|&executable| executable.short_name == name)
    }

    /// Pretty-prints the executables we found on the path, so the
    /// user can select one to run.
    pub fn pretty_print(&self) {
        println!("Runsh has found the following executables. Execute runsh <executable_name> to see what functions it offers.");
        self.executables.iter().for_each(|executable| {
            println!(
                "{} -- {}",
                executable.short_name,
                executable.path.as_os_str().to_string_lossy().to_string()
            );
        })
    }
}

fn is_ignored(entry: &DirEntry, ignored: &[&str]) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| ignored.contains(&s))
        .unwrap_or(false)
}

fn is_executable(entry: &DirEntry) -> bool {
    let permissions = match entry.metadata() {
        Ok(metadata) => metadata.permissions(),
        Err(_) => panic!("Couldn't get file metadata!"),
    };
    let is_executable = permissions.mode() & 0o111 != 0;
    is_executable
}
