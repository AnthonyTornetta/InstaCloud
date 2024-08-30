use api_config::{create_api_definitions, ApiConfig, ApiDefinition};

use super::ConfigVariables;

pub mod api_config;

pub fn load_definitions(
    api_config: &ApiConfig,
    variables: &ConfigVariables,
) -> anyhow::Result<Vec<ApiDefinition>> {
    create_api_definitions(api_config, &variables)
}
