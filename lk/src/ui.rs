use crate::{
    executables::Executables,
    script::{Function, Script},
};
use pastel_colours::{BLUE_FG, DARK_BLUE_BG, GREEN_FG, RED_FG, RESET_BG, RESET_FG};

pub fn print_root_header() {
    println!("{DARK_BLUE_BG}lk: ./{RESET_BG}");
}

pub fn print_script_header(script: &Script) {
    let script_path = script
        .path
        .to_owned()
        .into_os_string()
        .into_string()
        .unwrap();
    println!("{DARK_BLUE_BG}lk: {script_path}{RESET_BG}");
}

pub fn print_complete_header(script: &Script, function: &Function, params: &[String]) {
    println!(
        "{DARK_BLUE_BG}lk: {} -> {} ({}){RESET_BG}",
        script.path.as_os_str().to_string_lossy(),
        function.name,
        params.join(" ")
    );
}

pub fn print_no_functions_in_script_help() {
    println!("Could not find any functions! Why not add some. They look like this:");
    let example_function = r#"# Some great comment
# More insightful and fascinating insights into bash scripting
blow_mind() {
    echo "OMG so cool"
} "#;
    println!("{GREEN_FG}{example_function}{RESET_FG}");
}

pub fn print_bad_script_name(script: &str, executables: Executables) {
    println!("{RED_FG}Didn't find a script with name {BLUE_FG}{script}!{RESET_FG}\n");
    executables.pretty_print();
}

pub fn print_bad_function_name(script: &Script, function: &str) {
    println!("{RED_FG}Didn't find a function with name {BLUE_FG}{function}{RESET_FG}!\n");
    script.pretty_print();
}
