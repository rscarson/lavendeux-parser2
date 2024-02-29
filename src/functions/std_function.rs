use crate::{Error, State};
use polyvalue::{Value, ValueType};

use super::FunctionDocumentation;

/// A function argument type
#[derive(Debug, Copy, Clone)]
pub enum FunctionArgumentType {
    /// Normal argument
    Standard,
    /// 0-or-more of
    Plural,
    /// 0-or-1 of
    Optional,
}

/// A function argument
#[derive(Debug, Copy, Clone)]
pub struct FunctionArgument {
    /// Type condition to enforce
    pub expected_type: ValueType,

    /// How to parse the argument
    pub meta: FunctionArgumentType,
}

impl FunctionArgument {
    /// Returns true if the argument can be skipped on type errors or missing values
    pub fn is_optional(&self) -> bool {
        !matches!(self.meta, FunctionArgumentType::Standard)
    }

    /// Returns true if the argument should consume 0 or more args until a non-matching type is found
    pub fn is_plural(&self) -> bool {
        matches!(self.meta, FunctionArgumentType::Plural)
    }
}

pub trait ManageArguments {
    fn arg_count_span(&self) -> (usize, usize);
    fn map_arguments(
        &self,
        values: &[Value],
        state: &mut State,
        function_signature: String,
    ) -> Result<(), Error>;
}
impl ManageArguments for Vec<(&str, FunctionArgument)> {
    fn arg_count_span(&self) -> (usize, usize) {
        let (mut min, mut max) = (0, 0);
        for (_, arg) in self.iter() {
            if !arg.is_optional() {
                min += 1;
            }
            max += 1;
        }
        (min, max)
    }

    fn map_arguments(
        &self,
        values: &[Value],
        state: &mut State,
        function_signature: String,
    ) -> Result<(), Error> {
        let mut values = values.into_iter().peekable();

        for (i, (name, arg)) in self.iter().enumerate() {
            let next = values.next();
            if next.is_none() && !arg.is_optional() {
                let span = self.arg_count_span();
                return oops!(FunctionArguments {
                    min: span.0,
                    max: span.1,
                    signature: function_signature
                });
            } else if next.is_none() {
                continue;
            }

            let next = next.unwrap().clone().as_type(arg.expected_type);
            if next.is_err() {
                if arg.is_optional() {
                    continue;
                } else {
                    return oops!(FunctionArgumentType {
                        arg: i + 1,
                        expected_type: arg.expected_type,
                        signature: function_signature
                    });
                }
            }
            let next = next.unwrap();

            if arg.is_plural() {
                let mut matches = Vec::new();
                matches.push(next);
                while let Some(next) = values.peek() {
                    if next.is_a(arg.expected_type) {
                        matches.push(values.next().unwrap().clone());
                    } else {
                        break;
                    }
                }
                state.set_variable(name, Value::array(matches));
            } else {
                state.set_variable(name, next);
            }
        }

        if values.next().is_some() {
            let span = self.arg_count_span();
            return oops!(FunctionArguments {
                min: span.0,
                max: span.1,
                signature: function_signature
            });
        }

        Ok(())
    }
}

/// Object trait used for parser functions
pub trait ParserFunction
where
    Self: Send + Sync + std::fmt::Debug,
{
    /// Name of the function
    fn name(&self) -> &str;

    /// Return type of the function
    fn return_type(&self) -> ValueType;

    /// Expected arguments for the function
    fn expected_arguments(&self) -> Vec<(&str, FunctionArgument)>;

    /// Clones the function
    fn clone_self(&self) -> Box<dyn ParserFunction>;

    /// Identifies system functions that should not be overridden by user functions
    fn is_readonly(&self) -> bool {
        false
    }

    /// Documentation for the function
    fn documentation(&self) -> &dyn FunctionDocumentation;

    /// Mutable version of documentation
    fn documentation_mut(&mut self) -> &mut dyn FunctionDocumentation;

    /// Call the function's handler - use exec instead to map arguments first
    fn call(&self, state: &mut State) -> Result<Value, Error>;

    /// Loads the arguments into the state
    fn load_arguments(&self, values: &[Value], state: &mut State) -> Result<(), Error> {
        match self
            .expected_arguments()
            .map_arguments(values, state, self.signature())
        {
            Ok(_) => Ok(()),
            Err(e) => {
                state.scope_out();
                Err(e)
            }
        }
    }

    /// Returns the function signature
    fn signature(&self) -> String {
        format!(
            "{}({}) -> {}",
            self.name(),
            self.expected_arguments()
                .iter()
                .map(|(name, arg)| {
                    let type_name = if arg.expected_type == ValueType::Any {
                        "".to_string()
                    } else {
                        format!(":{}", arg.expected_type)
                    };
                    (if arg.is_optional() {
                        format!("[{}{}]", name, type_name)
                    } else {
                        format!("{}{}", name, type_name)
                    } + if arg.is_plural() { ", ..." } else { "" })
                })
                .collect::<Vec<String>>()
                .join(", "),
            self.return_type(),
        )
    }

    /// Executes the function with the given values
    /// Values are checked mapped into the state into a new scope
    /// arg1_references is used to add a pass-by-reference flag to the first argument
    fn exec(
        &self,
        values: &[Value],
        state: &mut State,
        arg1_references: Option<&str>,
    ) -> Result<Value, Error> {
        state.scope_into()?;
        state.lock_scope();
        self.load_arguments(values, state)?;

        // Mostly for array functions
        if let Some(reference) = arg1_references {
            state.set_variable("__flag_arg1_reference", Value::string(reference))
        }

        let result = self.call(state);
        state.scope_out();

        result
    }
}
