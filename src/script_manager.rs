use std::{
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};
use walkdir::{DirEntry, WalkDir};

fn is_ignored(entry: &DirEntry, ignored: &Vec<&str>) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| ignored.contains(&s))
        .unwrap_or(false)
}

pub struct Executable {
    pub short_name: String,
    // pub long_name: String,
    pub path: PathBuf,
}

pub fn find_executables() -> Vec<Executable> {
    // TODO: Load this from .gitignore/other ignore files
    let ignored = vec!["target", ".github", ".vscode", ".git"];
    let walker = WalkDir::new(".").into_iter();
    let mut executables: Vec<Executable> = Vec::new();
    for result in walker.filter_entry(|e| (!is_ignored(e, &ignored))) {
        let entry = match result {
            Ok(entry) => entry,
            Err(_) => panic!("Couldn't read dir!"),
        };
        // TODO: Why can I not use this in the filter_entry expression?
        if !entry.file_type().is_dir() {
            let permissions = match entry.metadata() {
                Ok(metadata) => metadata.permissions(),
                Err(_) => panic!("Couldn't get file metadata!"),
            };
            let is_executable = permissions.mode() & 0o111 != 0;
            if is_executable {
                executables.push(Executable {
                    short_name: entry.file_name().to_string_lossy().to_string(),
                    path: entry.into_path(),
                })
            }
        }
    }
    executables
}
