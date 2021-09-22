use crate::models::{Function, Script};
use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

// Takes in a script path and returns a list of Functions.
pub fn get_functions(script: &std::path::PathBuf) -> Result<Script, std::io::Error> {
    let mut script: Script = Script::new(script.to_owned());

    let lines = read_lines(&script.path)?;

    // `comments` accumulates comments until we find a function header line, and then they're cleared.
    let mut comments: Vec<String> = Vec::new();
    for line in lines.flatten() {
        // Find lines that are part of the same comment block
        if line.starts_with('#') {
            comments.push(line);
        } else if !line.starts_with('#') {
            // Find lines that start a function
            if is_function_header_line(&line) {
                let function = get_function(line, &comments);
                script.functions.push(function);
            }
            comments.clear();
        }
    }
    Ok(script)
}

fn is_function_header_line(line: &str) -> bool {
    if line.trim().starts_with('_') {
        false
    } else {
        let function_header_regex = Regex::new(r"^.*\(\).*\{$").unwrap();
        function_header_regex.is_match(line)
    }
}

/// Gets a `Function` from a line that contains a function name. Uses accumulated comments.
fn get_function(line: String, comments_found_so_far: &[String]) -> Function {
    let name = line.split("()").next();
    match name {
        Some(actual_name) => {
            let cleaned_comments = comments_found_so_far
                .iter()
                .map(|comment| comment.trim_start_matches('#'))
                .map(|comment| comment.trim_start())
                .map(String::from);
            let cleaned_name = actual_name.trim();
            Function {
                name: String::from(cleaned_name),
                comment: cleaned_comments.collect(),
            }
        }
        None => {
            panic!("There is some kind of formatting error with the name of this function:");
        }
    }
}

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
// https://doc.rust-lang.org/rust-by-example/std_misc/file/read_lines.html
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_function() {
        // Given
        let line = String::from("some_function(){");
        let comments = vec![String::from("# First line"), String::from("# Second line")];

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
        let comments = vec![
            String::from("#### First line"),
            String::from("# Second # line"),
        ];

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
