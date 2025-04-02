use crate::stack::{
    lambda::LambdaFunction,
    tf::{Terraform, TerraformEntity, TfField, TfResource},
};

use super::gateway::{ApiGateway, ResourcePath};

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
}

#[derive(Clone, Debug)]
pub struct ApiEndpoint {
    pub lambda: LambdaFunction,
    pub http_method: HttpMethod,
    pub route: String,
}

impl TerraformEntity for ApiEndpoint {
    fn data_type() -> crate::stack::tf::TfDataType {
        crate::stack::tf::TfDataType::Resource
    }
    fn tf_type() -> &'static str {
        "aws_api_gateway_method"
    }
    fn tf_identifier(&self) -> String {
        format!("endpoint_{}", self.lambda.tf_identifier())
    }
}

pub struct ApiGatewayIntegration<'a> {
    http_method: HttpMethod,
    api_endpoint: &'a ApiEndpoint,
}

impl<'a> TerraformEntity for ApiGatewayIntegration<'a> {
    fn data_type() -> crate::stack::tf::TfDataType {
        crate::stack::tf::TfDataType::Resource
    }
    fn tf_type() -> &'static str {
        "aws_api_gateway_integration"
    }
    fn tf_identifier(&self) -> String {
        format!("integration_{}", self.api_endpoint.tf_identifier())
    }
}

impl<'a> ApiGatewayIntegration<'a> {
    pub fn create_terraform(
        &self,
        gateway: &ApiGateway,
        resource_path: &ResourcePath,
    ) -> Terraform {
        let http_method = match self.http_method {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
        }
        .to_owned();

        let mut tf_gateway_integration =
            TfResource::new_resource("aws_api_gateway_integration", self.tf_identifier());

        tf_gateway_integration
            .add_field(
                "rest_api_id",
                TfField::Variable(gateway.var_gateway_rest_api("id")),
            )
            .add_field("resource_id", TfField::Variable(resource_path.var("id")))
            .add_field("http_method", TfField::String(http_method))
            // May not need this depends_on?
            .depends_on(&self.api_endpoint.lambda)
            // lambda can only be invoked w/ POST requests, so this turns the "GET" into a "POST" the lambda can handle
            .add_field("integration_http_method", TfField::String("POST".into()))
            .add_field("type", TfField::String("AWS_PROXY".into()))
            .add_field(
                "uri",
                TfField::Variable(self.api_endpoint.lambda.var("invoke_arn")),
            )
            .create_terraform()
    }
}

impl ApiEndpoint {
    pub fn unique_key(&self) -> String {
        self.lambda.unique_key()
    }

    pub fn zip_file(&self, path: &str) -> std::io::Result<()> {
        self.lambda.zip_file(path)
    }

    pub fn api_integration(&self) -> ApiGatewayIntegration<'_> {
        ApiGatewayIntegration {
            http_method: self.http_method.clone(),
            api_endpoint: self,
        }
    }

    pub fn create_terraform(
        &self,
        gateway: &ApiGateway,
        resource_path: &ResourcePath,
    ) -> Terraform {
        let lambda_tf = self.lambda.create_terraform();

        let http_method = match self.http_method {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
        }
        .to_owned();

        let mut tf_gateway_method = TfResource::new_resource(Self::tf_type(), self.tf_identifier());
        tf_gateway_method
            .add_field(
                "rest_api_id",
                TfField::Variable(gateway.var_gateway_rest_api("id")),
            )
            .add_field("resource_id", TfField::Variable(resource_path.var("id")))
            .add_field("http_method", TfField::String(http_method.clone()))
            .add_field("authorization", TfField::String("NONE".into()));

        let gw_integration = self.api_integration();

        let permission_tf = TfResource::new_resource("aws_lambda_permission", self.tf_identifier())
            .add_field(
                "statement_id",
                TfField::String("AllowAPIGatewayInvoke".into()),
            )
            .add_field("action", TfField::String("lambda:InvokeFunction".into()))
            .add_field(
                "function_name",
                TfField::Variable(self.lambda.var("function_name")),
            )
            .add_field(
                "principal",
                TfField::String("apigateway.amazonaws.com".into()),
            )
            .add_field(
                "source_arn",
                TfField::String(format!(
                    "${{{}}}/*/*/*",
                    gateway.var_gateway_rest_api("execution_arn").to_tf_string()
                )),
            )
            .create_terraform();

        //         resource "aws_api_gateway_deployment" "api_deployment_{api_identifier}_{function_name}" {
        //   depends_on = [
        //     {depends_on}
        //   ] # [aws_api_gateway_integration.lambda_integration_{api_identifier}_{function_name}]
        //
        //   rest_api_id = aws_api_gateway_rest_api.api_gateway.id
        //   # https://stackoverflow.com/questions/48955987/missing-authentication-token-on-unauthenticated-method
        //   # This being a fixed thing is preventing re-applies from working properly
        //   stage_name  = var.api_gateway_stage_name # should use aws_api_gateway_stage resource instead.
        // }
        // let deployment =
        //     TfResource::new_resource("aws_api_gateway_deployment", self.tf_identifier())
        //         .depends_on(&gw_integration)
        //         .add_field(
        //             "rest_api_id",
        //             TfField::Variable(gateway.var_gateway_rest_api("id")),
        //         )
        //         .add_field(
        //             "stage_name",
        //             TfField::Variable(gateway.var_path_mapping_resource("stage_name")),
        //         )
        //         .create_terraform();

        lambda_tf
            .combine(&tf_gateway_method.create_terraform())
            .combine(&gw_integration.create_terraform(gateway, resource_path))
            .combine(&permission_tf)
        // .combine(&deployment)
    }
}
