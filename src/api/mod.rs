use std::{collections::HashMap, fs};

use crate::config::api::api_config::{ApiDefinition, HttpMethod};

struct TfVars {
    /// AWS region
    pub region: String,
    /// Lambda function name
    pub name: String,
    /// Script runtime (e.g. nodejs20.x)
    pub runtime: String,
    /// Environment variables sent to the lambda script
    pub environment_vars: HashMap<String, String>,
    /// Gateway name
    pub gateway_name: String,
    /// For the environment (e.g. "prod")
    pub gateway_stage: String,
    /// "GET"/"POST"/etc
    pub method: String,
}

pub(super) fn setup_api_dir() {
    fs::copy(
        "terraform/lambda_test/variables.tf",
        "terraform/generated/api/variables.tf",
    )
    .expect("Failed to setup API dir");
}

pub(super) fn process_api_definition(api_def: &ApiDefinition) {
    let tf_vars = TfVars {
        environment_vars: Default::default(),
        name: api_def.name.clone(),
        method: match api_def.method {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            _ => todo!(),
        }
        .to_owned(),
        region: "us-east-1".into(),
        runtime: "nodejs20.x".into(),
        gateway_name: "lambda_gateway".into(),
        gateway_stage: "prod".into(),
    };

    fs::copy(
        "terraform/lambda_test/main.tf",
        &format!("terraform/generated/api/{}.tf", api_def.name),
    )
    .expect("Unable to copy file!");

    println!("{api_def:?}");
}
