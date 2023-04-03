use friday_rust::{Action, Config, Error, Result};
use std::{
    env,
    fs::{self, File},
    io::{self, Write},
    process::{exit, Command},
};

const DEFAULT_EDITOR: &str = "vi";
const HEADER_BYTES: &[u8] = b"# It's friday my dudes\n";

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

    ensure_header(&path)?;

    use Action::*;
    match cfg.action {
        Add => add(&path, &input),
        Show => show_file(&path),
        Edit => edit_file(&path),
        Help => {
            print_help();
            Ok(())
        }
    }
}

fn edit_file(path: &str) -> Result<()> {
    let mut editor = env::var("EDITOR").unwrap_or_default();
    // also handles case where EDITOR is set to "" explictly
    if editor.trim().is_empty() {
        log::debug!("resetting EDITOR to default");
        editor = DEFAULT_EDITOR.to_string()
    }

    // in case the editor env var contains args like e.g. `code -w`
    // it's necessary to split it up into program and args.
    let mut editor_parts = editor.split(' ');
    // since editor is checked before, it will have at least one part
    let mut cmd = Command::new(editor_parts.next().unwrap());
    // use rest of parts as args
    cmd.args(editor_parts);
    cmd.arg(path);

    log::debug!("running command: {:?}", cmd);

    cmd.status().map_err(|e| match e {
        ref e if e.kind() == io::ErrorKind::NotFound => {
            Error::InvalidArgument(format!("EDITOR {editor} could not be found").to_string())
        }
        e => Error::from(e),
    })?;
    Ok(())
}

fn open_file(path: &str) -> Result<File> {
    let file = File::options()
        .create(true)
        .append(true)
        .read(true)
        .open(path)?;
    Ok(file)
}

fn ensure_header(path: &str) -> Result<()> {
    let mut file = open_file(path)?;
    if file.metadata()?.len() == 0 {
        file.write_all(HEADER_BYTES)?;
    }
    Ok(())
}

fn add(path: &str, input: &str) -> Result<()> {
    if input.trim().is_empty() {
        return Err(Error::InvalidArgument(
            "expected non-empty input".to_string(),
        ));
    }

    open_file(path)?.write_all(format!("\n- {input}").as_bytes())?;
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
