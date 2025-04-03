use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod api;
pub mod cloud_config;
mod loading;

pub trait ContainsVariables {
    fn replace_variables(&mut self, vars: &ConfigVariables);
}

impl ContainsVariables for String {
    fn replace_variables(&mut self, vars: &ConfigVariables) {
        for (k, v) in vars.0.iter() {
            *self = self.replace(&k.0, v);
        }
    }
}

impl<T: ContainsVariables + Sized> ContainsVariables for Vec<T> {
    fn replace_variables(&mut self, vars: &ConfigVariables) {
        for item in self {
            item.replace_variables(vars);
        }
    }
}

impl<T: ContainsVariables + Sized> ContainsVariables for Option<T> {
    fn replace_variables(&mut self, vars: &ConfigVariables) {
        if let Some(s) = self {
            s.replace_variables(vars);
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
