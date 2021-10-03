mod executables;
mod rn_file;
mod script;
use anyhow::Result;
use executables::Executables;
use rn_file::execute_rn_file;
use rn_file::write_rn_file;

use crate::script::Script;
use structopt::StructOpt;

/// Run or list the contents of a script. Run by itself it will try and find scripts it can run.
#[derive(StructOpt)]
struct Cli {
    /// The name of the script to describe or run.
    script: Option<String>,
    /// The name of the function to run. This will not run the function, it will just validate that it exists.
    function: Option<String>,
    /// Optional params for the function. We're not processing them yet (e.g. validating) but
    /// they need to be permitted as a param to rn.
    #[allow(dead_code)]
    params: Vec<String>,
}

fn main() -> Result<()> {
    let args = Cli::from_args();

    let executables = Executables::new(".");

    // Have they requested a script, and if they did does it exist?
    let script = match args.script {
        Some(script) => match executables.get(&script) {
            Some(executable) => Some(Script::new(executable)),
            None => {
                println!("Didn't find a script with name {}", script);
                executables.pretty_print();
                None
            }
        },
        None => {
            executables.pretty_print();
            None
        }
    };

    if script.is_some() {
        let script = script.unwrap();
        let function = match args.function {
            Some(function) => match script.get(&function) {
                Some(function) => Some(function),
                None => {
                    println!("Didn't find a function with name {}", function);
                    script.pretty_print();
                    None
                }
            },

            None => {
                script.pretty_print();
                None
            }
        };

        if function.is_some() {
            write_rn_file(&script, &function.unwrap())?;
            execute_rn_file()?;
        }
    }
    Ok(())
}
