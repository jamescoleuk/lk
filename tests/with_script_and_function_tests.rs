// /// Tests lk's behaviour when the user asks for a script and a function
// use std::process::{Command, Output};

// fn run(script_name: &str, function_name: &str, args: &[&str]) -> Output {
//     Command::new("cargo")
//         .arg("run")
//         .arg(script_name)
//         .arg(function_name)
//         .args(args)
//         .output()
//         .expect("failed to execute process")
// }

// #[test]
// fn test_with_function() {
//     // When...
//     let output = run("script.sh", "another_function", &[]);
//     let stdout = String::from_utf8(output.stdout).unwrap();

//     // Then...
//     assert_eq!(output.status.success(), true);
//     println!("{}", stdout);
//     // The function is executed.
//     assert_eq!(stdout.contains("hello from another function"), true);
// }

// #[test]
// fn test_with_bad_function_name() {
//     // When...
//     let output = run("script.sh", "bad_function_name", &[]);
//     let stdout = String::from_utf8(output.stdout).unwrap();

//     // Then...
//     assert_eq!(output.status.success(), true);
//     assert_eq!(stdout.contains("Didn't find a function with name"), true);
// }

// #[test]
// fn test_function_params() {
//     // When...
//     let output = run("script.sh", "printing_function", &["hello", "person"]);
//     let stdout = String::from_utf8(output.stdout).unwrap();

//     // Then...
//     assert_eq!(output.status.success(), true);
//     assert_eq!(stdout.contains("You said hello person"), true);
// }

// #[test]
// fn bad_script_path() {
//     // When...
//     let output = run("bad_script_path.sh", "another_function", &[]);
//     let stdout = String::from_utf8(output.stdout).unwrap();

//     // Then...
//     assert_eq!(output.status.success(), true);
//     assert_eq!(stdout.contains("Didn't find a script with name"), true);
// }

// /// If this script isn't executed in its directory then it won't find 'file'
// /// and won't print the correct thing and the test will fail.
// #[test]
// fn executes_in_script_dir() {
//     // When...
//     let output = run("depends_on_file.sh", "depends_on_file", &[]);
//     let stdout = String::from_utf8(output.stdout).unwrap();

//     // Then...
//     assert_eq!(output.status.success(), true);
//     assert_eq!(stdout.contains("contents to print"), true);
// }
