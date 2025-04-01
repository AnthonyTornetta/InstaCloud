use std::collections::HashMap;

use derive_more::Display;

#[derive(Debug, Clone, Display)]
pub struct Terraform(pub String);

impl Terraform {
    pub fn combine(&self, other: &Terraform) -> Terraform {
        Self(format!("{}\n{}", self.0, other.0))
    }
}

#[derive(Debug, Clone)]
pub enum TfVar {
    Resource {
        resource_name: String,
        resource_identifier: String,
        field: String,
    },
    Data {
        data_name: String,
        data_identifier: String,
        field: String,
    },
}

#[derive(Debug, Clone)]
pub enum TfField {
    Variable(TfVar),
    String(String),
    List(Vec<TfField>),
    Map(HashMap<String, TfField>),
}

impl TfField {
    pub fn eq_prefix(&self) -> String {
        match self {
            Self::Variable(_) | Self::String(_) | Self::List(_) => "= ",
            Self::Map(_) => "",
        }
        .into()
    }

    pub fn to_tf_string(&self) -> String {
        match self {
            Self::String(s) => format!("\"{}\"", s),
            Self::Variable(v) => match v {
                TfVar::Resource {
                    resource_name,
                    resource_identifier,
                    field,
                } => {
                    format!("{resource_name}.{resource_identifier}.{field}",)
                }
                TfVar::Data {
                    data_name,
                    data_identifier,
                    field,
                } => {
                    format!("data.{data_name}.{data_identifier}.{field}",)
                }
            },
            Self::List(l) => {
                let list = l
                    .iter()
                    .map(|field| field.to_tf_string())
                    .collect::<Vec<String>>()
                    .join(", ");

                format!("[{list}]")
            }
            Self::Map(m) => {
                let map = m
                    .iter()
                    .map(|(key, field)| {
                        format!("{key} {}{}", field.eq_prefix(), field.to_tf_string())
                    })
                    .collect::<Vec<String>>()
                    .join("\n\t");

                format!("{{\n\t{map}\n}}")
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct TfResource {
    tf_type: String,
    resource_name: String,
    resource_identifier: String,
    fields: HashMap<String, TfField>,
}

impl TfResource {
    pub fn new_resource(
        resource_name: impl Into<String>,
        resource_identifier: impl Into<String>,
    ) -> Self {
        Self {
            resource_name: resource_name.into(),
            resource_identifier: resource_identifier.into(),
            fields: HashMap::new(),
            tf_type: "resource".into(),
        }
    }

    pub fn new_data(
        resource_name: impl Into<String>,
        resource_identifier: impl Into<String>,
    ) -> Self {
        Self {
            resource_name: resource_name.into(),
            resource_identifier: resource_identifier.into(),
            fields: HashMap::new(),
            tf_type: "data".into(),
        }
    }

    pub fn add_field(&mut self, field_name: &str, field_value: TfField) -> &mut Self {
        self.fields.insert(field_name.to_string(), field_value);

        self
    }

    pub fn create_terraform(&self) -> Terraform {
        let as_map = TfField::Map(self.fields.clone()).to_tf_string();

        let tf = format!(
            "{} \"{}\" \"{}\" {as_map}\n",
            self.tf_type, self.resource_name, self.resource_identifier
        );

        Terraform(tf)
    }
}
