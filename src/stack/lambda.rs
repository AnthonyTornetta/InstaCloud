use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use super::{
    iam::role::Role,
    tf::{Terraform, TerraformEntity, TfField, TfResource, TfVar},
    Shared,
};

#[derive(Clone, Debug)]
pub enum LambdaRuntime {
    NodeJs20,
}

#[derive(Clone, Debug)]
pub struct LambdaFunction {
    pub role: Shared<Role>,
    pub runtime: LambdaRuntime,
    pub file_path: String,
    pub environment_variables: HashMap<String, String>,
}

impl TerraformEntity for LambdaFunction {
    fn var(&self, field: impl Into<String>) -> TfVar {
        TfVar::Resource {
            resource_name: "aws_lambda_function".into(),
            resource_identifier: self.tf_identifier(),
            field: field.into(),
        }
    }

    fn tf_identifier(&self) -> String {
        format!("lambda_{}", self.unique_key())
    }

    fn tf_type() -> &'static str {
        "aws_lambda_function"
    }

    fn data_type() -> super::tf::TfDataType {
        super::tf::TfDataType::Resource
    }
}

impl LambdaFunction {
    pub fn unique_key(&self) -> String {
        let mut hasher = DefaultHasher::default();
        self.file_path.hash(&mut hasher);
        hasher.finish().to_string()
    }

    pub fn zip_path(&self) -> String {
        format!("lambda_function_{}.zip", self.unique_key())
    }

    pub fn create_terraform(&self) -> Terraform {
        /*
                resource "aws_lambda_function" "node_lambda_{api_identifier}_{function_name}" {
          function_name = "{api_identifier}_{function_name}" # var.lambda_function_name
          role          = aws_iam_role.lambda_role.arn
          handler       = "index.handler"
          runtime       = "{runtime}" # var.lambda_runtime

          filename = "lambda_function_{api_identifier}_{function_name}.zip"

          source_code_hash = filebase64sha256("lambda_function_{api_identifier}_{function_name}.zip")

          environment {
            variables = {environment_variables}
          }
        }
                * */

        let runtime = match self.runtime {
            LambdaRuntime::NodeJs20 => "nodejs20.x",
        }
        .to_owned();

        let environment_vars = self
            .environment_variables
            .iter()
            .map(|(key, val)| (key.to_owned(), TfField::String(val.into())))
            .collect::<HashMap<String, TfField>>();

        let mut lambda_resource =
            TfResource::new_resource("aws_lambda_function", self.tf_identifier());

        lambda_resource
            .add_field("filename", TfField::String(self.zip_path()))
            .add_field("function_name", TfField::String(self.tf_identifier()))
            .add_field("role", TfField::Variable(self.role.borrow().var("arn")))
            .add_field("handler", TfField::String("index.handler".into()))
            .add_field("runtime", TfField::String(runtime))
            .add_field(
                "source_code_hash",
                TfField::Raw(format!("filebase64sha256(\"{}\")", self.zip_path())),
            )
            .add_field("environment", TfField::Map(environment_vars));

        lambda_resource.create_terraform()
    }
}
