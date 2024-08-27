use std::fs;

use config::{
    api::load_definitions,
    cloud_config::{CloudConfig, CloudConfigRaw},
    ConfigVariable, ConfigVariables,
};

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

    for api_config in &cloud_config.api {
        let config_def = load_definitions(api_config, &vars)?;

        println!("{config_def:?}");
    }

    println!("{cloud_config:?}");

    Ok(())
}
