mod bash_file;
mod executables;
mod script;
mod shells;
mod ui;

use anyhow::Result;
use bash_file::BashFile;
use config::{Config, File};
use executables::Executables;
use fuzzy_finder::item::Item;
use fuzzy_finder::FuzzyFinder;

use log::{debug, info, LevelFilter};
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use script::Function;
use shells::UserShell;
use spinners::{Spinner, Spinners};
use std::path::Path;
use structopt::StructOpt;
use tempfile::tempdir;
use ui::{print_bad_function_name, print_bad_script_name};

mod tui;
/// Use lk to explore and execute scripts in your current directory,
/// and in its sub-directories. lk offers two options: 'list' or 'fuzzy'.
/// 'list' lets you explore your scripts and their functions in a
/// hierarchical way. 'fuzzy' lets you do a fuzzy search over all the
/// scripts and functions found by lk.
#[derive(StructOpt)]
struct Cli {
    /// Fuzzy search for available scripts and functions.
    #[structopt(long, short)]
    fuzzy: bool,

    /// List available scripts and functions.
    #[structopt(long, short)]
    list: bool,

    /// Show a full screen UI with lots of details
    #[structopt(long, short)]
    tui: bool,

    /// Optional: the name of a script to explore or use
    script: Option<String>,

    /// Optional: the name of the function to run.
    function: Option<String>,

    /// Optional: paths to include in the search, as a UNIX glob pattern.
    #[structopt(long, short)]
    includes: Vec<String>,

    /// Optional: paths to exclude in the search, as a UNIX glob pattern.
    #[structopt(long, short)]
    excludes: Vec<String>,

    /// Number of lines to show in fuzzy search.
    #[structopt(long, short = "n", default_value = "7")]
    number: i8,

    /// Optional: params for the function. We're not processing them yet (e.g. validating) but
    /// they need to be permitted as a param to lk.
    #[allow(dead_code)]
    params: Vec<String>,
}

fn main() -> Result<()> {
    // We will use the home directory to store lk configuration and log files.
    let lk_dir = match dirs::home_dir() {
        // Use a dir in ~/.config like a good human, but then store logs in it lol.
        Some(home_dir) => format!("{}/.config/lk", home_dir.to_string_lossy()),
        // If we don't have access to the home_dir for some reason then just use a temp dir.
        None => {
            println!("Unable to access your home directory. Using a temporary directory instead.");
            tempdir().unwrap().into_path().to_string_lossy().to_string()
        }
    };

    // Configure logging
    let log_file_path = format!("{lk_dir}/lk.log");
    let log_file = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build(log_file_path)?;

    let config = log4rs::config::Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(log_file)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))?;
    log4rs::init_config(config)?;

    info!("\n\nStarting lk...");

    let args = Cli::from_args();

    let mut sp = Spinner::new(Spinners::Line, "".to_string());

    // Set configuration defaults, then load the user config followed by a workspace if they exist.
    // Configurations in later files override earlier ones. However, command line configuration overrides these
    let builder = Config::builder()
        .set_default("default_mode", "fuzzy")?
        .set_default("includes", vec!["**/*".to_string(), "*".to_string()])?
        .set_default(
            "excludes",
            vec![
                // TODO: List these in help
                // TODO: write these to global config if there's no exclude section already
                "target".to_string(),
                ".github".to_string(),
                ".vscode".to_string(),
                ".git".to_string(),
                "node_modules".to_string(),
                ".nvm".to_string(),
                ".Trash".to_string(),
                ".npm".to_string(),
                ".cache".to_string(),
                "Library".to_string(),
                ".cargo".to_string(),
                ".sock".to_string(),
            ] as Vec<String>,
        )?
        .add_source(File::from(Path::new(&lk_dir).join("lk.toml")).required(false))
        .add_source(File::from(Path::new(".").join("lk.toml")).required(false));

    let config = builder.build()?;

    // Extract the config file includes and excludes
    let config_includes = config.get::<Vec<String>>("includes").unwrap();
    let config_excludes = config.get::<Vec<String>>("excludes").unwrap();

    // Merge the command line includes and excludes with the config file includes and excludes
    let includes: Vec<String> = args
        .includes
        .clone()
        .into_iter()
        .chain(config_includes)
        .collect();

    let excludes: Vec<String> = args
        .excludes
        .clone()
        .into_iter()
        .chain(config_excludes)
        .collect();

    let default_mode = config.get::<String>("default_mode").unwrap();

    info!(
        "Using default_mode {:?}, includes {:?} and excludes {:?}",
        default_mode, includes, excludes
    );

    //TODO: what should the root be and how does it overlap with the defaults or user specified includes?
    // What executable scripts are available in the configuration directory?
    let executables = Executables::new(&includes, &excludes)?;

    sp.stop();

    // What functions do these executables contain?
    let scripts: Vec<script::Script> = executables
        .executables
        .iter()
        .map(script::Script::new)
        .filter_map(Result::ok)
        .collect();

    debug!("Found the following scripts {:#?}", scripts);

    // Command line rules ok?
    if args.fuzzy {
        fuzzy(&scripts, args.number + 1)
    } else if args.list || args.script.is_some() {
        // If the user is specifying --list OR if there's some value for script.
        // Any value there is implicitly take as --list.
        list(executables, args)
    } else if args.tui {
        tui(&scripts)
    } else {
        // Neither requested, so fall back on the configuration
        match default_mode.as_str() {
            "fuzzy" => fuzzy(&scripts, args.number + 1),
            "list" => list(executables, args),
            _ => panic!("No default mode set! Has there been a problem creating the config file?"),
        }
    }
}

// Runs lk in 'tui' mode.
fn tui(scripts: &[script::Script]) -> Result<()> {
    println!("Running lk in tui mode");
    tui::list_example::show(scripts);
    Ok(())
}

/// Runs lk in 'fuzzy' mode.
fn fuzzy(scripts: &[script::Script], lines_to_show: i8) -> Result<()> {
    let result = FuzzyFinder::find(scripts_to_item(scripts), lines_to_show).unwrap();
    if let Some(function) = result {
        // We're going to write the equivalent lk command to the shell's history
        // file, so the user can easily re-run it.
        let history = UserShell::new();
        match history {
            Some(history) => {
                let lk_command = format!("lk {} {}", function.0.file_name(), function.1.name,);
                history.add_command(lk_command)?;
            }
            None => {
                log::warn!("Unable to write to history file because we couldn't figure out what shell you're using");
            }
        }
        // Finally we execute the function using a temporary bash file.
        BashFile::run(function.0.to_owned(), function.1.to_owned(), [].to_vec())?;
    }
    Ok(())
}

/// Runs lk in 'list' mode.
fn list(executables: Executables, args: Cli) -> Result<()> {
    // Did the user request a script?
    if let Some(script) = args.script {
        // Is it a script that exists on disk?
        if let Some(executable) = executables.get(&script) {
            // Yay, confirmed script
            let script = script::Script::new(executable)?;
            // Did the user pass a function?
            if let Some(function) = args.function {
                // Is it a function that exists in the script we found?
                if let Some(function) = script.get(&function) {
                    // Finally we execute the function using a temporary bash file.
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

/// Convert the scripts we find to the 'item' required for fuzzy find.
fn scripts_to_item(scripts: &[script::Script]) -> Vec<Item<(&script::Script, &Function)>> {
    let mut fuzzy_functions: Vec<Item<(&script::Script, &Function)>> = Vec::new();
    scripts.iter().for_each(|script| {
        script.functions.iter().for_each(|function| {
            fuzzy_functions.push(Item::new(
                format!("{} - {}", script.path(), function.name),
                (script, function),
            ))
        })
    });
    fuzzy_functions
}
