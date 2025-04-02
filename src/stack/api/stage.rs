use std::hash::{DefaultHasher, Hash, Hasher};

use crate::stack::tf::{Terraform, TerraformEntity, TfField, TfResource};

use super::{deployment::GatewayDeployment, gateway::ApiGateway};

pub struct Stage {
    pub stage_name: String,
}

impl TerraformEntity for Stage {
    fn tf_type() -> &'static str {
        "aws_api_gateway_stage"
    }
    fn data_type() -> crate::stack::tf::TfDataType {
        crate::stack::tf::TfDataType::Resource
    }
    fn tf_identifier(&self) -> String {
        let mut hasher = DefaultHasher::default();
        self.stage_name.hash(&mut hasher);
        format!("stage_{}", hasher.finish())
    }
}

impl Stage {
    pub fn create_terraform(
        &self,
        gateway: &ApiGateway,
        deployment: &GatewayDeployment,
    ) -> Terraform {
        /*
          deployment_id = aws_api_gateway_deployment.endpoint_lambda_5710212488084223146.id
          rest_api_id   = aws_api_gateway_rest_api.gateway_8169927463589532339.id
          stage_name    = "prod"
        }

                */
        TfResource::new_resource(Self::tf_type(), self.tf_identifier())
            .add_field("stage_name", self.stage_name.clone().into())
            .add_field("deployment_id", TfField::Variable(deployment.var("id")))
            .add_field(
                "rest_api_id",
                TfField::Variable(gateway.var_gateway_rest_api("id")),
            )
            .create_terraform()
    }
}
