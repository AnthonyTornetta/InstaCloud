use std::fs;

use api::load_api_configs;
use walkdir::WalkDir;

use super::{
    cloud_config::{CloudConfig, CloudConfigRaw},
    ConfigVariable, ConfigVariables,
};

pub mod api;

pub fn load_configs(base_path: &str) -> anyhow::Result<CloudConfig> {
    let base_path = "samples/testing";

    let cloud_toml = fs::read_to_string(format!("{base_path}/cloud.toml"))?;

    let cloud_config_raw =
        toml::from_str::<CloudConfigRaw>(&cloud_toml).expect("Failed to parse cloud.toml");

    // TODO: load from config file/env file
    let mut vars = ConfigVariables::new();
    vars.insert(
        ConfigVariable::new("domain").unwrap(),
        "api.cornchipss.com".to_owned(),
    );

    let api_configs =
        load_api_configs(cloud_config_raw.api.unwrap_or_default().into_iter(), &vars)?;

    Ok(CloudConfig { api: api_configs })
}
