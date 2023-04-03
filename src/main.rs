use std::{
    env,
    fs::{self, File},
    io::Write,
    process::exit,
};

use friday_rust::{Config, Error, Result};

// see here: https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html
fn main() {
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
    let input = cfg.input.unwrap_or_default();
    let path = cfg.file;
    match cfg.action.as_str() {
        "add" => {
            if input.is_empty() {
                return Err(Error::InvalidArgument(
                    "expected non-empty input".to_string(),
                ));
            }
            add(&path, &input)
        }
        "show" => show_file(&path),
        cmd => Err(Error::InvalidCommand(cmd.to_string())),
    }
}

fn add(path: &str, input: &str) -> Result<()> {
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
