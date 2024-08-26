use std::{collections::HashMap, fs};

use cloud_config::{CloudConfig, CloudConfigRaw, ConfigVariable};

mod cloud_config;

fn main() -> anyhow::Result<()> {
    let cloud_toml = fs::read_to_string("samples/testing/cloud.toml")?;

    let cloud_config =
        toml::from_str::<CloudConfigRaw>(&cloud_toml).expect("Failed to parse cloud.toml");

    // TODO: load from config file/env file
    let mut vars = HashMap::new();
    vars.insert(
        ConfigVariable::new("domain").unwrap(),
        "example.com".to_owned(),
    );

    let cloud_config = CloudConfig::new(vars, cloud_config)?;

    println!("{cloud_config:?}");

    Ok(())
}
