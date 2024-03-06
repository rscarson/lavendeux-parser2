use std::borrow::Cow;

use crate::{
    error::ErrorDetails,
    pest::LavendeuxParser,
    syntax_tree::{
        traits::{IntoOwned, NodeExt},
        Node, Reference,
    },
    Error, Rule, State,
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
    src: Vec<String>,
    body: Vec<Node<'i>>,

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
            body: UserDefinedFunction::compile(&self.src, &mut Default::default()).unwrap(),

            src_line_offset: self.src_line_offset,

            own_docs: self.own_docs.clone(),
        })
    }

    fn call(&self, state: &mut State, _: Option<&Reference>) -> Result<Value, Error> {
        // Execute the body - this is checked in the constructor
        // so we can unwrap here
        for node in self.body.iter().take(self.body.len() - 1) {
            match node.evaluate(state) {
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
        let body = &self.body;
        let body = body.iter().last().unwrap();
        match body.evaluate(state) {
            Ok(v) => Ok(v.as_type(self.returns)?),
            Err(e) => {
                let e = e.offset_linecount(self.src_line_offset);
                Err(e)
            }
        }
    }
}

impl UserDefinedFunction<'_> {
    /// Create a new user-defined function
    pub fn new<'i>(name: &'i str, src: Vec<String>, state: &mut State) -> Result<Self, Error> {
        // Check that the function is valid
        if src.is_empty() {
            /* Should be caught by the grammar */
            return oops!(Internal {
                msg: "User function must have at least one line".to_string()
            });
        }

        // to get rid of this, there needs to be a way to get a Node that owns its text,
        // or maybe something yoke-y
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

    fn compile(src: &[String], state: &mut State) -> Result<Vec<Node<'static>>, Error> {
        src.iter()
            .map(|l| {
                LavendeuxParser::build_ast(l, Rule::EXPR, state).and_then(|n| Ok(n.into_owned()))
            })
            .collect()
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
    pub fn src(&self) -> &Vec<String> {
        &self.src
    }

    /// Remove the lifetime from the function
    pub fn into_owned(self) -> UserDefinedFunction<'static> {
        UserDefinedFunction {
            name: self.name,
            args: self.args,
            returns: self.returns,
            body: self.body.into_iter().map(|n| n.into_owned()).collect(),
            src: self.src,
            src_line_offset: self.src_line_offset,
            own_docs: self.own_docs,
        }
    }
}
