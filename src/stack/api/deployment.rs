use std::collections::HashMap;

use crate::stack::{
    api::endpoint::ApiGatewayIntegration,
    lambda::LambdaFunction,
    tf::{Terraform, TerraformEntity, TfField, TfResource, TfVar},
};

use super::{endpoint::ApiEndpoint, gateway::ApiGateway};

pub struct GatewayDeployment<'a> {
    pub gateway: &'a ApiGateway,
}

impl<'a> TerraformEntity for GatewayDeployment<'a> {
    fn tf_type() -> &'static str {
        "aws_api_gateway_deployment"
    }
    fn tf_identifier(&self) -> String {
        self.gateway.tf_identifier()
    }
    fn data_type() -> crate::stack::tf::TfDataType {
        crate::stack::tf::TfDataType::Resource
    }
}

impl<'a> GatewayDeployment<'a> {
    pub fn create_terraform(&self) -> Terraform {
        /*
                        depends_on  = [aws_api_gateway_integration.lambda_5710212488084223146_endpoint_lambda_5710212488084223146]
                rest_api_id = aws_api_gateway_rest_api.gateway_8169927463589532339.id

        triggers = {
            redeployment = sha1(jsonencode([
              aws_api_gateway_method.users_get,
              aws_api_gateway_integration.users_get_lambda,
              aws_api_gateway_method.orders_post,
              aws_api_gateway_integration.orders_post_lambda
            ]))
          }

          lifecycle {
            create_before_destroy = true
          }
                      }
                      */
        let mut resource = TfResource::new_resource(Self::tf_type(), self.tf_identifier());
        resource
            .add_field(
                "rest_api_id",
                self.gateway.var_gateway_rest_api("id").into(),
            )
            .add_field(
                "lifecycle",
                TfField::map(vec![(
                    "create_before_destroy".to_string(),
                    TfField::Raw("true".into()),
                )]),
            );

        for endpoint in self.gateway.endpoints.iter() {
            resource.depends_on(endpoint);
        }

        let redeployment_json = self
            .gateway
            .endpoints
            .iter()
            .map(|x| {
                format!(
                    "{}.{},\n{}.{}",
                    ApiEndpoint::tf_type(),
                    x.tf_identifier(),
                    ApiGatewayIntegration::tf_type(),
                    x.api_integration().tf_identifier()
                )
            })
            .collect::<Vec<String>>()
            .join(",\n");

        let mut t_map = HashMap::new();
        t_map.insert(
            "redeployment".to_string(),
            TfField::Raw(format!("sha1(jsonencode([\n{redeployment_json}\n]))")),
        );

        resource.add_field("triggers", TfField::Object(t_map));

        resource.create_terraform()
    }
}
