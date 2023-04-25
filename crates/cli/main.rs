#![warn(clippy::pedantic)]

mod config;
mod error;

use config::{Action, Config};
use error::{Error, Result};

use colored::Colorize;
use friday_core::{DefaultManager, FileBacked, Manager, Section};
use std::{
    collections::HashMap,
    env,
    io::{self},
    process::{exit, Command},
};

const DEFAULT_EDITOR: &str = "vi";

// see here: https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html
fn main() {
    // see https://github.com/rust-lang/log
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("INFO"));

    let cfg = Config::build(env::args(), &env::vars().collect()).unwrap_or_else(|e| {
        eprintln!("failed to load options: {e}");
        exit(1)
    });

    run(cfg).unwrap_or_else(|e| {
        eprint!("error during run: {e}");
        exit(1)
    });
}

fn run(cfg: Config) -> Result<()> {
    log::debug!("running with config '{:?}'", cfg);
    let repo = FileBacked::new(&cfg.file)?;
    let manager = DefaultManager::new(repo);

    match cfg.action {
        Action::Add => add(manager, cfg.input.unwrap_or_default().as_str()),
        Action::Show => show(manager),
        Action::Edit => edit_file(&cfg.file),
        Action::Help => print_help(),
    }
}

fn add(manager: impl Manager, input: &str) -> Result<()> {
    manager.add(input, None)?;
    Ok(())
}

fn edit_file(path: &str) -> Result<()> {
    let mut editor = env::var("EDITOR").unwrap_or_default();
    // also handles case where EDITOR is set to "" explictly
    if editor.trim().is_empty() {
        log::debug!("resetting EDITOR to default");
        editor = DEFAULT_EDITOR.to_string();
    }

    // in case the editor env var contains args like e.g. `code -w`
    // it's necessary to split it up into program and args.
    let mut editor_parts = editor.split_whitespace();
    // since editor is checked before, it will have at least one part
    let mut cmd = Command::new(editor_parts.next().unwrap());
    // use rest of parts as args
    cmd.args(editor_parts);
    cmd.arg(path);

    log::debug!("running command: {:?}", cmd);

    cmd.status().map_err(|e| match e {
        ref e if e.kind() == io::ErrorKind::NotFound => {
            Error::InvalidArgument(format!("EDITOR {editor} could not be found"))
        }
        e => Error::from(e),
    })?;
    Ok(())
}

#[allow(clippy::unnecessary_wraps)] // easier to use in run
fn show(manager: impl Manager) -> Result<()> {
    let sections = manager.sections()?;
    for (section, tasks) in sections {
        let section_header = format!("## {section}").cyan();
        println!("{section_header}");

        for task in tasks {
            println!("- {task}",);
        }

        println!();
    }
    Ok(())
}

#[allow(clippy::unnecessary_wraps)] // easier to use in run
fn print_help() -> Result<()> {
    println!(
        "\
This binary let's you manage stuff to do on fridays.

The following commands are available:
    help            -> Print this help text.
    add <string>    -> Add a string to the end of the file.
    show            -> Show the contents of the file.

The location of the file that should be used can be configured
globally using the `FRIDAY_FILE` env var.
"
    );
    Ok(())
}
