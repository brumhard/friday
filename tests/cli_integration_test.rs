use std::{error::Error, fs};

use assert_cmd::Command;
use tempfile::TempDir;

fn friday_cli() -> Command {
    Command::cargo_bin("friday").unwrap()
}

// NOTE: it would also be possible to move the code in main.rs to lib.rs and
// only call run in the main.rs. Then, run could be tested here without the need
// to really run a binary here.
// This is documented here: https://doc.rust-lang.org/book/ch11-03-test-organization.html#integration-tests-for-binary-crates.
//
// Anyways, it's nice to test the whole interface of the CLI as well e2e.
#[test]
fn it_prints_help_on_empty_action() {
    let tmp_dir = TempDir::new().unwrap();
    let cmd = friday_cli().env("FRIDAY_FILE", tmp_dir.path().join("sth")).assert().success();
    let output = cmd.get_output();
    let output_str = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(
        output_str.contains("manage stuff to do on fridays"),
        "expected '{}' to contain 'manage stuff to do on fridays'",
        output_str
    )
}

#[test]
fn it_adds_to_file() -> Result<(), Box<dyn Error>> {
    let tmp_dir = TempDir::new()?;
    let file_path = tmp_dir.path().join("friday.md");
    let to_add = "something that should be added";
    friday_cli().arg("add").arg(to_add).env("FRIDAY_FILE", &file_path).assert().success();

    let content = fs::read_to_string(&file_path)?;
    assert!(content.contains(to_add), "expected '{}' to contain the added string", content);
    assert!(content.contains("# It's friday my dudes"), "expected file content to contain header",);
    // assert that file contains string
    Ok(())
}
