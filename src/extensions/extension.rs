use polyvalue::{Value, ValueType};
use rustyscript::{ModuleHandle, Runtime};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{error::WrapExternalError, Error, Token};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct FunctionDefinition {
    name: String,
    description: String,
    arguments: Vec<ValueType>,
    returns: ValueType,
}

impl FunctionDefinition {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn arguments(&self) -> &[ValueType] {
        &self.arguments
    }

    pub fn returns(&self) -> &ValueType {
        &self.returns
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn signature(&self) -> String {
        if self.name.starts_with('@') {
            format!("<{}> @{}", self.arguments[0], self.name)
        } else {
            let str_args = self
                .arguments
                .iter()
                .map(|a| a.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            format!("{}({}) -> {}", self.name, str_args, self.returns)
        }
    }

    pub fn call(
        &self,
        runtime: &mut Runtime,
        handle: &ModuleHandle,
        args: &[Value],
        variables: &mut HashMap<String, Value>,
        token: &Token,
    ) -> Result<Value, Error> {
        if args.len() < self.arguments.len() {
            return Err(Error::FunctionArguments {
                min: self.arguments.len(),
                max: self.arguments.len(),
                signature: self.signature(),
                token: token.clone(),
            });
        }

        for (i, arg) in self.arguments.iter().enumerate() {
            let actual_type = &args[i];
            if !actual_type.is_a(*arg) {
                return Err(Error::FunctionArgumentType {
                    arg: i + 1,
                    expected_type: *arg,
                    signature: self.signature(),
                    token: token.clone(),
                });
            }
        }

        // Fixed and currency types will be passed as floats, so we need to convert them
        let args = args
            .iter()
            .map(|v| {
                if v.is_a(ValueType::Fixed) || v.is_a(ValueType::Currency) {
                    v.as_type(ValueType::Float).unwrap()
                } else {
                    v.clone()
                }
            })
            .collect::<Vec<Value>>();

        // Inject parser state
        runtime
            .call_function(
                handle,
                "saveState",
                &[serde_json::to_value(variables.clone()).with_context(token)?],
            )
            .with_context(token)?;

        let mut args = args
            .iter()
            .map(|v| serde_json::to_value(v.clone()))
            .collect::<Result<Vec<serde_json::Value>, _>>()
            .with_context(token)?;
        args.insert(0, self.name.clone().into());
        let result: Value = runtime
            .call_function(handle, "callLavendeuxFunction", &args)
            .with_context(token)?;

        // Extract parser state
        let variables_out: HashMap<String, Value> = runtime
            .call_function(handle, "loadState", &[])
            .with_context(token)?;
        for (key, value) in variables_out {
            variables.insert(key, value);
        }

        result.as_type(self.returns).with_context(token)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ExtensionDetails {
    name: String,
    author: String,
    version: String,
    functions: HashMap<String, FunctionDefinition>,
}

impl ExtensionDetails {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn author(&self) -> &str {
        &self.author
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn signature(&self) -> String {
        format!("{} v{} by {}", self.name, self.version, self.author)
    }

    pub fn all_functions(&self) -> &HashMap<String, FunctionDefinition> {
        &self.functions
    }

    pub fn function_names(&self) -> Vec<String> {
        self.functions.keys().cloned().collect()
    }

    pub fn call_function(
        &self,
        runtime: &mut Runtime,
        handle: &ModuleHandle,
        name: &str,
        args: &[Value],
        variables: &mut HashMap<String, Value>,
        token: &Token,
    ) -> Result<Value, Error> {
        let function = self.functions.get(name).ok_or(Error::FunctionName {
            name: name.to_string(),
            token: token.clone(),
        })?;

        function.call(runtime, handle, args, variables, token)
    }
}

#[cfg(test)]
mod test {
    use rustyscript::Module;

    use super::super::runtime::ExtensionRuntime;
    use super::*;

    #[test]
    fn test_load_simple() {
        let module = Module::load("example_extensions/simple_extension.js").unwrap();
        let mut runtime = ExtensionRuntime::new(module).unwrap();
        assert_eq!(runtime.extension_details().name(), "Simple Extension");
        assert_eq!(runtime.extension_details().author(), "@rscarson");
        assert_eq!(runtime.extension_details().version(), "1.0.0");
        assert_eq!(runtime.extension_details().function_names().len(), 2);

        let mut variables = HashMap::new();

        let result = runtime
            .call_function(
                "add",
                &[super::Value::from(1.0), super::Value::from(2.0)],
                &mut variables,
                &Token::dummy(),
            )
            .unwrap();
        assert_eq!(result, Value::from(3i64));

        let result = runtime
            .call_function(
                "@colour",
                &[super::Value::from(0xFF)],
                &mut variables,
                &Token::dummy(),
            )
            .unwrap();
        assert_eq!(result, Value::from("#ff0000"));
    }

    #[test]
    fn test_load_stateful() {
        let module = Module::load("example_extensions/stateful_functions.js").unwrap();
        let mut runtime = ExtensionRuntime::new(module).unwrap();
        assert_eq!(runtime.extension_details().name(), "Stateful Extension");
        assert_eq!(runtime.extension_details().author(), "@rscarson");
        assert_eq!(runtime.extension_details().version(), "1.0.0");
        assert_eq!(runtime.extension_details().function_names().len(), 2);

        let mut variables = HashMap::new();

        runtime
            .call_function(
                "put",
                &[super::Value::from("foo"), super::Value::from(2.1)],
                &mut variables,
                &Token::dummy(),
            )
            .unwrap();

        assert_eq!(
            variables.get("foo"),
            Some(&Value::from(2.1)),
            "put should set a variable",
        );

        let result = runtime
            .call_function(
                "get",
                &[super::Value::from("foo")],
                &mut variables,
                &Token::dummy(),
            )
            .unwrap();
        assert_eq!(result, Value::from(2.1), "get should return the variable");
    }
}
