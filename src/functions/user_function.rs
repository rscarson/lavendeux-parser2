use crate::{error::ErrorDetails, syntax_tree::Node, Error, Rule, State};
use polyvalue::{Value, ValueType};

use super::{
    documentation::UserFunctionDocumentation,
    std_function::{FunctionArgument, FunctionArgumentType, ParserFunction},
    FunctionDocumentation,
};

/// A user-defined function
/// This is a function defined in lavendish, and is not a part of the standard library
#[derive(Debug)]
pub struct UserDefinedFunction {
    name: String,
    args: Vec<(String, ValueType)>,
    returns: ValueType,
    src: Vec<String>,
    // body: Vec<Node<'i>>,
    src_line_offset: usize,

    own_docs: UserFunctionDocumentation,
}
impl<'i> ParserFunction for UserDefinedFunction {
    fn name(&self) -> &str {
        &self.name
    }

    fn documentation(&self) -> &dyn FunctionDocumentation {
        &self.own_docs
    }

    fn documentation_mut(&mut self) -> &mut dyn FunctionDocumentation {
        &mut self.own_docs
    }

    fn return_type(&self) -> ValueType {
        self.returns
    }

    fn expected_arguments(&self) -> Vec<(&str, FunctionArgument)> {
        // map self.args to FunctionArgument Standard/All
        self.args
            .iter()
            .map(|(name, expects)| {
                (
                    name.as_str(),
                    FunctionArgument {
                        expected_type: *expects,
                        meta: FunctionArgumentType::Standard,
                    },
                )
            })
            .collect::<Vec<(&str, FunctionArgument)>>()
    }

    fn clone_self(&self) -> Box<dyn ParserFunction> {
        Box::new(Self {
            name: self.name.clone(),
            args: self.args.clone(),
            returns: self.returns,
            src: self.src.clone(),
            // body: Self::compile(&self.src).unwrap(),
            src_line_offset: self.src_line_offset,

            own_docs: self.own_docs.clone(),
        })
    }

    fn call(&self, state: &mut State) -> Result<Value, Error<'_>> {
        // Execute the body - this is checked in the constructor
        // so we can unwrap here
        let body = Self::compile(&self.src)?;
        for node in body.iter().take(body.len() - 1) {
            match node.get_value(state) {
                Ok(_) => {}
                Err(e) if matches!(e.details, ErrorDetails::Return { .. }) => {
                    if let ErrorDetails::Return { value } = e.details {
                        return Ok(value.as_type(self.returns)?);
                    }
                }
                Err(e) => {
                    let e = e.offset_linecount(self.src_line_offset);
                    return Err(e);
                }
            }
        }

        // Execute the last node
        let body = body.iter().last().unwrap();
        match body.get_value(state) {
            Ok(v) => Ok(v.as_type(self.returns)?),
            Err(e) => {
                let e = e.offset_linecount(self.src_line_offset);
                Err(e)
            }
        }
    }
}

impl<'i> UserDefinedFunction {
    /// Create a new user-defined function
    pub fn new(name: &'i str, src: Vec<String>) -> Result<Self, Error<'i>> {
        // Check that the function is valid
        if src.is_empty() {
            return oops!(Internal {
                msg: "Functions must have at least one line".to_string()
            });
        }

        Self::compile(&src)?;
        Ok(UserDefinedFunction {
            name: name.to_string(),
            args: vec![],
            returns: ValueType::Any,
            //    body,
            src,
            src_line_offset: 0,
            own_docs: UserFunctionDocumentation {
                category: "User-Defined Functions".to_string(),
                description: None,
                ext_description: None,
                examples: None,
            },
        })
    }

    pub fn compile(src: &'i Vec<String>) -> Result<Vec<Node<'i>>, Error<'i>> {
        src.iter()
            .map(|l| crate::pest::parse_input(l, Rule::EXPR))
            .collect::<Result<Vec<_>, _>>()
    }

    /// Add a required argument to the function
    pub fn add_arg(&mut self, name: &str, t: ValueType) {
        self.args.push((name.to_string(), t));
    }

    /// Set the return type of the function
    pub fn set_returns(&mut self, t: ValueType) {
        self.returns = t;
    }

    /// Offset the location in source-code for errors
    pub fn set_src_line_offset(&mut self, offset: usize) {
        self.src_line_offset = offset;
    }
}
