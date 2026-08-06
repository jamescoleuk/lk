//! Tests lk's behaviour when the user asks for a script
mod common;

use common::{run_lk, stdout};

#[test]
fn test_no_function() {
    // When...
    let output = run_lk(&["script.sh"]);
    let stdout = stdout(&output);

    // Then...
    assert!(output.status.success());
    assert!(stdout.contains("script.sh"));
    assert!(stdout.contains("First line of file header comment"));
    assert!(stdout.contains("Second line of file header comment"));
    // There's an ANSI colour reset between the function name and its comment,
    // so we assert on the two parts separately.
    assert!(stdout.contains("This function is very clever and"));
    assert!(stdout.contains("some_function"));
    assert!(stdout.contains("another_function"));
    assert!(stdout.contains("yet_more_functions"));
    // Private functions are hidden
    assert!(!stdout.contains("_hidden_function"));
}

#[test]
fn test_with_empty_script() {
    // When...
    let output = run_lk(&["empty_script.sh"]);
    let stdout = stdout(&output);

    // Then...
    assert!(output.status.success());
    assert!(stdout.contains("Could not find any functions!"));
}
