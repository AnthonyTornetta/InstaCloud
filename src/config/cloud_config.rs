use std::collections::HashSet;

use anyhow::bail;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{api::api_config::ApiConfig, ConfigVariable, ConfigVariables, ContainsVariables};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct StaticConfig {
    name: String,
    root: String,
    domain: String,
    prefix: Option<String>,
}

impl ContainsVariables for StaticConfig {
    fn insert_value(&mut self, var_name: &ConfigVariable, new_value: &str) {
        self.name.insert_value(var_name, new_value);
        self.root.insert_value(var_name, new_value);
        self.domain.insert_value(var_name, new_value);
        self.prefix.insert_value(var_name, new_value);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum DatabaseEngine {
    Postgres,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct DatabaseConfigRaw {
    name: String,
    engine: String,
}

impl ContainsVariables for DatabaseConfigRaw {
    fn insert_value(&mut self, var_name: &ConfigVariable, new_value: &str) {
        self.name.insert_value(var_name, new_value);
        self.engine.insert_value(var_name, new_value);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct DatabaseConfig {
    name: String,
    engine: DatabaseEngine,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CloudConfigRaw {
    api: Option<Vec<ApiConfig>>,
    database: Option<Vec<DatabaseConfigRaw>>,
    #[serde(rename = "static")]
    static_files: Option<Vec<StaticConfig>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloudConfig {
    pub api: Vec<ApiConfig>,
    pub database: Vec<DatabaseConfig>,
    pub static_files: Vec<StaticConfig>,
}

impl ContainsVariables for CloudConfigRaw {
    fn insert_value(&mut self, var_name: &ConfigVariable, new_value: &str) {
        if let Some(api) = &mut self.api {
            api.iter_mut().for_each(|x| {
                // x.root = format!("{base_path}/{}", x.root);
                x.insert_value(var_name, new_value);
            });
        }
        if let Some(database) = &mut self.database {
            database
                .iter_mut()
                .for_each(|x| x.insert_value(var_name, new_value));
        }
        if let Some(static_files) = &mut self.static_files {
            static_files.iter_mut().for_each(|x| {
                // x.root = format!("{base_path}/{}", x.root);
                x.insert_value(var_name, new_value)
            });
        }
    }
}

trait ParseConfig {
    type Output;
    type Error;

    fn parse(self) -> Result<Self::Output, Self::Error>;
}

#[derive(Error, Debug)]
enum DatabaseConfigError {
    #[error("The database `{0}`'s engine '{1}' is not a valid type.")]
    InvalidDatabaseEngine(String, String),
}

impl ParseConfig for DatabaseConfigRaw {
    type Error = DatabaseConfigError;
    type Output = DatabaseConfig;

    fn parse(self) -> Result<Self::Output, Self::Error> {
        match self.engine.to_lowercase().as_str() {
            "postgres" => Ok(DatabaseConfig {
                engine: DatabaseEngine::Postgres,
                name: self.name,
            }),
            _ => Err(DatabaseConfigError::InvalidDatabaseEngine(
                self.name,
                self.engine,
            )),
        }
    }
}

pub trait NameUniquenessChecker {
    /// Asserts all strings are unique (ignoring capitalization)
    fn check_all_unique_names(self) -> bool;
}

impl<'a, T: Iterator<Item = &'a str> + Clone> NameUniquenessChecker for T {
    fn check_all_unique_names(self) -> bool {
        let len = self.clone().into_iter().count();

        let set = self.map(|x| x.to_lowercase()).collect::<HashSet<String>>();

        set.len() == len
    }
}

impl CloudConfig {
    pub fn new(
        variables: &ConfigVariables,
        mut raw_config: CloudConfigRaw,
        base_path: &str,
    ) -> anyhow::Result<Self> {
        for (variable, value) in variables.iter() {
            raw_config.insert_value(variable, value);
        }

        let mut last_error = None;

        let mut api = raw_config.api.unwrap_or(vec![]);
        let mut static_files = raw_config.static_files.unwrap_or(vec![]);

        let databases = raw_config
            .database
            .unwrap_or(vec![])
            .into_iter()
            .map_while(|x| {
                let result = x.parse();
                match result {
                    Ok(res) => Some(res),
                    Err(err) => {
                        last_error = Some(err);
                        return None;
                    }
                }
            })
            .collect::<Vec<DatabaseConfig>>();

        if let Some(error) = last_error {
            bail!(error);
        }

        if !api.iter().map(|x| x.name.as_str()).check_all_unique_names() {
            bail!("Not all names for {api:?} are unique!");
        }
        if !databases
            .iter()
            .map(|x| x.name.as_str())
            .check_all_unique_names()
        {
            bail!("Not all names for {databases:?} are unique!");
        }
        if !static_files
            .iter()
            .map(|x| x.name.as_str())
            .check_all_unique_names()
        {
            bail!("Not all names for {static_files:?} are unique!");
        }

        for api in api.iter_mut() {
            api.root = format!("{base_path}/{}", api.root);
        }
        for static_file in static_files.iter_mut() {
            static_file.root = format!("{base_path}/{}", static_file.root);
        }

        Ok(Self {
            api,
            database: databases,
            static_files,
        })
    }
}
