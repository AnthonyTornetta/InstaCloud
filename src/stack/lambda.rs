use std::collections::HashMap;

use super::{iam::role::Role, Shared};

pub enum LambdaRuntime {
    NodeJs20,
}

pub struct LambdaFunction {
    pub role: Shared<Role>,
    pub runtime: LambdaRuntime,
    pub file_path: String,
    pub environment_variables: HashMap<String, String>,
}
