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
    pub resource_path: String,
    /// "GET"/"POST"/etc
    pub method: String,
}

pub(super) fn setup_api_dir() {
    fs::create_dir_all("terraform/generated/api/").expect("Unable to create API dir.");

    fs::copy(
        "terraform/lambda_test/variables.tf",
        "terraform/generated/api/variables.tf",
    )
    .expect("Failed to setup API dir");

    fs::copy(
        "terraform/lambda_test/main.tf",
        "terraform/generated/api/main.tf",
    )
    .expect("Unable to copy file!");
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
        resource_path: api_def.route.to_owned(),
    };

    let mut tf_file =
        fs::read_to_string("terraform/lambda_test/api.tf").expect("Failed to read TF file.");

    tf_file = tf_file
        .replace("{function_name}", &tf_vars.name)
        .replace("{runtime}", &tf_vars.runtime)
        .replace("{http_method}", &tf_vars.method)
        .replace("{resource_path}", &tf_vars.resource_path);

    fs::write(
        &format!("terraform/generated/api/{}.tf", tf_vars.name),
        tf_file,
    )
    .expect("Failed to write");
}
