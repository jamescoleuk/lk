use crate::models::{Function, Script};
use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

// Takes in a script path and returns a list of Functions.
pub fn get_functions(script: &std::path::Path) -> Result<Script, std::io::Error> {
    let mut script: Script = Script::new(script.to_owned());

    let lines = read_lines(&script.path)?;

    // `comments` accumulates comments until we find a function header line, and then they're cleared.
    let mut comments: Vec<String> = Vec::new();
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
                // If we don't yet have any comments, and this comment has 0 length
                // then we're probably dealing with a spacing line between the hashbang
                // and the actual file header. So we'll ignore this line.
                if script.comment.len() == 0 && comment.len() != 0 {
                    script.comment.push(comment);
                }
            } else {
                comments.push(clean_comment_line(&line));
            }
        } else if !line.starts_with('#') {
            // Find lines that start a function
            if is_function_header_line(&line) {
                let function = get_function(line, &comments);
                script.functions.push(function);
            }
            comments.clear();
            in_header_comments = false;
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

fn clean_comment_line(line: &str) -> String {
    let mut cleaned = line.trim_start_matches('#');
    cleaned = cleaned.trim_start();
    cleaned.to_owned()
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
