/// Parses a script file and extracts comments and functions.
use crate::executables::Executable;
use crate::ui::{print_no_functions_in_script_help, print_script_header};
use anyhow::Result;
use pastel_colours::{GREEN_FG, RESET_FG};
use regex::Regex;
use std::io::BufRead;
use std::sync::LazyLock;
use std::{fs::File, path::Path};

/// Matches lines like `some_function() {`, i.e. the start of a bash function.
static FUNCTION_HEADER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^.*\(\).*\{$").expect("function header regex is valid"));

/// Everything we need to know about a function in a script
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Function {
    pub name: String,
    pub comment: Vec<String>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Script {
    pub path: std::path::PathBuf,
    pub absolute_path: std::path::PathBuf,
    pub comment: Vec<String>,
    pub functions: Vec<Function>,
}

impl Script {
    pub fn new(executable: &Executable) -> Result<Self> {
        let lines = match read_lines(&executable.path) {
            Ok(lines) => lines,
            Err(err) => {
                log::error!(
                    "Unable to read executable: {}. Error was: {err}",
                    &executable.path.to_string_lossy()
                );
                anyhow::bail!("Permission denied in opening script");
            }
        };

        // `comments` accumulates comments until we find a function header line, and then they're cleared.
        let mut comments: Vec<String> = Vec::new();
        let mut included_comments: Vec<String> = Vec::new();
        let mut included_functions: Vec<Function> = Vec::new();
        let mut in_header_comments: bool = false;
        for line in lines.map_while(Result::ok) {
            // Find lines that are part of the same comment block
            if line.starts_with('#') {
                // Are we dealing with a hashbang line? If so, then we expect
                // the next line(s) until an empty line to be script comments.
                if line.contains("#!/") {
                    in_header_comments = true;
                } else if in_header_comments {
                    let comment = clean_comment_line(&line);
                    if included_comments.is_empty() && comment.is_empty() {
                        // If we don't yet have any comments, and this comment has 0 length
                        // then we're probably dealing with a spacing line between the hashbang
                        // and the actual file header. So we'll ignore this line.
                    } else {
                        included_comments.push(comment);
                    }
                } else {
                    comments.push(clean_comment_line(&line));
                }
            } else {
                // Find lines that start a function
                if is_function_header_line(&line) {
                    included_functions.push(get_function(&line, &comments));
                }
                comments.clear();
                in_header_comments = false;
            }
        }

        Ok(Self {
            comment: included_comments,
            functions: included_functions,
            path: executable.path.to_owned(),
            absolute_path: executable.absolute_path.to_owned(),
        })
    }

    pub fn get(&self, function_name: &str) -> Option<&Function> {
        self.functions.iter().find(|&n| n.name == function_name)
    }

    pub fn file_name(&self) -> String {
        self.path
            .file_name()
            .expect("script path always ends in a file name")
            .to_string_lossy()
            .to_string()
    }

    pub fn path(&self) -> String {
        self.path.to_string_lossy().to_string()
    }

    pub fn working_dir_absolute(&self) -> String {
        let mut path = self.absolute_path.clone();
        path.pop();
        path.to_string_lossy().to_string()
    }

    pub fn pretty_print(&self) {
        print_script_header(self);
        if self.functions.is_empty() {
            print_no_functions_in_script_help();
        } else {
            self.comment.iter().for_each(|comment_line| {
                println!("  {}", comment_line);
            });

            // Get the longest function name
            const INDENT: usize = 2;
            let padding = self
                .functions
                .iter()
                .map(|function| function.name.len())
                .max()
                .map_or(0, |longest| longest + INDENT);
            for function in &self.functions {
                // We'll pad left so everything aligns nicely.
                // First print the function name
                let name = &function.name;
                let coloured_to_print = format!("{GREEN_FG}{name:>padding$}{RESET_FG}");
                if !function.comment.is_empty() {
                    print!("{coloured_to_print}");
                } else {
                    println!("{coloured_to_print}");
                }

                // Then follow up with the comment lines
                for (i, line) in function.comment.iter().enumerate() {
                    if i == 0 {
                        println!(" {line}");
                    } else {
                        println!("{:>padding$} {line}", "");
                    }
                }
            }
        }
    }
}

/// Gets a `Function` from a line that contains a function name. Uses accumulated comments.
fn get_function(line: &str, comments_found_so_far: &[String]) -> Function {
    // `split` always yields at least one item, even for an empty string.
    let name = line.split("()").next().unwrap_or_default();
    Function {
        name: name.trim().to_owned(),
        comment: comments_found_so_far.to_vec(),
    }
}

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
// https://doc.rust-lang.org/rust-by-example/std_misc/file/read_lines.html
fn read_lines<P>(filename: P) -> std::io::Result<std::io::Lines<std::io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(std::io::BufReader::new(file).lines())
}

fn is_function_header_line(line: &str) -> bool {
    // Functions starting with an underscore are treated as private and ignored.
    !line.trim().starts_with('_') && FUNCTION_HEADER.is_match(line)
}

fn clean_comment_line(line: &str) -> String {
    let mut cleaned = line.trim_start_matches('#');
    cleaned = cleaned.trim_start();
    cleaned.to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_comment_line() {
        assert_eq!(clean_comment_line("#First line"), "First line");
        assert_eq!(clean_comment_line("# First line"), "First line");
        assert_eq!(clean_comment_line("# First # line"), "First # line");
        assert_eq!(clean_comment_line("## First # line"), "First # line");
        assert_eq!(clean_comment_line("### First # line"), "First # line");
        assert_eq!(clean_comment_line("### #First # line"), "#First # line");
        assert_eq!(clean_comment_line("#"), "");
        assert_eq!(clean_comment_line("#    "), "");
        assert_eq!(clean_comment_line("#   "), "");
        assert_eq!(clean_comment_line("##   "), "");
    }

    #[test]
    fn test_get_function() {
        // Given
        let line = "some_function(){";
        let comments = vec![String::from("First line"), String::from("Second line")];

        // When
        let function = get_function(line, &comments);

        // Then
        assert_eq!(function.name, "some_function");
        assert_eq!(function.comment, vec!["First line", "Second line"]);
    }

    #[test]
    fn test_get_function_edge() {
        // Given
        let line = "   some_function   ()   {";
        let comments = vec![String::from("First line"), String::from("Second # line")];

        // When
        let function = get_function(line, &comments);

        // Then
        assert_eq!(function.name, "some_function");
        assert_eq!(function.comment, vec!["First line", "Second # line"]);
    }

    #[test]
    fn test_is_function_header_line() {
        assert!(is_function_header_line("some_function(){"));
        assert!(is_function_header_line("some_function    () {"));
        assert!(is_function_header_line("some_function    ()     {"));
        assert!(is_function_header_line("    some_function    ()     {"));
        assert!(!is_function_header_line("_private_function() {"));
        assert!(!is_function_header_line("echo \"not a function\""));
    }
}
