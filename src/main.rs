use std::fs;

use api::{process_api_definition, setup_api_dir};
use config::{
    api::load_definitions,
    cloud_config::{CloudConfig, CloudConfigRaw},
    ConfigVariable, ConfigVariables,
};

mod api;
mod config;

fn main() -> anyhow::Result<()> {
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

    let _ = fs::remove_dir_all("terraform/generated");
    fs::create_dir("terraform/generated").expect("Unable to create generated dir!");

    setup_api_dir();

    for api_config in &cloud_config.api {
        let config_defs = load_definitions(api_config, &vars)?;

        let depends_on = config_defs
            .iter()
            .map(|x| format!("aws_api_gateway_integration.lambda_integration_{}", x.name))
            .collect::<Vec<String>>()
            .join(",\n\t\t");

        for def in config_defs {
            process_api_definition(&def, &depends_on);
        }
    }

    Ok(())
}
