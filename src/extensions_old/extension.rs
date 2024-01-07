use super::js_extension::{JsExtension, JsExtensionFunction};
use polyvalue::ValueType;
use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExtensionFunction {
    pub name: String,
    pub arguments: Vec<ValueType>,
    pub returns: ValueType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExtensionDecorator {
    pub name: String,
    pub argument: ValueType,
}

impl From<ExtensionDecorator> for ExtensionFunction {
    fn from(decorator: ExtensionDecorator) -> Self {
        Self {
            name: decorator.name,
            arguments: vec![decorator.argument],
            returns: ValueType::String,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Extension {
    pub name: String,
    pub author: String,
    pub version: String,
    pub functions: HashMap<String, ExtensionFunction>,
    pub decorators: HashMap<String, ExtensionDecorator>,
}

/// JsExtension is the type that is deserialized from js
/// It exists to support the legacy format of extension
/// Here we convert both types into the new format
impl From<JsExtension> for Extension {
    fn from(js_extension: JsExtension) -> Self {
        let mut instance = Self {
            name: js_extension.name.clone(),
            author: js_extension.author.clone(),
            version: js_extension.version.clone(),
            functions: HashMap::new(),
            decorators: HashMap::new(),
        };

        for (name, function) in js_extension.functions {
            match function {
                JsExtensionFunction::Legacy(_) => {
                    instance.functions.insert(
                        name.clone(),
                        ExtensionFunction {
                            name,
                            arguments: vec![ValueType::Any],
                            returns: ValueType::Any,
                        },
                    );
                }

                JsExtensionFunction::Standard(function) => {
                    instance.functions.insert(
                        name.clone(),
                        ExtensionFunction {
                            name,
                            arguments: function
                                .arguments
                                .iter()
                                .map(|a| ValueType::try_from(a.as_str()).unwrap_or(ValueType::Any))
                                .collect(),
                            returns: ValueType::try_from(function.returns.as_str())
                                .unwrap_or(ValueType::Any),
                        },
                    );
                }
            }
        }

        instance
    }
}
