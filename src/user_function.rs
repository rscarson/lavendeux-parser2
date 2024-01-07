use std::fmt::Display;

use crate::{
    flatten_arguments, pest, required_argument, state::State, std_functions::Function, Error, Node,
    Value,
};
use polyvalue::ValueType;

#[derive(Debug, Clone)]
pub struct UserFunction {
    name: String,
    arguments: Vec<String>,
    body: String,
}

impl UserFunction {
    /// Creates a new user function
    pub fn new(name: &str, arguments: Vec<String>, body: String) -> Result<Self, Error> {
        let inst = Self {
            name: name.to_string(),
            arguments,
            body,
        };

        // Check if the function is valid
        inst.body()?;

        Ok(inst)
    }

    /// Returns the name of this function
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the arguments of this function
    pub fn arguments(&self) -> &[String] {
        &self.arguments
    }

    /// Returns the body of this function
    pub fn body(&self) -> Result<Node, Error> {
        pest::parse_input(&self.body, pest::Rule::TOPLEVEL_EXPRESSION)
    }

    /// Executes this function
    pub fn execute(&self, state: &mut State, arguments: Vec<Value>) -> Result<Value, Error> {
        // Create a new scope
        state.scope_into()?;

        // Set the arguments
        for (name, value) in self.arguments.iter().zip(arguments) {
            state.set_variable(name, value);
        }

        // Execute the body - this is checked in the constructor
        // so we can unwrap here
        let body_result = self.body().unwrap().get_value(state);
        state.scope_out();

        body_result
    }

    /// Converts this function into a standard function
    pub fn to_std_function(&self) -> Function {
        Function::new(
            &self.name,
            &self.arguments().to_vec().join(", "),
            "user-defined",
            self.arguments()
                .iter()
                .map(|s| required_argument!(s, ValueType::Any))
                .collect(),
            ValueType::Any,
            |state, args, _token, func| {
                if let Some(function) = state.get_user_function(func) {
                    function.execute(state, flatten_arguments!(args, function.arguments))
                } else {
                    Err(Error::FunctionName { name: func.clone() })
                }
            },
            self.name.clone(),
        )
    }
}

impl Display for UserFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}({}) = {}",
            self.name(),
            self.arguments().join(", "),
            self.body
        )
    }
}
