use crate::models::Script;
use colored::*;
use pad::{Alignment, PadStr};

// TODO: make this a Display impl on Script?
pub fn print_script(script: Script) {
    let script_path = script.path.into_os_string().into_string().unwrap();
    if script.functions.is_empty() {
        println!(
            "Runsh has found no functions in {}. You could add some like this:",
            script_path.bright_blue()
        );
        let example_function = r#"# Some great comment
# More insightful and fascinating insights into bash scripting
blow_mind() {
    echo "OMG so cool"
} "#;
        println!("{}", example_function.green());
    } else {
        let example_command = format!("./{} {}", script_path, script.functions[0].name);
        print!("{}", script_path.on_blue());
        println!(" - Usage: {}\n", example_command.blue());

        script.comment.iter().for_each(|comment_line| {
            println!("{}", comment_line);
        });

        // Get the longest function name
        const INDENT: usize = 2;
        let padding = script
            .functions
            .iter()
            .max_by(|x, y| x.name.len().cmp(&y.name.len()))
            .unwrap() // Will always be Some because the name String must exist.
            .name
            .len()
            + INDENT;
        for function in script.functions {
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
