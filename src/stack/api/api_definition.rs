use super::endpoint::HttpMethod;

pub struct ApiDefinition {
    pub name: String,
    pub method: HttpMethod,
    pub route: String,
    pub file: String,
    pub root: String,
    pub read: Vec<String>,
    pub write: Vec<String>,
}
