use std::rc::Rc;

use crate::{error::ErrorDetails, syntax_tree::Node, Error, State};
use polyvalue::{Value, ValueType};

use super::{
    compiler_cache::cached_fn_compile,
    std_function::{FunctionArgument, FunctionArgumentType, FunctionDocumentation, ParserFunction},
};

#[derive(Debug, Clone)]
pub struct UserDefinedFunction {
    name: String,
    args: Vec<(String, ValueType)>,
    returns: ValueType,
    body: Vec<String>,

    src_line_offset: usize,
}
impl ParserFunction for UserDefinedFunction {
    fn name(&self) -> &str {
        &self.name
    }

    fn documentation(&self) -> &FunctionDocumentation {
        &FunctionDocumentation {
            category: "User-Defined Functions",
            description: None,
            ext_description: None,
            examples: None,
        }
    }

    fn return_type(&self) -> ValueType {
        self.returns
    }

    fn expected_arguments<'a>(&'a self) -> Vec<(&'a str, FunctionArgument)> {
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
        Box::new(self.clone())
    }

    fn call(&self, state: &mut State) -> Result<Value, Error> {
        // Execute the body - this is checked in the constructor
        // so we can unwrap here
        for node in self.body().iter().take(self.body().len() - 1) {
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
        let body = self.body();
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

impl UserDefinedFunction {
    pub fn new(name: &str, body: Vec<String>) -> Result<Self, Error> {
        // Check that the function is valid
        if body.is_empty() {
            /* Should be caught by the grammar */
            return oops!(Internal {
                msg: "User function must have at least one line".to_string()
            });
        }
        body.iter()
            .map(|l| cached_fn_compile(l, 0))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(UserDefinedFunction {
            name: name.to_string(),
            args: vec![],
            returns: ValueType::Any,
            body,
            src_line_offset: 0,
        })
    }

    pub fn add_arg(&mut self, name: &str, t: ValueType) {
        self.args.push((name.to_string(), t));
    }

    pub fn set_returns(&mut self, t: ValueType) {
        self.returns = t;
    }

    pub fn set_src_line_offset(&mut self, offset: usize) {
        self.src_line_offset = offset;
    }

    /// Returns the body of this function
    pub fn body(&self) -> Vec<Rc<Node>> {
        self.body
            .iter()
            .map(|s| cached_fn_compile(s, 0).unwrap())
            .collect()
    }
}
