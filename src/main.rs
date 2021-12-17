mod bash_file;
mod executables;
mod fuzzy;
mod script;
mod ui;

use crate::{
    fuzzy::ui_state::UiState,
    script::Script,
    ui::{print_bad_function_name, print_bad_script_name},
};

use anyhow::Result;
use bash_file::BashFile;
use executables::Executables;
use fuzzy::item::Item;
use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use script::Function;
use structopt::StructOpt;
use tempfile::tempdir;

/// Use lk to explore and execute scripts in your current directory.
/// Execute lk without arguments to see what scripts are available.
/// Execute lk with a script name to see what functions are available
/// in that script. Execute lk with a script name and a function
/// name to actually run that function.
#[derive(StructOpt)]
struct Cli {
    /// Fuzzy search for available scripts and functions.
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

    let lk_dir = match dirs::home_dir() {
        // Use a dir in ~/.config like a good human, but then store logs in it lol.
        Some(home_dir) => format!("{}/.config/lk", home_dir.to_string_lossy()),
        // If we don't have access to the home_dir for some reason then just use a temp dir.
        None => tempdir().unwrap().into_path().to_string_lossy().to_string(),
    };
    let log_file_path = format!("{}/lk.log", lk_dir);
    let log_file = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build(&log_file_path)?;

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(log_file)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))?;
    log4rs::init_config(config)?;

    log::info!("\n\nStarting lk...");

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
        let result = UiState::fuzzy_find_function(scripts_to_item(&scripts)).unwrap();
        if let Some(function) = result {
            BashFile::run(function.0.to_owned(), function.1.to_owned(), [].to_vec())?;
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
                        BashFile::run(script.to_owned(), function.to_owned(), args.params)?;
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

fn scripts_to_item(scripts: &[Script]) -> Vec<Item<(&Script, &Function)>> {
    let mut fuzzy_functions: Vec<Item<(&Script, &Function)>> = Vec::new();
    scripts.iter().for_each(|script| {
        script.functions.iter().for_each(|function| {
            fuzzy_functions.push(Item::new(
                format!(
                    "{}/{} - {}",
                    script.path(),
                    script.file_name(),
                    function.name
                ),
                (script, function),
            ))
        })
    });
    fuzzy_functions
}
