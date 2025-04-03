use std::fs;

use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::{
    cloud::api::ApiDefinitionError,
    config::{
        api::api_config::{ApiConfig, ApiEndpoint},
        ConfigVariables, ContainsVariables,
    },
};

pub fn create_api_definitions(
    api_config: &ApiConfig,
    vars: &ConfigVariables,
) -> Result<Vec<ApiEndpoint>, ApiDefinitionError> {
    let mut result = vec![];

    let base_path = &api_config.root;
    for item in WalkDir::new(base_path).into_iter().filter_map(|x| x.ok()) {
        let item_path = item.path();

        if item_path.is_dir() {
            continue;
        }

        if item.file_name() != "endpoints.toml" {
            continue;
        }

        let item_path_str = item_path.to_str().unwrap();

        let data = toml::from_str::<ApiEndpointsRaw>(
            &fs::read_to_string(item.file_name())
                .map_err(|e| ApiDefinitionError::CannotReadTomlFile(e))?,
        )
        .map_err(|e| ApiDefinitionError::CannotParseTomlFile(e))?;

        for mut x in data.api {
            x.replace_variables(vars);

            result.push(ApiEndpoint {
                read: x.read.unwrap_or_default(),
                write: x.write.unwrap_or_default(),
                name: x.name,
                file: x.file,
                route: format!(
                    "{}/{}",
                    api_config.prefix.as_ref().map(|x| x.as_str()).unwrap_or(""),
                    x.route
                        .as_ref()
                        .map(|x| x.as_str())
                        .unwrap_or(&item_path_str[base_path.len()..])
                ),
                method: x
                    .method
                    .as_str()
                    .try_into()
                    .map_err(|_| ApiDefinitionError::InvalidMethod(x.method.clone()))?,
            });
        }
    }

    Ok(result)
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ApiConfigRaw {
    pub name: String,
    pub root: String,
    pub domain: String,
    pub prefix: Option<String>,
}

impl ContainsVariables for ApiConfigRaw {
    fn replace_variables(&mut self, vars: &ConfigVariables) {
        self.name.replace_variables(vars);
        self.root.replace_variables(vars);
        self.domain.replace_variables(vars);
        self.prefix.replace_variables(vars);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ApiEndpointRaw {
    name: String,
    method: String,
    route: Option<String>,
    file: String,
    read: Option<Vec<String>>,
    write: Option<Vec<String>>,
}

impl ContainsVariables for ApiEndpointRaw {
    fn replace_variables(&mut self, vars: &ConfigVariables) {
        self.method.replace_variables(vars);
        self.route.replace_variables(vars);
        self.file.replace_variables(vars);
        self.name.replace_variables(vars);
        self.read.replace_variables(vars);
        self.write.replace_variables(vars);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ApiEndpointsRaw {
    api: Vec<ApiEndpointRaw>,
}

pub fn load_raw_api_config(
    raw: ApiConfigRaw,
    vars: &ConfigVariables,
) -> Result<ApiConfig, ApiDefinitionError> {
    let mut api_def = ApiConfig {
        name: raw.name,
        endpoints: vec![],
        domain: raw.domain,
        root: raw.root,
        prefix: raw.prefix,
    };

    api_def.endpoints = create_api_definitions(&api_def, vars)?;

    Ok(api_def)
}

pub fn load_api_configs(
    api_defs: impl Iterator<Item = ApiConfigRaw>,
    vars: &ConfigVariables,
) -> Result<Vec<ApiConfig>, ApiDefinitionError> {
    api_defs
        .map(|raw| load_raw_api_config(raw, vars))
        .collect::<Result<Vec<_>, ApiDefinitionError>>()
}
