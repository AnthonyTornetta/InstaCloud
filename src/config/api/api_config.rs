use serde::{Deserialize, Serialize};

use crate::cloud::api::HttpMethod;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ApiConfig {
    pub name: String,
    pub root: String,
    pub domain: String,
    pub prefix: Option<String>,
    pub endpoints: Vec<ApiEndpoint>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ApiEndpoint {
    pub name: String,
    pub method: HttpMethod,
    pub route: String,
    pub file: String,
    pub read: Vec<String>,
    pub write: Vec<String>,
}
