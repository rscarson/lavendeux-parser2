use serde::{Deserialize, Serialize};
use std::collections::HashMap;

fn default_name() -> String {
    "Unnamed Extension".to_string()
}
fn default_author() -> String {
    "Anonymous".to_string()
}
fn default_version() -> String {
    "0.0.0".to_string()
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct JsExtensionFunctionDefinition {
    pub arguments: Vec<String>,
    pub returns: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum JsExtensionFunction {
    Legacy(String),
    Standard(JsExtensionFunctionDefinition),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct JsExtension {
    #[serde(default = "default_name")]
    pub name: String,

    #[serde(default = "default_author")]
    pub author: String,

    #[serde(default = "default_version")]
    pub version: String,

    #[serde(default)]
    pub functions: HashMap<String, JsExtensionFunction>,

    #[serde(default)]
    pub decorators: HashMap<String, JsExtensionFunction>,
}
