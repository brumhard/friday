use friday_rust::{Command, Config, Error, Result};
use std::{
    env,
    fs::{self, File},
    io::Write,
    process::exit,
};

// see here: https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html
fn main() {
    // see https://github.com/rust-lang/log
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("INFO"));

    let cfg = Config::from_args(env::args()).unwrap_or_else(|e| {
        eprintln!("failed to load options: {e}");
        exit(1)
    });

    run(cfg).unwrap_or_else(|e| {
        eprint!("error during run: {e}");
        exit(1)
    })
}

fn run(cfg: Config) -> Result<()> {
    log::debug!("running with config '{:?}'", cfg);
    let input = cfg.input.unwrap_or_default();
    let path = cfg.file;

    use Command::*;
    match cfg.action {
        Add => add(&path, &input),
        Show => show_file(&path),
        Help => {
            print_help();
            Ok(())
        }
    }
}

fn add(path: &str, input: &str) -> Result<()> {
    if input.is_empty() {
        return Err(Error::InvalidArgument(
            "expected non-empty input".to_string(),
        ));
    }

    let mut file = File::options().append(true).create(true).open(path)?;
    file.write_all(input.as_bytes())?;
    file.write_all(b"\n")?;
    Ok(())
}

fn show_file(path: &str) -> Result<()> {
    let test = fs::read_to_string(path)?;
    println!("{test}");
    Ok(())
}

fn print_help() {
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
    )
}
