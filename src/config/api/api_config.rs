use std::fs;

use anyhow::bail;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use walkdir::WalkDir;

use crate::config::{cloud_config::NameUniquenessChecker, ConfigVariables};

use super::super::{ConfigVariable, ContainsVariables};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ApiConfig {
    pub name: String,
    pub root: String,
    pub domain: String,
    pub prefix: Option<String>,
}

impl ContainsVariables for ApiConfig {
    fn insert_value(&mut self, var_name: &ConfigVariable, new_value: &str) {
        self.name.insert_value(var_name, new_value);
        self.root.insert_value(var_name, new_value);
        self.domain.insert_value(var_name, new_value);
        self.prefix.insert_value(var_name, new_value);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum HttpMethod {
    Any,
    Get,
    Post,
    Put,
    Delete,
    Options,
    Patch,
    Head,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ApiDefinitionRaw {
    name: String,
    method: String,
    route: String,
    file: String,
    read: Option<Vec<String>>,
    write: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ApiToml {
    api: Vec<ApiDefinitionRaw>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiDefinition {
    pub name: String,
    pub method: HttpMethod,
    pub route: String,
    pub file: String,
    pub root: String,
    pub read: Vec<String>,
    pub write: Vec<String>,
}

#[derive(Error, Debug)]
pub enum ApiDefinitionError {
    #[error("Invalid API method: {0}")]
    InvalidMethod(String),
    #[error("API definitions contain duplicate name")]
    DuplicateNameFound,
}

impl ApiDefinition {
    pub fn create_definition(
        raw: ApiDefinitionRaw,
        root: String,
        mut prefix: &str,
    ) -> Result<Self, ApiDefinitionError> {
        let method = match raw.method.to_lowercase().as_str() {
            "any" => HttpMethod::Any,
            "get" => HttpMethod::Get,
            "post" => HttpMethod::Post,
            "put" => HttpMethod::Put,
            "delete" => HttpMethod::Delete,
            "options" => HttpMethod::Options,
            "patch" => HttpMethod::Patch,
            "head" => HttpMethod::Head,
            _ => return Err(ApiDefinitionError::InvalidMethod(raw.method)),
        };

        while prefix.ends_with('/') {
            prefix = &prefix[0..prefix.len() - 1];
        }

        Ok(Self {
            file: raw.file,
            method,
            name: raw.name,
            route: format!("{prefix}/{}", raw.route),
            root,
            read: raw.read.unwrap_or_default(),
            write: raw.write.unwrap_or_default(),
        })
    }
}

impl ContainsVariables for ApiDefinitionRaw {
    fn insert_value(&mut self, var_name: &ConfigVariable, new_value: &str) {
        self.file.insert_value(var_name, new_value);
        self.method.insert_value(var_name, new_value);
        self.name.insert_value(var_name, new_value);
        self.route.insert_value(var_name, new_value);

        if let Some(r) = &mut self.read {
            r.iter_mut()
                .for_each(|x| x.insert_value(var_name, new_value));
        }
        if let Some(w) = &mut self.write {
            w.iter_mut()
                .for_each(|x| x.insert_value(var_name, new_value));
        }
    }
}

pub fn create_api_definitions(
    api_config: &ApiConfig,
    variables: &ConfigVariables,
) -> anyhow::Result<Vec<ApiDefinition>> {
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

        result.append(&mut read_api_config(
            &item_path_str[0..item_path_str.len() - "endpoints.toml".len()],
            item_path_str,
            variables,
            api_config.prefix.as_ref().map(|x| x.as_str()).unwrap_or(""),
        )?);
    }

    Ok(result)
}

fn read_api_config(
    root_dir: &str,
    config_path: &str,
    variables: &ConfigVariables,
    prefix: &str,
) -> anyhow::Result<Vec<ApiDefinition>> {
    let file = fs::read_to_string(config_path).unwrap();

    let mut raw_config = toml::from_str::<ApiToml>(&file)?;

    for (variable, value) in variables.iter() {
        for def in raw_config.api.iter_mut() {
            def.insert_value(variable, value);
        }
    }

    let mut last_error = None;

    let definitions = raw_config
        .api
        .into_iter()
        .map_while(|x| {
            let result = ApiDefinition::create_definition(x, root_dir.to_owned(), prefix);
            match result {
                Ok(res) => Some(res),
                Err(err) => {
                    last_error = Some(err);
                    return None;
                }
            }
        })
        .collect::<Vec<ApiDefinition>>();

    if let Some(error) = last_error {
        bail!(error);
    }

    if !definitions
        .iter()
        .map(|x: &ApiDefinition| x.name.as_str())
        .check_all_unique_names()
    {
        bail!(ApiDefinitionError::DuplicateNameFound);
    }

    Ok(definitions)
}
