use std::collections::{HashMap, HashSet};

use anyhow::bail;
use serde::{Deserialize, Serialize};
use thiserror::Error;

trait ContainsVariables {
    fn insert_value(&mut self, var_name: &ConfigVariable, new_value: &str);
}

impl ContainsVariables for String {
    fn insert_value(&mut self, var_name: &ConfigVariable, new_value: &str) {
        *self = self.replace(&var_name.0, new_value);
    }
}

impl<T: ContainsVariables + Sized> ContainsVariables for Option<T> {
    fn insert_value(&mut self, var_name: &ConfigVariable, new_value: &str) {
        if let Some(s) = self {
            s.insert_value(var_name, new_value);
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ApiConfig {
    name: String,
    root: String,
    domain: String,
    prefix: Option<String>,
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
    api: Vec<ApiConfig>,
    database: Vec<DatabaseConfig>,
    static_files: Vec<StaticConfig>,
}

impl ContainsVariables for CloudConfigRaw {
    fn insert_value(&mut self, var_name: &ConfigVariable, new_value: &str) {
        if let Some(api) = &mut self.api {
            api.iter_mut()
                .for_each(|x| x.insert_value(var_name, new_value));
        }
        if let Some(database) = &mut self.database {
            database
                .iter_mut()
                .for_each(|x| x.insert_value(var_name, new_value));
        }
        if let Some(static_files) = &mut self.static_files {
            static_files
                .iter_mut()
                .for_each(|x| x.insert_value(var_name, new_value));
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

fn check_all_unique<'a, T: Clone + Iterator<Item = &'a str>>(names: T) -> bool {
    let len = names.clone().into_iter().count();

    names
        .map(|x| x.to_lowercase())
        .collect::<HashSet<String>>()
        .len()
        == len
}

impl CloudConfig {
    pub fn new(
        variables: HashMap<ConfigVariable, String>,
        mut raw_config: CloudConfigRaw,
    ) -> anyhow::Result<Self> {
        for (variable, value) in variables {
            raw_config.insert_value(&variable, &value);
        }

        let mut last_error = None;

        let api = raw_config.api.unwrap_or(vec![]);
        let static_files = raw_config.static_files.unwrap_or(vec![]);

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

        if !check_all_unique(api.iter().map(|x| x.name.as_str())) {
            bail!("Not all names for {api:?} are unique!");
        }
        if !check_all_unique(databases.iter().map(|x| x.name.as_str())) {
            bail!("Not all names for {databases:?} are unique!");
        }
        if !check_all_unique(static_files.iter().map(|x| x.name.as_str())) {
            bail!("Not all names for {static_files:?} are unique!");
        }

        Ok(Self {
            api,
            database: databases,
            static_files,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConfigVariable(String);

#[derive(Error, Debug)]
pub enum ConfigVariableError {
    #[error("Invalid variable name - {0}")]
    InvalidName(String),
}

impl ConfigVariable {
    pub fn new(var_name: &str) -> Result<Self, ConfigVariableError> {
        if var_name.split_whitespace().count() != 1 {
            return Err(ConfigVariableError::InvalidName(var_name.to_owned()));
        }

        Ok(Self(format!("${var_name}")))
    }
}
