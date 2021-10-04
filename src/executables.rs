use colored::Colorize;
use content_inspector::{inspect, ContentType};
use pad::{Alignment, PadStr};
use std::{io::Read, os::unix::fs::PermissionsExt, path::PathBuf};
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
            if !entry.file_type().is_dir() && is_executable(&entry) && !is_binary(&entry) {
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
        println!("{}", "lk: ./".on_blue());
        // Get the longest executable name
        const INDENT: usize = 2;
        let padding = self
            .executables
            .iter()
            .max_by(|x, y| x.short_name.len().cmp(&y.short_name.len()))
            .unwrap() // Will always be Some because the name String must exist.
            .short_name
            .len()
            + INDENT;
        self.executables.iter().for_each(|executable| {
            // We'll pad right so everything aligns nicely.
            let to_print = executable
                .short_name
                .pad_to_width_with_alignment(padding, Alignment::Right)
                .green();
            println!(
                "{} - {}",
                to_print,
                executable.path.as_os_str().to_string_lossy().to_string()
            );
        });
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
    permissions.mode() & 0o111 != 0
}

fn is_binary(entry: &DirEntry) -> bool {
    // We're testing for executable permissions before we check for binary or text
    // because we don't want to attempt to read any files we don't have to.
    let file = std::fs::File::open(entry.path()).unwrap();
    // We're only going to read a smidgen of the file because that's all we need
    // for using content_inspector.
    let mut buffer = [0; 10];
    std::io::BufReader::new(file)
        .read_exact(&mut buffer)
        .unwrap();
    inspect(&buffer) == ContentType::BINARY
}
