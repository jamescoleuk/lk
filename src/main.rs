mod bash_file;
mod executables;
mod script;
pub mod ui;
use anyhow::Result;
use bash_file::BashFile;
use colored::Colorize;
use executables::Executables;

use crate::{
    script::Script,
    ui::{print_bad_function_name, print_bad_script_name},
};
use structopt::StructOpt;

/// Use lk to explore and execute scripts in your current directory.
/// Execute lk without arguments to see what scripts are available.
/// Execute lk with a script name to see what functions are available
/// in that script. Execute lk with a script name and a function
/// name to actually run that function.
#[derive(StructOpt)]
struct Cli {
    /// Optional: the name of a script to explore or use
    script: Option<String>,
    /// Optional: the name of the function to run.
    function: Option<String>,
    /// Optional: params for the function. We're not processing them yet (e.g. validating) but
    /// they need to be permitted as a param to lk.
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
                print_bad_script_name(&script, executables);
                None
            }
        },
        None => {
            executables.pretty_print();
            None
        }
    };

    // Did we find a script?
    if let Some(script) = script {
        // Did the user pass a function?
        if let Some(function) = args.function {
            // Is it a function that exists in the script we found?
            if let Some(function) = script.get(&function) {
                // Do our thing
                let bash_file = BashFile::new(script.to_owned(), function.to_owned(), args.params);
                bash_file.write()?;
                bash_file.execute()?;
            } else {
                print_bad_function_name(&script, &function);
            }
        } else {
            script.pretty_print();
        }
    }

    Ok(())
}
