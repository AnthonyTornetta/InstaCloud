use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Any,
    Get,
    Post,
    Put,
    Delete,
    Options,
    Patch,
    Head,
}

impl From<HttpMethod> for String {
    fn from(value: HttpMethod) -> Self {
        match value {
            HttpMethod::Any => "ANY",
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Put => "PUT",
            HttpMethod::Options => "OPTIONS",
            HttpMethod::Head => "HEAD",
        }
        .to_owned()
    }
}

#[derive(Error, Debug)]
pub enum HttpMethodError {
    #[error("Invalid HttpMethod string value")]
    InvalidStringValue,
}

impl<'a> TryFrom<&'a str> for HttpMethod {
    type Error = HttpMethodError;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value.to_uppercase().as_str() {
            "ANY" => Ok(HttpMethod::Any),
            "GET" => Ok(HttpMethod::Get),
            "POST" => Ok(HttpMethod::Post),
            "PATCH" => Ok(HttpMethod::Patch),
            "DELETE" => Ok(HttpMethod::Delete),
            "PUT" => Ok(HttpMethod::Put),
            "OPTIONS" => Ok(HttpMethod::Options),
            "HEAD" => Ok(HttpMethod::Head),
            _ => Err(HttpMethodError::InvalidStringValue),
        }
    }
}

pub struct ApiEndpoint {
    name: String,
    method: HttpMethod,
    route: String,
    file: String,
    read: Vec<String>,
    write: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiDefinition {
    pub name: String,
    pub method: HttpMethod,
    pub route: String,
    pub file: String,
    pub root: String,
    pub read: Vec<String>,
    pub write: Vec<String>,
}

#[derive(Error, Debug)]
pub enum ApiDefinitionError {
    #[error("Invalid API method: {0}")]
    InvalidMethod(String),
    #[error("API definitions contain duplicate name")]
    DuplicateNameFound,
    #[error("Not a valid API path")]
    InvalidPath,
    #[error("Cannot read TOML file: {0}")]
    CannotReadTomlFile(std::io::Error),
    #[error("Cannot parse TOML file: {0}")]
    CannotParseTomlFile(toml::de::Error),
}
