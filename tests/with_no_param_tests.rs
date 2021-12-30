//TODO: Fix! Caused by 8bd7fd7a00d2b8c8238794f924ecedb2628577c8 and the previous config.
// Tests lk's behaviour when no params are passed, i.e. the finding and
// printing of executables.
// use std::process::{Command, Output};

// fn run_no_param() -> Output {
//     Command::new("cargo")
//         .arg("run")
//         .output()
//         .expect("failed to execute process")
// }

// #[test]
// fn finds_executables() {
//     // When...
//     let output = run_no_param();
//     let stdout = String::from_utf8(output.stdout).unwrap();

//     // Then...
//     assert_eq!(output.status.success(), true);
//     // The scripts should be present
//     assert_eq!(stdout.contains("script01.sh"), true);
//     assert_eq!(stdout.contains("script02.sh"), true);
//     assert_eq!(stdout.contains("script04.sh"), true);
// }

// #[test]
// fn binaries_are_ignored() {
//     // When...
//     let output = run_no_param();
//     let stdout = String::from_utf8(output.stdout).unwrap();

//     // Then...
//     assert_eq!(output.status.success(), true);
//     // The binary should be ignored
//     assert_eq!(stdout.contains("mkfifo"), false);
// }

// #[test]
// fn must_have_executable_permissions() {
//     // When...
//     let output = run_no_param();
//     let stdout = String::from_utf8(output.stdout).unwrap();

//     // Then...
//     assert_eq!(output.status.success(), true);
//     // The script without executable permissions should be ignored
//     assert_eq!(stdout.contains("script03.sh"), false);
// }
