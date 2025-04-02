use std::hash::{DefaultHasher, Hash, Hasher};

use crate::stack::{
    tf::{Terraform, TfField, TfResource, TfVar},
    Shared,
};

#[derive(Default, Clone, Debug)]
pub enum EndpointConfiguration {
    #[default]
    Regional,
}

#[derive(Debug, Clone)]
pub struct Certificate {
    pub domain: String,
}

impl Certificate {
    fn unique_key(&self) -> String {
        let mut hasher = DefaultHasher::default();
        self.domain.hash(&mut hasher);
        hasher.finish().to_string()
    }

    pub fn tf_identifier(&self) -> String {
        format!("certificate_{}", self.unique_key())
    }

    pub fn create_terraform(&self) -> Terraform {
        let identifier = self.tf_identifier();

        TfResource::new_data("aws_acm_certificate", identifier)
            .add_field("domain", TfField::String(self.domain.clone()))
            .add_field(
                "statuses",
                TfField::List(vec![TfField::String("ISSUED".into())]),
            )
            .create_terraform()
    }

    pub fn var(&self, field: impl Into<String>) -> TfVar {
        TfVar::Data {
            data_name: "aws_acm_certificate".into(),
            data_identifier: self.tf_identifier(),
            field: field.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Domain {
    pub endpoint_configuration: EndpointConfiguration,
    pub certificate: Shared<Certificate>,
}

impl Domain {
    fn unique_key(&self) -> String {
        self.certificate.borrow().unique_key()
    }

    pub fn tf_identifier(&self) -> String {
        format!("domain_{}", self.unique_key())
    }

    pub fn var(&self, field: impl Into<String>) -> TfVar {
        TfVar::Resource {
            resource_name: "aws_api_gateway_domain_name".into(),
            resource_identifier: self.tf_identifier(),
            field: field.into(),
        }
    }

    pub fn create_terraform(&self) -> Terraform {
        let endpoint_cfg_type = match self.endpoint_configuration {
            EndpointConfiguration::Regional => "REGIONAL",
        };

        let cert = self.certificate.borrow();

        TfResource::new_resource("aws_api_gateway_domain_name", &self.tf_identifier())
            .add_field("domain_name", TfField::String(cert.domain.clone()))
            .add_field(
                "regional_certificate_arn",
                TfField::Variable(cert.var("arn")),
            )
            .add_field(
                "endpoint_configuration",
                TfField::Map(
                    vec![(
                        "types".into(),
                        TfField::List(vec![TfField::String(endpoint_cfg_type.to_string())]),
                    )]
                    .into_iter()
                    .collect(),
                ),
            )
            .create_terraform()
    }
}
