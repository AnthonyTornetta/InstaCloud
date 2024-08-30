use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    hash::{DefaultHasher, Hash, Hasher},
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

#[derive(Debug, Default)]
struct PathNode {
    children: HashMap<String, PathNode>,
}

impl PathNode {
    fn new() -> Self {
        Self::default()
    }

    // Add a path to the tree
    fn add_path(&mut self, path: &str) {
        let mut current_node = self;

        // Split the path into segments and iterate over them
        for segment in path.split('/') {
            // Insert the segment into the current node's children if it doesn't exist
            current_node = current_node
                .children
                .entry(segment.to_string())
                .or_insert_with(|| Default::default())
        }
    }
}

fn recurse(route_tree: &PathNode, route_so_far: &str) {
    for (subroute, children) in &route_tree.children {
        let route_here = if route_so_far.is_empty() {
            subroute.to_owned()
        } else {
            format!("{route_so_far}/{subroute}")
        };

        let mut resource_path = fs::read_to_string("terraform/lambda_test/resource_path.tf")
            .expect("Cannot load resource_path.tf");

        let mut hasher = DefaultHasher::default();
        route_here.hash(&mut hasher);
        let hash_here = hasher.finish();

        println!("{route_here}");

        let parent_id = if route_so_far.is_empty() {
            "aws_api_gateway_rest_api.api_gateway.root_resource_id".to_owned()
        } else {
            let mut hasher = DefaultHasher::default();
            route_so_far.hash(&mut hasher);
            let hash_so_far = hasher.finish();
            format!("aws_api_gateway_resource.api_resource_{hash_so_far}.id")
        };

        resource_path = resource_path
            .replace("{resource_path_hash}", &format!("{hash_here}"))
            .replace("{resource_path}", &subroute)
            .replace("{parent_id}", &parent_id);

        fs::write(
            &format!("terraform/generated/api/resource_path_{hash_here}.tf"),
            resource_path,
        )
        .expect("Failed to write resource_path.tf");

        recurse(children, &route_here);
    }
}

pub(super) fn setup_api_dir(api_defs: &[ApiDefinition]) {
    let routes = api_defs
        .iter()
        .map(|x| x.route.as_str())
        .collect::<HashSet<&str>>();

    let mut route_tree = PathNode::new();

    for route in routes.iter() {
        route_tree.add_path(&route);
    }

    recurse(&route_tree, "");
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

    let mut hasher = DefaultHasher::default();
    tf_vars.resource_path.hash(&mut hasher);
    let resource_hash = hasher.finish();

    println!("HASHING: {}", tf_vars.resource_path);

    tf_file = tf_file
        .replace("{function_name}", &tf_vars.name)
        .replace("{runtime}", &tf_vars.runtime)
        .replace("{http_method}", &tf_vars.method)
        .replace("{depends_on}", depends_on)
        .replace("{resource_path}", &tf_vars.resource_path)
        .replace("{resource_path_hash}", &format!("{resource_hash}"));

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

    let fs_conents = fs::read(file_path)
        .unwrap_or_else(|_| panic!("Unable to read file {file_path} - is it there?"));

    zw.write_all(&fs_conents)?;

    zw.finish()?;

    Ok(())
}
