use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::{api::api_config::ApiConfig, loading::api::ApiConfigRaw};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CloudConfigRaw {
    pub api: Option<Vec<ApiConfigRaw>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloudConfig {
    pub api: Vec<ApiConfig>,
}

trait ParseConfig {
    type Output;
    type Error;

    fn parse(self) -> Result<Self::Output, Self::Error>;
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

impl CloudConfig {}
