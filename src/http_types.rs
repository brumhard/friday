use std::{collections::HashMap, fmt, str};

use serde::Serialize;

use crate::{Error};

#[derive(PartialEq, Eq, Debug)]
pub enum Method {
    GET,
    POST,
}

impl str::FromStr for Method {
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "GET" => Ok(Self::GET),
            "POST" => Ok(Self::POST),
            invalid => Err(Error::InvalidArgument(format!(
                "{invalid} is not an HTTP method"
            ))),
        }
    }
}

impl fmt::Display for Method {
    // This uses the autogenerated debug trait from `#[derive(Debug)]`
    // to display the enum name.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct Request {
    pub method: Method,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

pub struct Response {
    pub status: u16,
    pub body: Option<String>,
}

impl Response {
    pub fn new<S: Serialize>(mut status: u16, body: &S) -> Response {
        let body = serde_json::to_string(body).unwrap_or_else(|_| {
            status = 500;
            "serialization error".to_string()
        });
        Response {
            status,
            body: Some(body),
        }
    }
}
