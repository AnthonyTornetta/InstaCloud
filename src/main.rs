use std::fs;

use api::{process_api_definition, setup_api_dir, setup_api_dir_root};
use config::{
    api::load_definitions,
    cloud_config::{CloudConfig, CloudConfigRaw},
    ConfigVariable, ConfigVariables,
};
use stack::{
    api::{
        domain_name::{Certificate, Domain, EndpointConfiguration},
        endpoint::{ApiEndpoint, HttpMethod},
        gateway::ApiGateway,
    },
    iam::role::{Role, RoleAction, RoleEffect, RolePolicy, RoleService},
    lambda::{LambdaFunction, LambdaRuntime},
    shared,
};
use walkdir::WalkDir;

mod api;
mod config;
mod database;
pub mod stack;

fn main() {
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
            role,
            file_path: "temp.js".into(),
            runtime: LambdaRuntime::NodeJs20,
            environment_variables: Default::default(),
        },
        http_method: HttpMethod::Get,
    };

    let gateway = ApiGateway {
        name: "API Gateway".into(),
        stage_name: "prod".into(),
        domain: Some(dn.clone()),
        endpoints: vec![endpoint],
    };

    let cert_tf = cert.borrow().create_terraform();
    let dn_tf = dn.borrow().create_terraform();
    let gw_tf = gateway.create_terraform();

    let tf = cert_tf.combine(&dn_tf).combine(&gw_tf);

    println!("{tf}");
}

fn main2() -> anyhow::Result<()> {
    let base_path = "samples/testing";

    let cloud_toml = fs::read_to_string(format!("{base_path}/cloud.toml"))?;

    let cloud_config =
        toml::from_str::<CloudConfigRaw>(&cloud_toml).expect("Failed to parse cloud.toml");

    // TODO: load from config file/env file
    let mut vars = ConfigVariables::new();
    vars.insert(
        ConfigVariable::new("domain").unwrap(),
        "example.com".to_owned(),
    );

    let cloud_config = CloudConfig::new(&vars, cloud_config, base_path)?;

    // Removes all old generated files w/out deleting the current terraform state.
    let delete_extensions = ["zip", "tf"];
    for entry in WalkDir::new("terraform/generated") {
        let Ok(entry) = entry else {
            continue;
        };

        if entry
            .path()
            .extension()
            .map(|x| delete_extensions.contains(&x.to_str().unwrap_or("")))
            .unwrap_or(false)
        {
            fs::remove_file(entry.path()).expect("Unable to remove old tf file.");
        }
    }

    fs::create_dir_all("terraform/generated").expect("Unable to create generated dir!");

    // This entire thing is pure evil and should be re-written asap
    let mut defs = vec![];

    let depends_on = cloud_config
        .api
        .iter()
        .flat_map(|api_config| {
            let api_identifier = api_config.tf_prefix();

            let root_path = setup_api_dir_root(&api_config.domain);

            let config_defs = load_definitions(api_config, &vars)
                .unwrap_or_else(|_| panic!("Failed to laod API definitions for: {api_config:?}"));

            let res = config_defs
                .iter()
                .map(|x| {
                    format!(
                        "aws_api_gateway_integration.lambda_integration_{}_{}",
                        api_identifier, x.name
                    )
                })
                .collect::<Vec<String>>();

            defs.push((config_defs, api_config, root_path));

            res
        })
        .collect::<Vec<String>>()
        .join(",\n\t\t");

    for (config_defs, api_config, root_path) in defs {
        setup_api_dir(&config_defs, &root_path);

        let depends_on = cloud_config
            .api
            .iter()
            .filter(|cfg| cfg.domain == api_config.domain)
            .flat_map(|api_config| {
                let api_identifier = api_config.tf_prefix();

                let config_defs = load_definitions(api_config, &vars).unwrap_or_else(|_| {
                    panic!("Failed to laod API definitions for: {api_config:?}")
                });

                let res = config_defs
                    .iter()
                    .map(|x| {
                        format!(
                            "aws_api_gateway_integration.lambda_integration_{}_{}",
                            api_identifier, x.name
                        )
                    })
                    .collect::<Vec<String>>();

                res
            })
            .collect::<Vec<String>>()
            .join(",\n\t\t");

        for def in config_defs {
            process_api_definition(
                &def,
                api_config,
                &depends_on,
                &root_path,
                &api_config.domain,
            );
        }
    }

    Ok(())
}
