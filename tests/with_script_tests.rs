/// Tests lk's behaviour when the user asks for a script
use std::process::Command;

#[test]
fn test_no_function() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("script.sh")
        .output()
        .expect("failed to execute process");

    let stdout = String::from_utf8(output.stdout).unwrap();
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
    let output = Command::new("cargo")
        .arg("run")
        .arg("empty_script.sh")
        .output()
        .expect("failed to execute process");
    assert_eq!(output.status.success(), true);
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.contains("Could not find any functions!"), true);
}
