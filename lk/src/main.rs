mod bash_file;
mod config;
mod executables;
mod script;
mod ui;

use anyhow::Result;
use bash_file::BashFile;
use executables::Executables;
use fuzzy_finder::item::Item;
use fuzzy_finder::FuzzyFinder;
use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use pastel_colours::{COLOUR_GREEN, COLOUR_RED};
use script::Function;
use spinners::{Spinner, Spinners};
use structopt::StructOpt;
use tempfile::tempdir;
use termion::color;
use ui::{print_bad_function_name, print_bad_script_name};

use crate::script::Script;

/// Use lk to explore and execute scripts in your current directory,
/// and in its sub-directories. lk offers two options: 'list' or 'fuzzy'.
/// 'list' lets you explore your scripts and their functions in a
/// hierarchical way. 'fuzzy' lets you do a fuzzy search over all the
/// scripts and functions found by lk.
#[derive(StructOpt)]
struct Cli {
    /// Set the default mode: fuzzy or list
    #[structopt(long, short)]
    default: Option<String>,
    /// Fuzzy search for available scripts and functions.
    #[structopt(long, short)]
    fuzzy: bool,
    /// List available scripts and functions.
    #[structopt(long, short)]
    list: bool,
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
    let lk_dir = match dirs::home_dir() {
        // Use a dir in ~/.config like a good human, but then store logs in it lol.
        Some(home_dir) => format!("{}/.config/lk", home_dir.to_string_lossy()),
        // If we don't have access to the home_dir for some reason then just use a temp dir.
        None => {
            println!("Unable to access your home directory. Using a temporary directory instead.");
            tempdir().unwrap().into_path().to_string_lossy().to_string()
        }
    };

    let mut config_file = config::ConfigFile::new(&lk_dir, "lk.toml");

    let args = Cli::from_args();

    let log_file_path = format!("{}/lk.log", lk_dir);
    let log_file = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build(&log_file_path)?;

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(log_file)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))?;
    log4rs::init_config(config)?;

    log::info!("\n\nStarting lk...");

    let sp = Spinner::new(&Spinners::Line, "".to_string());
    let executables = Executables::new(".");
    sp.stop();

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
    if let Some(default) = args.default {
        match default.as_str() {
            "fuzzy" => {
                println!(
                    "Setting default mode to {}fuzzy{}",
                    color::Fg(COLOUR_GREEN),
                    color::Fg(color::Reset)
                );
                config_file.config.default_mode = "fuzzy".to_string();
                config_file.save();
            }
            "list" => {
                println!(
                    "Setting default mode to {}list{}",
                    color::Fg(COLOUR_GREEN),
                    color::Fg(color::Reset)
                );
                config_file.config.default_mode = "list".to_string();
                config_file.save();
            }
            _ => {
                // Truely hideous code. It wasn't much better with crossterm.
                println!(
                    "{}Unknown default!{} Please specify either {}fuzzy{} or {}list{}. You can try out either using the {}--fuzzy{} or {}--list{} flags.",
                    color::Fg(COLOUR_RED),
                    color::Fg(color::Reset),
                    color::Fg(COLOUR_GREEN),
                    color::Fg(color::Reset),
                    color::Fg(COLOUR_GREEN),
                    color::Fg(color::Reset),
                    color::Fg(COLOUR_GREEN),
                    color::Fg(color::Reset),
                    color::Fg(COLOUR_GREEN),
                    color::Fg(color::Reset),
                );
            }
        }
    } else if args.fuzzy {
        fuzzy(&scripts)?
    } else if args.list || args.script.is_some() {
        // If the user is specifying --list OR if there's some value for script.
        // Any value there is implicitly take as --list.
        list(executables, args)?
    } else {
        // Neither requested, so fall back on the default which will always exist.
        match config_file.config.default_mode.as_str() {
            "fuzzy" => fuzzy(&scripts)?,
            "list" => list(executables, args)?,
            _ => panic!("No default mode set! Has there been a problem creating the config file?"),
        }
    }
    Ok(())
}

fn fuzzy(scripts: &[Script]) -> Result<()> {
    let result = FuzzyFinder::find(scripts_to_item(scripts)).unwrap();
    if let Some(function) = result {
        BashFile::run(function.0.to_owned(), function.1.to_owned(), [].to_vec())?;
    }
    Ok(())
}

fn list(executables: Executables, args: Cli) -> Result<()> {
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
