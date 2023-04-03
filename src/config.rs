use std::{env, path::Path, str::FromStr};

use crate::Error;

const FRIDAY_FILE: &str = "test";

pub enum Command {
    Show,
    Add,
    Help,
}

impl TryFrom<&str> for Command {
    type Error = Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "add" => Ok(Command::Add),
            "show" => Ok(Command::Show),
            "help" => Ok(Command::Help),
            cmd => Err(Error::InvalidArgument(cmd.to_string())),
        }
    }
}

impl FromStr for Command {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Command::try_from(s)
    }
}

pub struct Config {
    pub action: Command,
    pub input: Option<String>,
    pub file: String,
}

impl Config {
    pub fn from_args(mut args: impl Iterator<Item = String>) -> Result<Config, Error> {
        // first item is binary name
        args.next();

        let action: Command = args.next().unwrap_or_default().as_str().parse()?;

        let input = args.reduce(|mut iter, arg| {
            iter += &format!(" {arg}");
            iter
        });

        let file = env::var("FRIDAY_FILE").unwrap_or_else(|_| FRIDAY_FILE.to_string());
        if !Path::new(&file).is_file() {
            return Err(Error::InvalidArgument(
                "FRIDAY_FILE must point to a valid file".to_string(),
            ));
        };

        Ok(Config {
            action,
            input,
            file,
        })
    }
}

// TODO: add tests https://doc.rust-lang.org/stable/book/ch11-00-testing.html
