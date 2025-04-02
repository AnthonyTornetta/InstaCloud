use std::collections::HashMap;

use derive_more::Display;

#[derive(Debug, Clone, Display, Default)]
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

impl TfVar {
    pub fn to_tf_string(&self) -> String {
        match self {
            Self::Resource {
                resource_name,
                resource_identifier,
                field,
            } => format!("{resource_name}.{resource_identifier}.{field}",),
            Self::Data {
                data_name,
                data_identifier,
                field,
            } => format!("data.{data_name}.{data_identifier}.{field}",),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TfField {
    Raw(String),
    Variable(TfVar),
    String(String),
    List(Vec<TfField>),
    Map(HashMap<String, TfField>),
}

impl TfField {
    pub fn eq_prefix(&self) -> String {
        match self {
            Self::Variable(_) | Self::String(_) | Self::List(_) | Self::Raw(_) => "= ",
            Self::Map(_) => "",
        }
        .into()
    }

    pub fn to_tf_string(&self) -> String {
        match self {
            Self::Raw(s) => s.clone(),
            Self::String(s) => format!("\"{}\"", s),
            Self::Variable(v) => v.to_tf_string(),
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

#[derive(Clone, Copy, Debug)]
pub enum TfDataType {
    Resource,
    Data,
}

pub trait TerraformEntity {
    fn var(&self, field: impl Into<String>) -> TfVar;
    fn tf_identifier(&self) -> String;
    fn tf_type() -> &'static str;
    fn data_type() -> TfDataType;
}

pub struct TfOutput {}

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

    pub fn depends_on<T: TerraformEntity>(&mut self, depends_on: &T) -> &mut Self {
        let var = match T::data_type() {
            TfDataType::Resource => format!("{}.{}", T::tf_type(), depends_on.tf_identifier()),
            TfDataType::Data => format!("data.{}.{}", T::tf_type(), depends_on.tf_identifier()),
        };

        if let Some(depends_on) = self.fields.get_mut("depends_on") {
            if let TfField::List(ref mut list) = depends_on {
                list.push(TfField::Raw(var));
            } else {
                panic!("Invalid state for depends_on. Expected a list, got: {depends_on:?}");
            }
        } else {
            self.add_field("depends_on", TfField::List(vec![TfField::Raw(var)]));
        }

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
