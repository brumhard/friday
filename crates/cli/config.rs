use std::{collections::HashMap, convert, fmt, str};

use crate::Error;

const DEFAULT_FILE: &str = "friday.md";

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    Show,
    Add,
    Help,
    Edit,
}

impl convert::TryFrom<&str> for Action {
    type Error = Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "add" => Ok(Action::Add),
            "show" => Ok(Action::Show),
            "edit" => Ok(Action::Edit),
            "help" | "" => Ok(Action::Help),
            cmd => Err(Error::InvalidCommand(cmd.to_string())),
        }
    }
}

impl str::FromStr for Action {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Action::try_from(s)
    }
}

impl fmt::Display for Action {
    // This uses the autogenerated debug trait from `#[derive(Debug)]`
    // to display the enum name.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    pub action: Action,
    pub input: Option<String>,
    pub file: String,
}

impl Config {
    // TODO: add doc test here: https://doc.rust-lang.org/book/ch14-02-publishing-to-crates-io.html#documentation-comments-as-tests
    pub fn build(
        mut args: impl Iterator<Item = String>,
        env_vars: &HashMap<String, String>,
    ) -> Result<Config, Error> {
        // first item is binary name
        args.next();

        let action: Action = args.next().unwrap_or_default().as_str().parse()?;

        let input = args.reduce(|mut iter, arg| {
            iter += &format!(" {arg}");
            iter
        });

        let mut file = env_vars.get("FRIDAY_FILE").cloned().unwrap_or_default();
        if file.trim().is_empty() {
            let home = dirs::home_dir().ok_or_else(|| {
                Error::InvalidArgument("failed to get users home dir".to_string())
            })?;
            // since home dir is always a valid path and `DEFAULT_FILE` also
            // there won't be any loss when converting.
            file = home.join(DEFAULT_FILE).to_string_lossy().to_string();
        }

        Ok(Config {
            action,
            input,
            file,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Result;

    // This is a way to create table driven tests.
    // It's definitely overkill here but whatever.
    // https://users.rust-lang.org/t/table-driven-aka-data-driven-testing/3848
    macro_rules! test_config_input {
        ($name:ident, $($in:expr),+ => $out:expr) => {
            #[test]
            fn $name() -> Result<()> {
                let args: Vec<String> = vec!["binary", "add", $($in, )*]
                    .iter()
                    .map(|&s| s.to_string())
                    .collect();
                let cfg = Config::build(args.into_iter(), &HashMap::new())?;
                assert!(cfg.input.is_some());
                let input = cfg.input.unwrap();
                assert_eq!(input, $out.to_string());
                Ok(())
            }
        };
    }

    test_config_input!(
        multiple_inputs,
        "these", "args", "are", "joined", "together" => "these args are joined together"
    );
    test_config_input!(
        single_input,
        "this is a single string argument" => "this is a single string argument"
    );

    #[test]
    fn config_is_created() -> Result<()> {
        let friday_file = "testing".to_string();

        let args = vec!["binary".to_string(), "show".to_string()];
        let env_vars = HashMap::from([("FRIDAY_FILE".to_string(), friday_file.clone())]);
        let cfg = Config::build(args.into_iter(), &env_vars)?;
        assert_eq!(
            cfg,
            Config {
                action: Action::Show,
                file: friday_file,
                input: None
            }
        );
        Ok(())
    }

    #[test]
    fn config_fails_for_invalid_enum() {
        let args = vec!["binary".to_string(), "invalid".to_string()];
        let cfg = Config::build(args.into_iter(), &HashMap::new());
        assert!(cfg.is_err());
    }
}