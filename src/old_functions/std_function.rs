use std::{collections::HashMap, fmt::Debug, ops::Range};

use crate::{oops, state::State, Error, Token, Value};
use polyvalue::ValueType;

#[derive(Debug, Clone)]
pub struct Argument {
    pub name: String,
    pub expects: ValueType,
    pub optional: bool,
    pub plural: bool,
}

pub type FunctionHandler =
    fn(&mut State, HashMap<String, Vec<Value>>, &Token, &String) -> Result<Value, Error>;

#[derive(Clone)]
pub struct Function {
    name: String,
    description: String,
    category: String,

    arguments: Vec<Argument>,
    returns: ValueType,
    handler: FunctionHandler,

    /// Arbitrary data that can be associated with this function
    /// This is useful for extensions, and user-defined functions
    data: String,
}

impl Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Function")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("category", &self.category)
            .field("arguments", &self.arguments)
            .field("returns", &self.returns)
            .field("data", &self.data)
            .finish()
    }
}

impl Function {
    /// Creates a new function
    pub fn new(
        name: &str,
        description: &str,
        category: &str,
        arguments: Vec<Argument>,
        returns: ValueType,
        handler: FunctionHandler,
        data: String,
    ) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            category: category.to_string(),
            arguments,
            returns,
            handler,
            data,
        }
    }

    /// Returns the name of this function
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the description of this function
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Returns the category of this function
    pub fn category(&self) -> &str {
        &self.category
    }

    /// Returns the arguments of this function
    pub fn arguments(&self) -> &[Argument] {
        &self.arguments
    }

    /// Returns the range of arguments that are required for this function.
    pub fn argument_span(&self) -> Range<usize> {
        let mut min = 0;
        let mut max = 0;

        for argument in self.arguments() {
            max += 1;
            if !argument.optional {
                min += 1;
            }
        }

        min..max
    }

    /// Returns the return type of this function
    pub fn returns(&self) -> ValueType {
        self.returns
    }

    /// Return the function's signature
    pub fn signature(&self) -> String {
        format!(
            "{}({}) -> {}",
            self.name,
            self.arguments()
                .iter()
                .map(|e| e.expects.to_string())
                .collect::<Vec<String>>()
                .join(", "),
            self.returns
        )
    }

    /// Returns the function's handler
    pub fn handler(&self) -> FunctionHandler {
        self.handler
    }

    /// Maps the given arguments to their names
    /// while validating the arguments against this function's signature
    pub fn map_arguments(
        &self,
        arguments: Vec<Value>,
        token: &Token,
    ) -> Result<HashMap<String, Vec<Value>>, Error> {
        let mut validated: HashMap<String, Vec<Value>> = HashMap::new();

        let mut arg_iter = self.arguments.iter().peekable();
        let mut in_iter = arguments.iter().peekable();
        let mut arg_index = 0;
        while arg_iter.peek().is_some() {
            let next_expected = arg_iter.next().unwrap();

            if next_expected.plural {
                let mut values = vec![];
                while in_iter.peek().is_some() {
                    let next_argument = in_iter.next().unwrap();
                    arg_index += 1;
                    if let Ok(value) = next_argument.as_type(next_expected.expects) {
                        values.push(value);
                    } else {
                        return oops!(
                            FunctionArgumentType {
                                arg: arg_index,
                                expected_type: next_expected.expects,
                                signature: self.signature()
                            },
                            token.clone()
                        );
                    }
                }
                validated.insert(next_expected.name.clone(), values);
                break;
            }

            if let Some(next_argument) = in_iter.next() {
                arg_index += 1;
                if let Ok(value) = next_argument.as_type(next_expected.expects) {
                    validated.insert(next_expected.name.clone(), vec![value]);
                } else if next_expected.optional {
                    continue;
                } else {
                    return oops!(
                        FunctionArgumentType {
                            arg: arg_index,
                            expected_type: next_expected.expects,
                            signature: self.signature()
                        },
                        token.clone()
                    );
                }
            } else if next_expected.optional {
                continue;
            } else {
                let s = self.argument_span();
                return oops!(
                    FunctionArguments {
                        min: s.start,
                        max: s.end,
                        signature: self.signature()
                    },
                    token.clone()
                );
            }
        }

        Ok(validated)
    }

    /// Executes this function
    pub fn execute(
        &self,
        state: &mut State,
        arguments: Vec<Value>,
        token: &Token,
    ) -> Result<Value, Error> {
        let arguments = self.map_arguments(arguments, token)?;
        (self.handler)(state, arguments, token, &self.data)
    }

    /// Register this function in the given map
    pub fn register(&self, map: &mut HashMap<String, Function>) {
        map.insert(self.name.clone(), self.clone());
    }
}
