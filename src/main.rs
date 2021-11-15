mod bash_file;
mod executables;
mod fuzzy_find;
mod script;
pub mod ui;
use crate::{
    script::Script,
    ui::{print_bad_function_name, print_bad_script_name},
};
use fuzzy_find::fuzzy_find_function;

use anyhow::Result;
use bash_file::BashFile;
use executables::Executables;
use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use script::Function;
use structopt::StructOpt;

/// Use lk to explore and execute scripts in your current directory.
/// Execute lk without arguments to see what scripts are available.
/// Execute lk with a script name to see what functions are available
/// in that script. Execute lk with a script name and a function
/// name to actually run that function.
#[derive(StructOpt)]
struct Cli {
    #[structopt(long, short)]
    fuzzy: bool,
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

    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build("output.log")?;

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))?;

    log4rs::init_config(config)?;

    log::info!("Hello, world!");

    let executables = Executables::new(".");

    let scripts: Vec<Script> = executables
        .executables
        .iter()
        .map(|executable| Script::new(executable))
        .collect();

    // Prints all scripts
    // scripts.iter().for_each(|script| {
    //     script
    //         .functions
    //         .iter()
    //         .for_each(|function| println!("{} - {}", script.file_name(), function.name))
    // });

    if args.fuzzy {
        let result = fuzzy_find_function(&scripts)?;
        if result.is_some() {
            let fuz = result.unwrap();
            execute(fuz.script.to_owned(), fuz.function.to_owned(), [].to_vec())?;
        }
    } else {
        // Did the user request a script?
        if let Some(script) = args.script {
            // Is it a script that exists on disk?
            if let Some(executable) = executables.get(&script) {
                // Yay, confirmed script
                let script = Script::new(executable);
                // Did the user pass a function?
                if let Some(function) = args.function {
                    // Is it a function that exists in the script we found?
                    if let Some(function) = script.get(&function) {
                        // Do our thing
                        execute(script.to_owned(), function.to_owned(), args.params)?;
                    } else {
                        print_bad_function_name(&script, &function);
                    }
                } else {
                    // No function, display a list of what's available
                    script.pretty_print();
                }
            } else {
                print_bad_script_name(&script, executables);
            }
        } else {
            // No executable, display a list of what's available
            executables.pretty_print();
        }
    }

    Ok(())
}

fn execute(script: Script, function: Function, params: Vec<String>) -> Result<()> {
    let bash_file = BashFile::new(script.to_owned(), function.to_owned(), params);
    bash_file.write()?;
    bash_file.execute()
}
