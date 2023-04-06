use core::fmt;
use std::default;
use std::str;

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum Section {
    Dump,
    Custom(String),
}

impl default::Default for Section {
    fn default() -> Self {
        Self::Dump
    }
}

impl fmt::Display for Section {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dump => write!(f, "dump"),
            Self::Custom(x) => write!(f, "{x}"),
        }
    }
}

impl str::FromStr for Section {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "" | "dump" => Ok(Self::Dump),
            any => Ok(Self::Custom(any.to_string())),
        }
    }
}
