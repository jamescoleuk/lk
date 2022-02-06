/// Finds executables in the current directory.
use crate::{ui::print_root_header};
use content_inspector::{inspect, ContentType};
use pad::{Alignment, PadStr};
use pastel_colours::{DARK_GREEN_FG, RESET_FG};
use std::{io::Read, os::unix::fs::PermissionsExt, path::PathBuf, fs::Permissions};
use walkdir::{DirEntry, WalkDir};

pub struct Executable {
    pub short_name: String,
    pub path: PathBuf,
    pub absolute_path: PathBuf,
}

pub struct Executables {
    // root: String,
    pub executables: Vec<Executable>,
}

impl Executables {
    pub fn new(root: &str) -> Self {
        // TODO: Load this from .gitignore/other ignore files
        let ignored = vec![
            "target",
            ".github",
            ".vscode",
            ".git",
            "node_modules",
            ".nvm",
            ".Trash",
            ".npm",
            ".cache",
            "Library",
            ".cargo",
            ".sock",
        ];
        let walker = WalkDir::new(root).into_iter();
        let mut executables: Vec<Executable> = Vec::new();
        for result in walker.filter_entry(|e| (!is_ignored(e, &ignored))) {
            let entry = match result {
                Ok(entry) => entry,
                Err(_) => panic!("Couldn't read dir!"),
            };
            if should_include_file(&entry) {
                let path = entry.into_path();
                let absolute_path = std::fs::canonicalize(&path).unwrap();
                executables.push(Executable {
                    short_name: path.file_name().unwrap().to_string_lossy().to_string(),
                    path,
                    absolute_path,
                })
            }
        }
        Self { executables }
    }

    pub fn get(&self, name: &str) -> Option<&Executable> {
        self.executables
            .iter()
            .find(|&executable| executable.short_name == name)
    }

    /// Pretty-prints the executables we found on the path, so the
    /// user can select one to run.
    pub fn pretty_print(&self) {
        print_root_header();
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
            let path = executable.path.as_os_str().to_string_lossy().to_string();
            // We'll pad right so everything aligns nicely.
            let to_print = executable
                .short_name
                .pad_to_width_with_alignment(padding, Alignment::Right);
            println!("{DARK_GREEN_FG}{to_print}{RESET_FG} - {path}");
        });
    }
}

/// Determines whether or not we should include this entry in our search results
fn should_include_file(entry: &DirEntry) -> bool {
    // We'll need to check file permissions
    let permissions = match entry.metadata() {
        Ok(metadata) => metadata.permissions(),
        Err(_) => panic!("Couldn't get file metadata for {:?}!", entry.path()),
    };

    // If we don't have permissions to access the file we're not going to get very far.
    if has_permissions(&permissions)
        // We're ignoring dirs, obviously
        && !entry.file_type().is_dir() 
        // We're including executables
        && is_executable(&permissions) 
        // We're ignoring symlinks (for now)
        && !entry.path_is_symlink()
    {
        // This involves reading the first few bytes if the file, and for performance reasons
        // we want to do this as little as possible. So it's the last thing we check.
        if !is_binary(entry)
        // We're ignoring binary files
        {
            return true;
        }
    }
    false
}

fn has_permissions(permissions: &Permissions) -> bool {
    // TODO: learn about octal representations of permissions.
    //       All I currently know is that we can't read this.
    permissions.mode() != 33279 
}

fn is_ignored(entry: &DirEntry, ignores: &[&str]) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| {
            for ignored in ignores.iter() {
                if s.contains(ignored) {
                    return true;
                }
            }
            false
        })
        .unwrap_or(false)
}

fn is_executable(permissions: &Permissions) -> bool {
    permissions.mode() & 0o111 != 0
}

fn is_binary(entry: &DirEntry) -> bool {
    let path = entry.path();
    let path_str = path.to_string_lossy();

    // We're testing for executable permissions before we check for binary or text
    // because we don't want to attempt to read any files we don't have to.
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(err) => {
            log::error!(
                "Unable to access file: {}. The error was: {}",
                path_str,
                err
            );
            return false;
        }
    };

    // We're only going to read a smidgen of the file because that's all we need
    // for using content_inspector.
    let mut buffer = [0; 10];

    let head = std::io::BufReader::new(file).read_exact(&mut buffer);
    match head {
        Ok(_) => inspect(&buffer) == ContentType::BINARY,
        Err(err) => {
            if err.to_string().as_str() == "failed to fill whole buffer" {
                log::debug!( "Found a tiny file and didn't read it all. Ignoring it. Path: {path_str}");
            } else {
                log::error!("Unable to read file: {path_str}. The error was: {err}");
            }
            false
        }
    }
}
