use super::tf::Terraform;

#[derive(Default)]
pub enum Region {
    #[default]
    UsEast1,
}

impl Region {
    pub fn to_tf_string(&self) -> &'static str {
        match self {
            Self::UsEast1 => "us-east-1",
        }
    }
}

pub struct Provider {
    pub region: Region,
}

impl Provider {
    pub fn create_terraform(&self) -> Terraform {
        Terraform(format!(
            r#"provider "aws" {{
    region = "{}"
}}"#,
            self.region.to_tf_string()
        ))
    }
}
