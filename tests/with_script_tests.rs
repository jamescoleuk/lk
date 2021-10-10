/// Tests lk's behaviour when the user asks for a script
use std::process::{Command, Output};

fn run_with_function(script_name: &str) -> Output {
    Command::new("cargo")
        .arg("run")
        .arg(script_name)
        .output()
        .expect("failed to execute process")
}

#[test]
fn test_no_function() {
    // When...
    let output = run_with_function("script.sh");
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Then...
    assert_eq!(output.status.success(), true);
    assert_eq!(stdout.contains("script.sh"), true);
    assert_eq!(stdout.contains("First line of file header comment"), true);
    assert_eq!(stdout.contains("Second line of file header comment"), true);
    assert_eq!(stdout.contains("some_function This function"), true);
    assert_eq!(stdout.contains("This function is very clever and"), true);
    assert_eq!(stdout.contains("And here is some more detailed"), true);
    assert_eq!(stdout.contains("some_function"), true);
    assert_eq!(stdout.contains("another_function"), true);
    assert_eq!(stdout.contains("yet_more_functions"), true);
}

#[test]
fn test_with_empty_script() {
    // When...
    let output = run_with_function("empty_script.sh");
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Then...
    assert_eq!(output.status.success(), true);
    assert_eq!(stdout.contains("Could not find any functions!"), true);
}
