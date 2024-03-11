use std::borrow::Cow;

use crate::{
    error::ErrorDetails,
    syntax_tree::{
        traits::{IntoOwned, NodeExt},
        Node,
    },
    Error, Lavendeux, Rule, State,
};
use polyvalue::{Value, ValueType};

use super::{
    documentation::UserFunctionDocumentation,
    std_function::{FunctionArgument, FunctionArgumentType, ParserFunction},
    FunctionDocumentation,
};

/// A user-defined function
/// This is a function defined in lavendish, and is not a part of the standard library
#[derive(Debug, Clone)]
pub struct UserDefinedFunction<'i> {
    name: String,
    args: Vec<(String, ValueType)>,
    returns: ValueType,
    src: String,
    body: Node<'i>,

    src_line_offset: usize,

    own_docs: UserFunctionDocumentation,
}
impl ParserFunction for UserDefinedFunction<'_> {
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

    fn expected_arguments(&self) -> Vec<(Cow<'static, str>, FunctionArgument)> {
        // map self.args to FunctionArgument Standard/All
        self.args
            .iter()
            .map(|(name, expects)| {
                (
                    Cow::Owned(name.clone()),
                    FunctionArgument {
                        expected_type: *expects,
                        meta: FunctionArgumentType::Standard,
                    },
                )
            })
            .collect()
    }

    fn clone_self(&self) -> Box<dyn ParserFunction> {
        Box::new(UserDefinedFunction {
            name: self.name.clone(),
            args: self.args.clone(),
            returns: self.returns,
            src: self.src.clone(),
            body: UserDefinedFunction::compile(&self.src, &mut Default::default()).unwrap(), // This is safe because the function is already checked

            src_line_offset: self.src_line_offset,

            own_docs: self.own_docs.clone(),
        })
    }

    fn call(&self, state: &mut State) -> Result<Value, Error> {
        // Execute the body - this is checked in the constructor
        // so we can unwrap here
        match self.body.evaluate(state) {
            Ok(v) => Ok(v.as_type(self.returns)?),
            Err(e) => {
                if let ErrorDetails::Return { value } = e.details {
                    return Ok(value.as_type(self.returns)?);
                } else {
                    let e = e.offset_linecount(self.src_line_offset);
                    return Err(e);
                }
            }
        }
    }
}

impl UserDefinedFunction<'_> {
    /// Create a new user-defined function
    pub fn new(name: &str, src: String, state: &mut State) -> Result<Self, Error> {
        let body = Self::compile(&src, state)?;
        Ok(UserDefinedFunction {
            name: name.to_string(),
            args: vec![],
            returns: ValueType::Any,
            body,
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

    fn compile(src: &str, state: &mut State) -> Result<Node<'static>, Error> {
        Lavendeux::eval_rule(src, state, Rule::BLOCK).map(|n| n.into_owned())
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

    /// Get the source code of the function
    pub fn src(&self) -> &str {
        &self.src
    }

    /// Remove the lifetime from the function
    pub fn into_owned(self) -> UserDefinedFunction<'static> {
        UserDefinedFunction {
            name: self.name,
            args: self.args,
            returns: self.returns,
            body: self.body.into_owned(),
            src: self.src,
            src_line_offset: self.src_line_offset,
            own_docs: self.own_docs,
        }
    }
}
