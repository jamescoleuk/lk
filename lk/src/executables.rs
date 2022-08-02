/// Finds executables in the current directory.
use crate::ui::print_root_header;
use anyhow::{bail, Result};
use content_inspector::{inspect, ContentType};
use glob::glob;
#[cfg(not(test))]
use log::{debug, error};
use pad::{Alignment, PadStr};
use pastel_colours::{DARK_GREEN_FG, RESET_FG};
use std::{fs::Permissions, io::Read, os::unix::fs::PermissionsExt, path::PathBuf};
#[cfg(test)]
use std::{println as debug, println as error}; //

#[derive(Debug)]
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
    pub fn new(includes: &[String], excludes: &[String]) -> Result<Self> {
        // Validate to ensure there is at least on include.
        if includes.is_empty() {
            bail!("No includes specified.");
        }

        // Get all the excluded files
        let mut files_to_exclude: Vec<PathBuf> = Vec::new();
        for exclude in excludes {
            for entry in glob(exclude)? {
                match entry {
                    Ok(path) => files_to_exclude.push(path),
                    Err(e) => error!("{:?}", e),
                }
            }
        }

        // Get all the included files but not the excluded ones.
        let mut files_to_include: Vec<PathBuf> = Vec::new();
        for include in includes {
            for entry in glob(include)? {
                match entry {
                    Ok(path) => {
                        // Exclude subpaths and full paths that are in the excludes list.
                        let is_subpath = files_to_exclude
                            .iter()
                            .any(|exclude| path.starts_with(exclude));

                        if !files_to_exclude.contains(&path)
                            && !is_subpath
                            && should_include_file(&path)
                        {
                            files_to_include.push(path);
                        }
                    }
                    Err(e) => error!("{:?}", e),
                }
            }
        }
        debug!("Excluding {:?}", files_to_exclude);
        debug!("Including {:?}", files_to_include);

        let executables: Vec<Executable> = files_to_include
            .into_iter()
            .map(|include| {
                let path = include.to_str().unwrap();
                let absolute_path = include.clone();
                let short_name = path.split('/').last().unwrap().to_string();

                Executable {
                    short_name,
                    path: include,
                    absolute_path,
                }
            })
            .collect();

        debug!("{:?}", executables);
        Ok(Self { executables })
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
        let padding = if self.executables.is_empty() {
            0 // If we don't have any executables we don't care about the padding, so just use 0.
        } else {
            self.executables
                .iter()
                .max_by(|x, y| x.short_name.len().cmp(&y.short_name.len()))
                .unwrap() // Will always be Some because the name String must exist.
                .short_name
                .len()
                + INDENT
        };
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
fn should_include_file(path: &PathBuf) -> bool {
    // We'll need to check file permissions
    let permissions = match path.metadata() {
        Ok(metadata) => metadata.permissions(),
        Err(_) => panic!("Couldn't get file metadata for {:?}!", path),
    };

    // If we don't have permissions to access the file we're not going to get very far.
    if has_permissions(&permissions)
        // We're ignoring dirs, obviously
        && !path.is_dir()
        // We're including executables
        && is_executable(&permissions)
        // We're ignoring symlinks (for now)
        && !path.is_symlink()
    {
        // This involves reading the first few bytes if the file, and for performance reasons
        // we want to do this as little as possible. So it's the last thing we check.
        if !is_binary(path)
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

fn is_executable(permissions: &Permissions) -> bool {
    permissions.mode() & 0o111 != 0
}

fn is_binary(path: &PathBuf) -> bool {
    // let path = entry.path();
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
                log::debug!(
                    "Found a tiny file and didn't read it all. Ignoring it. Path: {path_str}"
                );
            } else {
                log::error!("Unable to read file: {path_str}. The error was: {err}");
            }
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_should_include_all_files() {
        let executables = Executables::new(&["**/*".to_string()], &[]);
        // This depends on the number of scripts in the tests directory - so please take care when changing those files.
        assert_eq!(executables.unwrap().executables.len(), 10);
    }

    #[test]
    fn should_include_only_specific_folder() {
        let executables = Executables::new(&["**/tests/executables_tests/**/*".to_string()], &[]);
        // This depends on the number of scripts in the tests directory - so please take care when changing those files.
        assert_eq!(executables.unwrap().executables.len(), 4);
    }

    #[test]
    fn should_include_multiple_specific_folders() {
        let executables = Executables::new(
            &[
                "**/tests/executables_tests/**/*".to_string(),
                "**/tests/depends_on_file/**/*".to_string(),
            ],
            &[],
        );
        // This depends on the number of scripts in the tests directory - so please take care when changing those files.
        assert_eq!(executables.unwrap().executables.len(), 6);
    }

    #[test]
    fn should_exclude_multiple_specific_folders() {
        let executables = Executables::new(
            &["**/*.*".to_string()],
            &[
                "**/tests/depends_on_file/**/*".to_string(),
                "**/tests/executables_tests/**/*".to_string(),
            ],
        );
        // This depends on the number of scripts in the tests directory - so please take care when changing those files.
        assert_eq!(executables.unwrap().executables.len(), 4);
    }

    #[test]
    fn should_exclude_by_file_folder() {
        let executables = Executables::new(
            &["**/tests/**/*.*".to_string()],
            &["*/**/exclude_me".to_string()],
        );
        // This depends on the number of scripts in the tests directory - so please take care when changing those files.
        assert_eq!(executables.unwrap().executables.len(), 9);
    }

    #[test]
    fn should_exclude_by_file_name() {
        let executables = Executables::new(
            &["**/tests/**/*".to_string()],
            &["*/**/exclude_me/should_not_be_included.sh".to_string()],
        );
        // This depends on the number of scripts in the tests directory - so please take care when changing those files.
        assert_eq!(executables.unwrap().executables.len(), 9);
    }

    #[test]
    fn should_fail_when_no_includes() {
        let executables = Executables::new(&[], &[]);
        assert!(executables.is_err());
    }

    //FIXME: uncomment when workspace has been removed.
    // #[test]
    // fn should_include_scripts_in_pwd() {
    //     // Should include everything in the current dir.
    //     let executables = Executables::new(&["*".to_string()], &[]);
    //     // This depends on the number of scripts in the tests directory - so please take care when changing those files.

    //     assert!(!executables.unwrap().executables.is_empty());
    // }
}
