use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod api;
pub mod cloud_config;

pub trait ContainsVariables {
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

#[derive(Default, Debug)]
pub struct ConfigVariables(HashMap<ConfigVariable, String>);

impl ConfigVariables {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, variable: ConfigVariable, value: String) {
        self.0.insert(variable, value);
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (&'a ConfigVariable, &'a String)> {
        self.0.iter()
    }
}
