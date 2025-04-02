use std::{
    collections::{HashMap, HashSet},
    hash::{DefaultHasher, Hash, Hasher},
};

use crate::stack::{
    tf::{Terraform, TerraformEntity, TfField, TfResource, TfVar},
    Shared,
};

use super::{
    deployment::GatewayDeployment, domain_name::Domain, endpoint::ApiEndpoint, stage::Stage,
};

#[derive(Default, Debug, Clone)]
pub struct ApiGateway {
    pub name: String,
    pub domain: Option<Shared<Domain>>,
    pub stage_name: String,

    pub endpoints: Vec<ApiEndpoint>,
}

#[derive(Debug, Clone)]
pub struct ResourcePath {
    parent_id: TfVar,
    resource_path: String,
}

impl ResourcePath {
    pub fn unique_id(&self) -> String {
        let mut hasher = DefaultHasher::default();
        self.resource_path.hash(&mut hasher);
        let hash = hasher.finish();
        hash.to_string()
    }

    pub fn tf_identifier(&self) -> String {
        format!("resource_path_{}", self.unique_id())
    }

    pub fn var(&self, field: impl Into<String>) -> TfVar {
        TfVar::Resource {
            resource_name: "aws_api_gateway_resource".into(),
            resource_identifier: self.tf_identifier(),
            field: field.into(),
        }
    }

    fn create_terraform(&self, gateway: &ApiGateway) -> Terraform {
        /*
                resource "aws_api_gateway_resource" "api_resource_{resource_path_hash}" {
          rest_api_id = aws_api_gateway_rest_api.api_gateway.id
          parent_id   = {parent_id} # aws_api_gateway_rest_api.api_gateway.root_resource_id
          path_part   = "{resource_path}"
        }
        */
        TfResource::new_resource("aws_api_gateway_resource", self.tf_identifier())
            .add_field(
                "rest_api_id",
                TfField::Variable(gateway.var_gateway_rest_api("id")),
            )
            .add_field("parent_id", TfField::Variable(self.parent_id.clone()))
            .add_field("path_part", TfField::String(self.resource_path.clone()))
            .create_terraform()
    }
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

fn recurse(
    gateway: &ApiGateway,
    route_tree: &PathNode,
    path_so_far: Option<&ResourcePath>,
    route_so_far: &str,
) -> Vec<ResourcePath> {
    let mut paths = vec![];

    for (subroute, children) in &route_tree.children {
        let route_here = if route_so_far.is_empty() {
            subroute.to_owned()
        } else {
            format!("{route_so_far}/{subroute}")
        };

        // let mut resource_path = fs::read_to_string("terraform/lambda_test/resource_path.tf")
        //     .expect("Cannot load resource_path.tf");

        let parent_id = if let Some(path_so_far) = path_so_far {
            path_so_far.var("id")
        } else {
            gateway.var_gateway_rest_api("root_resource_id")
        };
        // else {
        //     let mut hasher = DefaultHasher::default();
        //     route_so_far.hash(&mut hasher);
        //     let hash_so_far = hasher.finish();
        //     format!("aws_api_gateway_resource.api_resource_{hash_so_far}.id")
        // };
        let rp = ResourcePath {
            parent_id,
            resource_path: route_here.clone(),
        };

        // resource_path = resource_path
        //     .replace("{resource_path_hash}", &format!("{hash_here}"))
        //     .replace("{resource_path}", &subroute)
        //     .replace("{parent_id}", &parent_id);
        //
        // fs::write(
        //     &format!("{root_dir}/resource_path_{hash_here}.tf"),
        //     resource_path,
        // )
        // .expect("Failed to write resource_path.tf");

        paths.append(&mut recurse(gateway, children, Some(&rp), &route_here));

        paths.push(rp);
    }

    paths
}

impl ApiGateway {
    fn unique_key(&self) -> String {
        let mut hasher = DefaultHasher::default();
        self.name.hash(&mut hasher);
        self.stage_name.hash(&mut hasher);
        hasher.finish().to_string()
    }

    pub fn tf_identifier(&self) -> String {
        format!("gateway_{}", self.unique_key())
    }

    pub fn var_gateway_rest_api(&self, field: impl Into<String>) -> TfVar {
        TfVar::Resource {
            resource_name: "aws_api_gateway_rest_api".into(),
            resource_identifier: self.tf_identifier(),
            field: field.into(),
        }
    }

    pub fn var_gateway_resource(&self, field: impl Into<String>) -> TfVar {
        TfVar::Resource {
            resource_name: "aws_api_gateway_rest_api".into(),
            resource_identifier: self.tf_identifier(),
            field: field.into(),
        }
    }

    pub fn var_path_mapping_resource(&self, field: impl Into<String>) -> TfVar {
        TfVar::Resource {
            resource_name: "aws_api_gateway_base_path_mapping".into(),
            resource_identifier: self.tf_identifier(),
            field: field.into(),
        }
    }

    pub fn create_terraform(&self) -> Terraform {
        // resource "aws_api_gateway_rest_api" "api_gateway"
        let mut gateway_resource = TfResource::new_resource(
            "aws_api_gateway_rest_api",
            format!("{}", self.tf_identifier()),
        );
        gateway_resource.add_field("name", TfField::String(self.name.clone()));

        let stage = Stage {
            stage_name: self.stage_name.clone(),
        };

        let Some(domain) = self.domain.as_ref() else {
            todo!();
        };
        let mut path_mapping_resource = TfResource::new_resource(
            "aws_api_gateway_base_path_mapping",
            format!("{}", self.tf_identifier()),
        );
        path_mapping_resource
            .add_field(
                "domain_name",
                TfField::Variable(domain.borrow().var("domain_name")),
            )
            .add_field("api_id", TfField::Variable(self.var_gateway_rest_api("id")))
            .add_field("stage_name", stage.var("stage_name").into());

        let gateway_tf = gateway_resource
            .create_terraform()
            .combine(&path_mapping_resource.create_terraform());

        let routes = self
            .endpoints
            .iter()
            .map(|x| x.route.as_str())
            .collect::<HashSet<&str>>();

        let mut route_tree = PathNode::new();

        for route in routes.iter() {
            route_tree.add_path(&route);
        }

        let resource_paths = recurse(self, &route_tree, None, "");

        let resource_tf = resource_paths
            .iter()
            .map(|x| x.create_terraform(self))
            .reduce(|a, b| a.combine(&b))
            .unwrap_or(Terraform::default());

        let deployment = GatewayDeployment { gateway: self };
        let deployment_tf = deployment.create_terraform();

        let endpoints_tf = self
            .endpoints
            .iter()
            .map(|endpoint| {
                let path = resource_paths
                    .iter()
                    .find(|x| x.resource_path == endpoint.route)
                    .unwrap_or_else(|| panic!("Failed to find resource path for endpoint! {resource_paths:?} - {endpoint:?}"));
                endpoint.create_terraform(self, path)
            })
            .reduce(|a, b| a.combine(&b))
            .unwrap_or(Terraform::default());

        let stage_tf = stage.create_terraform(self, &deployment);

        gateway_tf
            .combine(&stage_tf)
            .combine(&resource_tf)
            .combine(&endpoints_tf)
            .combine(&deployment_tf)
    }
}
