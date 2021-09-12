use std::io::{self, Write};
use std::process::Command;

#[test]
fn test_no_function() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("tests/script.sh")
        .output()
        .expect("failed to execute process");

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(output.status.success(), true);
    assert_eq!(stdout.contains("script.sh"), true);
    assert_eq!(stdout.contains("Usage"), true);
    assert_eq!(stdout.contains("This function is very clever and"), true);
    assert_eq!(stdout.contains("And here is some more detailed"), true);
    assert_eq!(stdout.contains("some_function"), true);
    assert_eq!(stdout.contains("another_function"), true);
    assert_eq!(stdout.contains("yet_more_functions"), true);
}

#[test]
fn test_with_function() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("tests/script.sh")
        .arg("another_function")
        .output()
        .expect("failed to execute process");
    // Should return a non-0 exit code, allowing bash to || "$@",
    // thereby running the script's function itself.
    assert_eq!(output.status.success(), false);
}
