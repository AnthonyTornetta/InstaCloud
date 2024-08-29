use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::{Cursor, Write},
};

use zip::{write::SimpleFileOptions, ZipWriter};

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

pub(super) fn setup_api_dir_root() {
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

pub(super) fn setup_api_dir(api_defs: &[ApiDefinition]) {
    let routes = api_defs
        .iter()
        .map(|x| x.route.as_str())
        .collect::<HashSet<&str>>();

    for route in routes {
        let mut resource_path = fs::read_to_string("terraform/lambda_test/resource_path.tf")
            .expect("Cannot load resource_path.tf");

        resource_path = resource_path.replace("{resource_path}", route);

        fs::write(
            &format!("terraform/generated/api/resource_path_{route}.tf"),
            resource_path,
        )
        .expect("Failed to write resource_path.tf");
    }
}

pub(super) fn process_api_definition(api_def: &ApiDefinition, depends_on: &str) {
    let tf_vars = TfVars {
        environment_vars: Default::default(),
        name: api_def.name.clone(),
        method: match api_def.method {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Any => "ANY",
            HttpMethod::Put => "PUT",
            HttpMethod::Head => "HEAD",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Options => "OPTIONS",
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
        .replace("{depends_on}", depends_on)
        .replace("{resource_path}", &tf_vars.resource_path);

    fs::write(
        &format!("terraform/generated/api/{}.tf", tf_vars.name),
        tf_file,
    )
    .expect("Failed to write");

    create_lambda_files(&format!("{}/{}", api_def.root, api_def.file), &tf_vars)
        .expect("Unable to create api files!");
}

fn create_lambda_files(file_path: &str, tf_vars: &TfVars) -> anyhow::Result<()> {
    let file_buf = File::create(&format!(
        "terraform/generated/api/lambda_function_{}.zip",
        tf_vars.name
    ))?;

    let mut zw = ZipWriter::new(file_buf);
    zw.start_file("index.js", SimpleFileOptions::default())?;

    let fs_conents = fs::read(file_path)?;

    zw.write_all(&fs_conents)?;

    zw.finish()?;

    Ok(())
}
