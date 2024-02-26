use crate::{oops, Error, State};
use polyvalue::{Value, ValueType};

#[derive(Debug, Copy, Clone)]
pub enum FunctionArgumentType {
    Standard,
    Plural,
    Optional,
}

#[derive(Debug, Copy, Clone)]
pub struct FunctionArgument {
    pub expected_type: ValueType,
    pub meta: FunctionArgumentType,
}

impl FunctionArgument {
    pub fn is_optional(&self) -> bool {
        !matches!(self.meta, FunctionArgumentType::Standard)
    }

    pub fn is_plural(&self) -> bool {
        matches!(self.meta, FunctionArgumentType::Plural)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct FunctionDocumentation {
    pub category: &'static str,

    pub description: Option<&'static str>,

    pub ext_description: Option<&'static str>,
    pub examples: Option<&'static str>,
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

pub trait ParserFunction
where
    Self: Send + Sync + std::fmt::Debug,
{
    fn name<'a>(&'a self) -> &'a str;
    fn return_type(&self) -> ValueType;
    fn expected_arguments<'a>(&'a self) -> Vec<(&'a str, FunctionArgument)>;

    fn clone_self(&self) -> Box<dyn ParserFunction>;

    /// Identifies system functions that should not be overridden by user functions
    fn is_readonly(&self) -> bool {
        false
    }

    fn documentation(&self) -> &FunctionDocumentation;

    fn call(&self, state: &mut State) -> Result<Value, Error>;

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
