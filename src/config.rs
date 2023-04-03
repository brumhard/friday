use std::{env, path::Path};

use crate::Error;

pub struct Config {
    pub action: String,
    pub input: Option<String>,
    pub file: String,
}

impl Config {
    pub fn from_args(mut args: impl Iterator<Item = String>) -> Result<Config, Error> {
        // first item is binary name
        args.next();

        let action = args.next().unwrap_or_default();

        let input = args.reduce(|mut iter, arg| {
            iter += &format!(" {arg}");
            iter
        });

        let file = env::var("FRIDAY_FILE").unwrap_or_else(|_| "./test".to_string());
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
