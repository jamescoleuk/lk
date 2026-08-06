//! Tests lk's behaviour when no script is named, i.e. the finding and
//! printing of executables.
mod common;

use common::{run_lk, stdout};

#[test]
fn finds_executables() {
    // When...
    let output = run_lk(&["--list"]);
    let stdout = stdout(&output);

    // Then...
    assert!(output.status.success());
    // The scripts should be present
    assert!(stdout.contains("script01.sh"));
    assert!(stdout.contains("script02.sh"));
    assert!(stdout.contains("script04.sh"));
}

#[test]
fn binaries_are_ignored() {
    // When...
    let output = run_lk(&["--list"]);
    let stdout = stdout(&output);

    // Then...
    assert!(output.status.success());
    // The binary should be ignored
    assert!(!stdout.contains("mkfifo"));
}

#[test]
fn must_have_executable_permissions() {
    // When...
    let output = run_lk(&["--list"]);
    let stdout = stdout(&output);

    // Then...
    assert!(output.status.success());
    // The script without executable permissions should be ignored
    assert!(!stdout.contains("script03.sh"));
}
