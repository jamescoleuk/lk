use colored::*;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use structopt::StructOpt;

/// Run or list the contents of a script
#[derive(StructOpt)]
struct Cli {
    /// The path to the script to describe or run.
    #[structopt(parse(from_os_str))]
    script: std::path::PathBuf,
    /// The name of the function to run.
    function: Option<String>,
}

/// Everything we need to know about a function in a script
#[derive(PartialEq, Debug)]
struct Function {
    name: String,
    comment: Vec<String>,
}

/// Errors we might generate.
/// TODO: I think there's probably a better way to do this. Revisit when I've got more rust experience.
#[derive(PartialEq, Debug)]
enum RunshError {
    BadFunctionName,
    BadScriptPath,
}

fn main() {
    let args = Cli::from_args();
    match get_functions(&args.script) {
        Ok(functions) => match &args.function {
            Some(function_to_run) => {
                match functions
                    .iter()
                    .find(|&n| n.name == String::from(function_to_run))
                {
                    Some(_) => {
                        // Found a valid function. We're going to return a non-0 exit code
                        // so the script knows that it can go ahead and run the function.
                        std::process::exit(78);
                    }
                    None => {
                        let script = &args.script.into_os_string().into_string().unwrap();
                        println!("{}", "Function does not exist!\n".red());
                        print_functions(functions, script);
                    }
                }
            }
            None => {
                let script = &args.script.into_os_string().into_string().unwrap();
                print_functions(functions, script);
            }
        },
        Err(_) => {
            let script = &args.script.into_os_string().into_string().unwrap();
            println!(
                "{} {}",
                "Unable to get functions from".red(),
                script.green()
            );
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

// Takes in a script path and returns a list of Functions.
fn get_functions(script: &std::path::PathBuf) -> Result<Vec<Function>, RunshError> {
    let mut functions: Vec<Function> = Vec::new();
    match read_lines(script) {
        Ok(lines) => {
            let mut comments: Vec<String> = Vec::new();
            for maybe_line in lines {
                if let Ok(line) = maybe_line {
                    // Find lines that are part of the same comment block
                    if line.starts_with('#') {
                        comments.push(line);
                    } else if !line.starts_with('#') {
                        // Find lines that start a function
                        if line.contains("(){") || line.contains("() {") {
                            let function = get_function(line, &comments)?;
                            functions.push(function);
                        }
                        comments.clear();
                    }
                }
            }
            Result::Ok(functions)
        }
        Err(_) => Result::Err(RunshError::BadScriptPath),
    }
}

/// Gets a `Function` from a line that contains a function name. Uses accumulated comments.
fn get_function(line: String, comments_found_so_far: &Vec<String>) -> Result<Function, RunshError> {
    let name = line.split("()").next();
    match name {
        Some(actual_name) => {
            let cleaned_comments = comments_found_so_far
                .iter()
                .map(|comment| comment.trim_start_matches("#"))
                .map(|comment| comment.trim_start())
                .map(|comment| String::from(comment));
            let cleaned_name = actual_name.trim();
            let this_function = Function {
                name: String::from(cleaned_name),
                comment: cleaned_comments.collect(),
            };
            return Ok(this_function);
        }
        None => {
            println!(
                "{} {} ",
                "There is some kind of formatting error with the name of this function:".red(),
                &line
            );
            Result::Err(RunshError::BadFunctionName)
        }
    }
}

/// Pretty-prints the passed functions
fn print_functions(functions: Vec<Function>, script: &String) {
    if functions.len() == 0 {
        println!(
            "{} has found no functions in {}. You could add some like this:",
            "Runsh",
            script.bright_blue()
        );
        let example_function = r#"# Some great comment
# More insightful and fascinating insights into bash scripting
blow_mind() {
    echo "OMG so cool"
} "#;
        println!("{}", example_function.green());
    } else {
        let example_command = format!("./{} {}", script, functions[0].name);
        println!(
            "{} has found the following functions in {}. Execute them like this: \n   {}\n",
            "Runsh",
            script.bright_blue(),
            example_command.green()
        );
        for function in functions {
            println!("  {}", function.name.on_blue());
            for line in function.comment {
                println!("    {}", line);
            }
        }
    }
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
        let result = get_function(line, &comments);

        // Then
        assert_eq!(result.is_ok(), true);
        let function = result.ok().unwrap();
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
        let result = get_function(line, &comments);

        // Then
        assert_eq!(result.is_ok(), true);
        let function = result.ok().unwrap();
        assert_eq!(function.name, "some_function");
        assert_eq!(function.comment, vec!["First line", "Second # line"]);
    }
}
