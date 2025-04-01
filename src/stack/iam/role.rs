use crate::stack::tf::Terraform;

#[derive(Default, Debug)]
pub enum RoleEffect {
    #[default]
    Allow,
}

#[derive(Debug, Default)]
pub enum RoleAction {
    #[default]
    AssumeRole,
}

pub enum RoleService {
    EC2,
    Lambda,
}

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
}

// pub struct Role {
//     name: String,
//     policy:
//  name = "lambda_role"
//
//   assume_role_policy = jsonencode({
//     Version = "2012-10-17"
//     Statement = [
//       {
//         Action = "sts:AssumeRole"
//         Effect = "Allow"
//         Principal = {
//           Service = "lambda.amazonaws.com"
//         }
//       }
//     ]
//   })
//
// }
