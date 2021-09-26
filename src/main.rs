mod models;
mod parser;
mod pretty_printer;
mod script_manager;
use anyhow::{anyhow, Result};

use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

use crate::parser::get_functions;
use crate::pretty_printer::print_script;
use crate::script_manager::find_executables;
use colored::*;
use models::ValidatedRequest;
use pretty_printer::print_executables;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use structopt::StructOpt;

/// Run or list the contents of a script. Run by itself it will try and find scripts it can run.
#[derive(StructOpt)]
struct Cli {
    /// The name of the script to describe or run.
    script: Option<String>,
    /// The name of the function to run. This will not run the function, it will just validate that it exists.
    function: Option<String>,
    /// Optional params for the function. We're not processing them yet (e.g. validating) but
    /// they need to be permitted as a param to runsh.
    #[allow(dead_code)]
    params: Vec<String>,
}

fn main() -> Result<()> {
    let args = Cli::from_args();

    let executables = find_executables();

    match args.script {
        // If the user passed a script name then we want to validate it
        Some(script_name) => {
            let executable = executables
                .iter()
                .find(|&executable| executable.short_name == script_name);
            match executable {
                // If the user passed a valid function then we want to run it.
                Some(e) => {
                    let validated_request = process_script_request(&e.path, args.function)?;
                    match write_runsh_file(validated_request) {
                        Ok(_) => execute_runsh_file(),
                        Err(e) => {
                            eprintln!(
                                "Unable to write out runsh's temporary file! The error was {}",
                                e.to_string()
                            );
                            Err(e)
                        }
                    }
                }
                // If the user did not pass a valid function then we want to let them know.
                None => {
                    println!("Couldn't find a script called '{}'", script_name);
                    print_executables(executables);
                    Ok(())
                }
            }
        }
        // If the user didn't pass a script name we want to show what scripts there are available
        None => {
            print_executables(executables);
            Ok(())
        }
    }
}

fn process_script_request(script: &PathBuf, function: Option<String>) -> Result<ValidatedRequest> {
    let script_name: String = script.as_path().to_string_lossy().to_string();
    let script = get_functions(script.as_path())?;
    match function {
        Some(function_to_run) => {
            match script.functions.iter().find(|&n| n.name == function_to_run) {
                Some(_) => {
                    return Ok(ValidatedRequest {
                        script_name: script_name,
                        function_to_run: Option::Some(function_to_run),
                    });
                }
                None => {
                    println!("{}", "Function does not exist!\n".red());
                    print_script(script);

                    Err(anyhow!(
                        "Function does not exist! Requested function was: {}",
                        function_to_run
                    ))
                }
            }
        }
        None => {
            print_script(script);
            Ok(ValidatedRequest {
                script_name: script_name,
                function_to_run: Option::None,
            })
        }
    }
}

/// Runsh uses a temporary file in order to execute a function in a script. This temporary file
/// sources the script we're going to execute and then it can run the function because it'll
/// have been loaded into the shell. `std::process::Command` has no way to do this. An alternative
/// would be adding `"$@"` to the end of the scripts but I'd rather avoid this stipulation.
fn write_runsh_file(validated_request: ValidatedRequest) -> Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .mode(0o700)
        .open("~runsh")?;
    let runsh_file = r#"#!/usr/bin/env bash
# 
# Temporary runsh file used to execute functions in scripts.
# If you see it here you can delete it and/or gitignore it.

"#;
    writeln!(
        file,
        "{} source {} && {}",
        runsh_file,
        validated_request.script_name,
        validated_request.function_to_run.unwrap()
    )?;
    Ok(())
}

/// This executes the runsh file, and then removes it.
fn execute_runsh_file() -> Result<()> {
    let mut cmd = Command::new("./~runsh")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();

    let exit_status = cmd.wait()?;
    match exit_status.code() {
        Some(code) => {
            match std::fs::remove_file("./~runsh") {
                Ok(_) => {
                    // Great, we've tidied up.
                }
                Err(e) => {
                    eprintln!(
                        "Yikes! I couldn't remove my temporary file, './~runsh'! The error was {}",
                        e.to_string()
                    )
                }
            }
            std::process::exit(code)
        }
        None => eprintln!("Your function exited without a status code!"),
    }
    Ok(())
}
