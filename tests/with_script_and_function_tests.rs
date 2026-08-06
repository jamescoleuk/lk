//! Tests lk's behaviour when the user asks for a script and a function
mod common;

use common::{run_lk, stdout};

#[test]
fn test_with_function() {
    // When...
    let output = run_lk(&["script.sh", "another_function"]);
    let stdout = stdout(&output);

    // Then...
    assert!(output.status.success());
    // The function is executed.
    assert!(stdout.contains("hello from another function"));
}

#[test]
fn test_with_bad_function_name() {
    // When...
    let output = run_lk(&["script.sh", "bad_function_name"]);
    let stdout = stdout(&output);

    // Then...
    assert!(output.status.success());
    assert!(stdout.contains("Didn't find a function with name"));
}

#[test]
fn test_function_params() {
    // When...
    let output = run_lk(&["script.sh", "printing_function", "hello", "person"]);
    let stdout = stdout(&output);

    // Then...
    assert!(output.status.success());
    assert!(stdout.contains("You said hello person"));
}

#[test]
fn bad_script_path() {
    // When...
    let output = run_lk(&["bad_script_path.sh", "another_function"]);
    let stdout = stdout(&output);

    // Then...
    assert!(output.status.success());
    assert!(stdout.contains("Didn't find a script with name"));
}

/// If this script isn't executed in its directory then it won't find 'file'
/// and won't print the correct thing and the test will fail.
#[test]
fn executes_in_script_dir() {
    // When...
    let output = run_lk(&["depends_on_file.sh", "depends_on_file"]);
    let stdout = stdout(&output);

    // Then...
    assert!(output.status.success());
    assert!(stdout.contains("contents to print"));
}
