use std::fs;

use cloud::api::HttpMethod;
use stack::{
    api::{
        domain_name::{Certificate, Domain, EndpointConfiguration},
        endpoint::ApiEndpoint,
        gateway::ApiGateway,
    },
    iam::role::{Role, RoleAction, RoleEffect, RolePolicy, RoleService},
    lambda::{LambdaFunction, LambdaRuntime},
    region::{Provider, Region},
    shared,
};

pub mod cloud;
mod config;
mod database;
pub mod stack;
mod tf_generation;

fn main() {
    let provider = Provider {
        region: Region::UsEast1,
    };

    let cert = shared(Certificate {
        domain: "api.cornchipss.com".into(),
    });

    let dn = shared(Domain {
        endpoint_configuration: EndpointConfiguration::Regional,
        certificate: cert.clone(),
    });

    let role = shared(Role {
        name: "LambdaRole".into(),
        policies: vec![RolePolicy {
            action: RoleAction::AssumeRole,
            effect: RoleEffect::Allow,
            service: RoleService::Lambda,
        }],
    });

    let endpoint = ApiEndpoint {
        lambda: LambdaFunction {
            role: role.clone(),
            file_path: "samples/testing/api-endpoints/based/posts/get-posts.js".into(),
            runtime: LambdaRuntime::NodeJs20,
            environment_variables: Default::default(),
        },
        http_method: HttpMethod::Get,
        route: "test".into(),
    };

    let endpoint2 = ApiEndpoint {
        lambda: LambdaFunction {
            role: role.clone(),
            file_path: "samples/testing/api-endpoints/based/other/get-other.js".into(),
            runtime: LambdaRuntime::NodeJs20,
            environment_variables: Default::default(),
        },
        http_method: HttpMethod::Get,
        route: "test2".into(),
    };

    // This should be done in endpoint!
    endpoint
        .zip_file("terraform/generated/test/")
        .expect("Failed to zip file!");

    endpoint2
        .zip_file("terraform/generated/test/")
        .expect("Failed to zip file!");

    let gateway = ApiGateway {
        name: "API Gateway".into(),
        stage_name: "prod".into(),
        domain: Some(dn.clone()),
        endpoints: vec![endpoint, endpoint2],
    };

    let provider_tf = provider.create_terraform();
    let role_tf = role.borrow().create_terraform();
    let cert_tf = cert.borrow().create_terraform();
    let dn_tf = dn.borrow().create_terraform();
    let gw_tf = gateway.create_terraform();

    let tf = provider_tf.combine(&role_tf.combine(&cert_tf.combine(&dn_tf).combine(&gw_tf)));

    fs::write("terraform/generated/test/main.tf", tf.to_string()).expect("Unable to write file!");

    println!("{tf}");
}
