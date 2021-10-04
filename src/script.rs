use std::io::BufRead;
use std::{fs::File, path::Path};

use colored::Colorize;
use pad::{Alignment, PadStr};
use regex::bytes::Regex;

use crate::executables::Executable;

/// Everything we need to know about a function in a script
#[derive(PartialEq, Debug, Clone)]
pub struct Function {
    pub name: String,
    pub comment: Vec<String>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Script {
    pub path: std::path::PathBuf,
    pub comment: Vec<String>,
    pub functions: Vec<Function>,
}

impl Script {
    pub fn new(executable: &Executable) -> Self {
        // let mut script: Script = Script::new(script.to_owned());

        let lines = match read_lines(&executable.path) {
            Ok(lines) => lines,
            Err(e) => panic!("{}", e),
        };

        // `comments` accumulates comments until we find a function header line, and then they're cleared.
        let mut comments: Vec<String> = Vec::new();
        let mut included_comments: Vec<String> = Vec::new();
        let mut included_functions: Vec<Function> = Vec::new();
        let mut in_header_comments: bool = false;
        for line in lines.flatten() {
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
            } else if !line.starts_with('#') {
                // Find lines that start a function
                if is_function_header_line(&line) {
                    let function = get_function(line, &comments);
                    included_functions.push(function);
                }
                comments.clear();
                in_header_comments = false;
            }
            // }
        }

        Self {
            comment: included_comments,
            functions: included_functions,
            path: executable.path.to_owned(),
        }
    }

    pub fn get(&self, function_name: &str) -> Option<&Function> {
        self.functions.iter().find(|&n| n.name == function_name)
    }

    pub fn path(&self) -> String {
        return self.path.as_os_str().to_string_lossy().to_string();
    }

    pub fn pretty_print(&self) {
        let script_path = self.path.to_owned().into_os_string().into_string().unwrap();
        println!("{}{}", "lk: ".on_blue(), script_path.on_blue());
        if self.functions.is_empty() {
            println!("Could not find any functions! Why not add some. They look like this:");
            let example_function = r#"# Some great comment
# More insightful and fascinating insights into bash scripting
blow_mind() {
    echo "OMG so cool"
} "#;
            println!("{}", example_function.italic());
        } else {
            self.comment.iter().for_each(|comment_line| {
                println!("  {}", comment_line);
            });

            // Get the longest function name
            const INDENT: usize = 2;
            let padding = self
                .functions
                .iter()
                .max_by(|x, y| x.name.len().cmp(&y.name.len()))
                .unwrap() // Will always be Some because the name String must exist.
                .name
                .len()
                + INDENT;
            for function in &self.functions {
                // We'll pad right so everything aligns nicely.
                // First print the function name
                let to_print = function
                    .name
                    .pad_to_width_with_alignment(padding, Alignment::Right)
                    .green();
                if !function.comment.is_empty() {
                    print!("{}", to_print);
                } else {
                    println!("{}", to_print);
                }

                // Then follow up with the comment lines
                function.comment.iter().enumerate().for_each(|(i, line)| {
                    if i == 0 {
                        println!(" {}", line);
                    } else {
                        println!(
                            "{} {}",
                            "".pad_to_width_with_alignment(padding, Alignment::Right),
                            line
                        );
                    }
                });
            }
        }
    }
}

/// Gets a `Function` from a line that contains a function name. Uses accumulated comments.
fn get_function(line: String, comments_found_so_far: &[String]) -> Function {
    let name = line.split("()").next();
    match name {
        Some(actual_name) => Function {
            name: String::from(actual_name.trim()),
            comment: comments_found_so_far
                .iter()
                .map(|comment| comment.to_owned())
                .collect(),
        },
        None => {
            panic!("There is some kind of formatting error with the name of this function:");
        }
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
    if line.trim().starts_with('_') {
        false
    } else {
        Regex::new(r"^.*\(\).*\{$")
            .unwrap()
            .is_match(line.as_bytes())
    }
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
        let line = String::from("some_function(){");
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
        let line = String::from("   some_function   ()   {");
        let comments = vec![String::from("First line"), String::from("Second # line")];

        // When
        let function = get_function(line, &comments);

        // Then
        assert_eq!(function.name, "some_function");
        assert_eq!(function.comment, vec!["First line", "Second # line"]);
    }

    #[test]
    fn test_is_function_header_line() {
        assert_eq!(
            is_function_header_line(&String::from("some_function(){")),
            true
        );
        assert_eq!(
            is_function_header_line(&String::from("some_function    () {")),
            true
        );
        assert_eq!(
            is_function_header_line(&String::from("some_function    ()     {")),
            true
        );
        assert_eq!(
            is_function_header_line(&String::from("    some_function    ()     {")),
            true
        );
    }
}
