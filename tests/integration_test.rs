use std::{error::Error, fs};

use assert_cmd::Command;

fn friday() -> Command {
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
}

#[test]
fn it_prints_help_on_empty_action() {
    let cmd = friday().assert();
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
    let tmp_dir = tempdir::TempDir::new("testing")?;
    let file_path = tmp_dir.path().join("friday.md");
    let to_add = "something that should be added";
    friday()
        .arg("add")
        .arg(to_add)
        // TODO: set temp file
        .env("FRIDAY_FILE", file_path.to_string_lossy().to_string())
        .assert()
        .success();

    let content = fs::read_to_string(file_path)?;
    assert!(
        content.contains(to_add),
        "expected '{}' to contain the added string",
        content
    );
    assert!(
        content.contains("# It's friday my dudes"),
        "expected file content to contain header",
    );
    // assert that file contains string
    Ok(())
}
