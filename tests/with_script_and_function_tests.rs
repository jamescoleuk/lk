/// Tests lk's behaviour when the user asks for a script and a function
use std::process::Command;

#[test]
fn test_with_function() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("script.sh")
        .arg("another_function")
        .output()
        .expect("failed to execute process");
    assert_eq!(output.status.success(), true);
    let stdout = String::from_utf8(output.stdout).unwrap();
    println!("{}", stdout);
    // The function is executed.
    assert_eq!(stdout.contains("hello from another function"), true);
}

#[test]
fn test_with_bad_function_name() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("script.sh")
        .arg("bad_function_name")
        .output()
        .expect("failed to execute process");
    assert_eq!(output.status.success(), true);
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.contains("Didn't find a function with name"), true);
}

#[test]
fn test_function_params() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("script.sh")
        .arg("printing_function")
        .arg("hello")
        .arg("person")
        .output()
        .expect("failed to execute process");
    assert_eq!(output.status.success(), true);
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.contains("You said hello person"), true);
}

#[test]
fn bad_script_path() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("bad_script_path.sh")
        .arg("another_function")
        .output()
        .expect("failed to execute process");
    assert_eq!(output.status.success(), true);
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.contains("Didn't find a script with name"), true);
}
