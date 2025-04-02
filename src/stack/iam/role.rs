use std::hash::{DefaultHasher, Hash, Hasher};

use crate::stack::tf::{Terraform, TfField, TfResource, TfVar};

#[derive(Default, Debug, Clone)]
pub enum RoleEffect {
    #[default]
    Allow,
}

#[derive(Debug, Default, Clone)]
pub enum RoleAction {
    #[default]
    AssumeRole,
}

#[derive(Clone, Debug)]
pub enum RoleService {
    EC2,
    Lambda,
}

#[derive(Clone, Debug)]
pub struct RolePolicy {
    pub action: RoleAction,
    pub effect: RoleEffect,
    pub service: RoleService,
}

impl RolePolicy {
    pub fn new(service: RoleService) -> Self {
        Self {
            service,
            action: Default::default(),
            effect: Default::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Role {
    pub name: String,
    pub policies: Vec<RolePolicy>,
}

impl Role {
    pub fn new(name: impl Into<String>, policies: Vec<RolePolicy>) -> Self {
        Self {
            name: name.into(),
            policies,
        }
    }

    pub fn unique_key(&self) -> String {
        let mut hasher = DefaultHasher::default();
        self.name.hash(&mut hasher);
        hasher.finish().to_string()
    }

    pub fn tf_identifier(&self) -> String {
        format!("role_{}", self.unique_key())
    }

    pub fn var(&self, field: impl Into<String>) -> TfVar {
        TfVar::Resource {
            resource_name: "aws_iam_role".into(),
            resource_identifier: self.tf_identifier(),
            field: field.into(),
        }
    }

    pub fn create_terraform(&self) -> Terraform {
        let mut tf_res = TfResource::new_resource("aws_iam_role", self.tf_identifier());
        /*
        resource "aws_iam_role" "lambda_role" {
          name = "lambda_role"

          assume_role_policy = jsonencode({
            Version = "2012-10-17"
            Statement = [
              {
                Action = "sts:AssumeRole"
                Effect = "Allow"
                Principal = {
                  Service = "lambda.amazonaws.com"
                }
              }
            ]
          })
        }
        */

        let statement = self
            .policies
            .iter()
            .map(|p| {
                let action = match p.action {
                    RoleAction::AssumeRole => "sts:AssumeRole",
                };
                let service = match p.service {
                    RoleService::Lambda => "lambda.amazonaws.com",
                    RoleService::EC2 => "ec2.amazonaws.com",
                };

                format!(
                    r#"{{
    "Action" = "{action}"
    "Effect" = "Allow"
    "Principal" = {{
        "Service" = "{service}"
    }}
}}"#,
                )
            })
            .collect::<Vec<String>>()
            .join(",\n\t\t\t");

        let assume_role_policy = format!(
            r#"jsonencode({{
        "Version": "2012-10-17",
        "Statement": [
            {statement}
        ]
    }})"#
        );

        tf_res
            .add_field("name", TfField::String(self.name.clone()))
            .add_field("assume_role_policy", TfField::Raw(assume_role_policy));

        tf_res.create_terraform()
    }
}
