use std::{collections::HashMap, fmt, str};

use serde::{de::DeserializeOwned, Serialize};

use crate::Error;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
}

impl str::FromStr for Method {
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "get" => Ok(Self::GET),
            "post" => Ok(Self::POST),
            "put" => Ok(Self::PUT),
            "patch" => Ok(Self::PATCH),
            "delete" => Ok(Self::DELETE),
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
        write!(f, "{self:?}")
    }
}

#[derive(Clone)]
pub struct Request {
    pub method: Method,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub raw_body: Option<String>,
}

impl Request {
    pub fn json<D: DeserializeOwned>(&self) -> Option<D> {
        if let Some(body) = &self.raw_body {
            let d = serde_json::from_str(body);
            match d {
                Ok(d) => return Some(d),
                Err(_) => return None,
            }
        }
        None
    }
}

pub struct Response {
    pub status: u16,
    pub body: Option<String>,
}

impl Response {
    pub fn json<S: Serialize>(mut status: u16, body: &S) -> Response {
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