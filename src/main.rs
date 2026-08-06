mod bash_file;
mod executables;
mod script;
mod shells;
mod tui;
mod ui;

use anyhow::{Context, Result, bail};
use bash_file::BashFile;
use clap::Parser;
use config::{Config, File};
use executables::Executables;
use log::{LevelFilter, debug, info};
use script::{Function, Script};
use shells::UserShell;
use spinners::{Spinner, Spinners};
use std::path::{Path, PathBuf};

/// Use lk to explore and execute scripts in your current directory,
/// and in its sub-directories. lk offers two options: 'tui' or 'list'.
/// 'tui' shows a full-screen UI with a fuzzy search. 'list' lets you
/// explore your scripts and their functions in a hierarchical way.
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// List available scripts and functions.
    #[arg(long, short)]
    list: bool,

    /// Show a full screen UI with lots of details.
    #[arg(long, short)]
    tui: bool,

    /// Optional: the name of a script to explore or use.
    script: Option<String>,

    /// Optional: the name of the function to run.
    function: Option<String>,

    /// Optional: params for the function. We're not processing them yet
    /// (e.g. validating) but they need to be permitted as a param to lk.
    #[arg(trailing_var_arg = true)]
    params: Vec<String>,

    /// Optional: paths to include in the search, as a UNIX glob pattern.
    #[arg(long, short)]
    includes: Vec<String>,

    /// Optional: paths to exclude in the search, as a UNIX glob pattern.
    #[arg(long, short)]
    excludes: Vec<String>,
}

/// The directory where lk keeps its configuration and log file.
/// `LK_CONFIG_DIR` overrides the default of `~/.config/lk`, which is
/// useful for testing and for anyone who wants their config elsewhere.
fn lk_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("LK_CONFIG_DIR") {
        return PathBuf::from(dir);
    }
    match dirs::home_dir() {
        Some(home_dir) => home_dir.join(".config").join("lk"),
        // If we don't have access to the home dir for some reason then just use a temp dir.
        None => {
            println!("Unable to access your home directory. Using a temporary directory instead.");
            std::env::temp_dir().join("lk")
        }
    }
}

fn configure_logging(lk_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(lk_dir)
        .with_context(|| format!("Unable to create lk's config directory at {lk_dir:?}"))?;
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(lk_dir.join("lk.log"))?;
    simplelog::WriteLogger::init(
        LevelFilter::Info,
        simplelog::ConfigBuilder::new()
            .set_time_format_rfc3339()
            .build(),
        log_file,
    )?;
    Ok(())
}

fn main() -> Result<()> {
    let lk_dir = lk_dir();
    configure_logging(&lk_dir)?;

    info!("Starting lk...");

    let args = Cli::parse();

    let mut sp = Spinner::new(Spinners::Line, String::new());

    // Set configuration defaults, then load the user config followed by a workspace
    // config if they exist. Configurations in later files override earlier ones.
    // However, command line configuration overrides these.
    let config = Config::builder()
        .set_default("default_mode", "tui")?
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
            ],
        )?
        .add_source(File::from(lk_dir.join("lk.toml")).required(false))
        .add_source(File::from(Path::new(".").join("lk.toml")).required(false))
        .build()?;

    // Merge the command line includes and excludes with the config file includes and excludes
    let includes: Vec<String> = args
        .includes
        .iter()
        .cloned()
        .chain(config.get::<Vec<String>>("includes")?)
        .collect();

    let excludes: Vec<String> = args
        .excludes
        .iter()
        .cloned()
        .chain(config.get::<Vec<String>>("excludes")?)
        .collect();

    let default_mode = config.get::<String>("default_mode")?;

    info!("Using default_mode {default_mode:?}, includes {includes:?} and excludes {excludes:?}");

    // What executable scripts are available in the configuration directory?
    let executables = Executables::new(&includes, &excludes)?;

    sp.stop();

    // What functions do these executables contain?
    let scripts: Vec<Script> = executables
        .executables
        .iter()
        .map(Script::new)
        .filter_map(Result::ok)
        .collect();

    debug!("Found the following scripts {scripts:#?}");

    // Command line rules ok?
    if args.list || args.script.is_some() {
        // If the user is specifying --list OR if there's some value for script.
        // Any value there is implicitly taken as --list.
        list(executables, args)
    } else if args.tui {
        tui(&scripts)
    } else {
        // Neither requested, so fall back on the configuration
        match default_mode.as_str() {
            "list" => list(executables, args),
            "tui" => tui(&scripts),
            // The fuzzy mode was removed in 0.7.0. Its search lives on in
            // the TUI, so send anyone with an old config there.
            "fuzzy" => {
                println!(
                    "The fuzzy mode was removed in lk 0.7.0 — its search lives on in the TUI."
                );
                println!("Please update default_mode in your lk.toml.");
                tui(&scripts)
            }
            other => {
                bail!("Unknown default_mode {other:?} in lk.toml. Expected \"tui\" or \"list\".")
            }
        }
    }
}

/// Executes the selected function, first writing the equivalent lk command
/// to the shell's history file so the user can easily re-run it.
fn execute(script: &Script, function: &Function) -> Result<()> {
    match UserShell::new() {
        Some(history) => {
            let lk_command = format!("lk {} {}", script.file_name(), function.name);
            history.add_command(&lk_command)?;
        }
        None => {
            log::warn!(
                "Unable to write to history file because we couldn't figure out what shell you're using"
            );
        }
    }
    // Finally we execute the function using a temporary bash file.
    BashFile::run(script.to_owned(), function.to_owned(), Vec::new())
}

/// Runs lk in 'tui' mode.
fn tui(scripts: &[Script]) -> Result<()> {
    if let Some((script, function)) = tui::list::find(scripts)? {
        execute(&script, &function)?;
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
            let script = Script::new(executable)?;
            // Did the user pass a function?
            if let Some(function) = args.function {
                // Is it a function that exists in the script we found?
                if let Some(function) = script.get(&function) {
                    // Finally we execute the function using a temporary bash file.
                    BashFile::run(script.to_owned(), function.to_owned(), args.params)?;
                } else {
                    ui::print_bad_function_name(&script, &function);
                }
            } else {
                // No function, display a list of what's available
                script.pretty_print();
            }
        } else {
            ui::print_bad_script_name(&script, &executables);
        }
    } else {
        // No executable, display a list of what's available
        executables.pretty_print();
    }
    Ok(())
}
