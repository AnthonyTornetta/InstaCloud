use std::hash::{DefaultHasher, Hash, Hasher};

use crate::stack::{
    tf::{Terraform, TfField, TfResource, TfVar},
    Shared,
};

use super::{domain_name::Domain, endpoint::ApiEndpoint};

#[derive(Default)]
pub struct ApiGateway {
    pub name: String,
    pub domain: Option<Shared<Domain>>,
    pub stage_name: String,

    pub endpoints: Vec<ApiEndpoint>,
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

    pub fn create_terraform(&self) -> Terraform {
        // resource "aws_api_gateway_rest_api" "api_gateway"
        let mut gateway_resource = TfResource::new_resource(
            "aws_api_gateway_rest_api",
            format!("{}", self.tf_identifier()),
        );
        gateway_resource.add_field("name", TfField::String(self.name.clone()));

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
            .add_field("stage_name", TfField::String(self.stage_name.clone()));

        gateway_resource
            .create_terraform()
            .combine(&path_mapping_resource.create_terraform())
    }
}
