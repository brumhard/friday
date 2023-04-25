use core::fmt;
use std::default;
use std::str;

use schemars::JsonSchema;
use serde_with::DeserializeFromStr;
use serde_with::SerializeDisplay;

#[derive(
    Ord,
    PartialOrd,
    PartialEq,
    Eq,
    Hash,
    Clone,
    Debug,
    DeserializeFromStr,
    SerializeDisplay,
    JsonSchema,
)]
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

impl From<Option<&str>> for Section {
    fn from(o: Option<&str>) -> Self {
        match o {
            Some(x) => x.parse().unwrap(),
            None => Section::Dump,
        }
    }
}
