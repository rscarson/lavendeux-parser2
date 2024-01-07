//! Function Nodes
//!
//! Node types for function calls
//!
use super::*;
use crate::{Error, Rule, State, ToToken, Value};
use pest::iterators::Pair;

define_node!(
    FunctionCall {
        name: String,
        arguments: Vec<Node>
    },
    rules = [FUNCTION_CALL_EXPRESSION],

    new = |input:Pair<Rule>| {
        let mut token = input.to_token();
        let mut children = input.into_inner();

        let name = children.next().unwrap().as_str().to_string();
        children.next().unwrap(); // skip marker
        let arguments = if name == "help" {
            //
            // Help function takes in an identifier as an argument
            // and returns the help text for that function / category / etc.
            // But we need to prevent lookups of the identifier
            children
            .map(|child| {
                let ident = Identifier::from_pair(child)?;
                let value_lit = ValueLiteral::new(Value::from(ident.token().input.clone()), ident.token().clone());
                Ok(value_lit.boxed())
            })
            .collect::<Result<Vec<Node>, Error>>()?
        } else {
            //
            // Other functions take in expressions as arguments
            children
            .map(|child| Ok(child.to_ast_node()?))
            .collect::<Result<Vec<Node>, Error>>()?
        };

        // Function arguments can have variable references to the first argument in order to
        // update values. This is especially useful for array functions like push, pop, etc.
        let effective_reference = arguments.iter().next().and_then(|c| c.token().references.clone());
        token.references = effective_reference;

        Ok(Self {
            name,
            arguments,
            token
        }.boxed())
    },

    value = |call: &FunctionCall, state: &mut State| {
        if call.name == "help" {
            let filter = if let Some(s) = call.arguments.last() {
                Some(s.get_value(state)?.to_string())
            } else {
                None
            };

            return Ok(Value::from(state.help(filter)));
        }

        let function = state.get_function(&call.name).ok_or(Error::FunctionName { name:
            call.name.clone()
        })?;

        // Collect arguments
        let mut arguments = Vec::new();
        for argument in call.arguments.iter() {
            arguments.push(argument.get_value(state)?);
        }

        function.execute(state, arguments, call.token())
    }
);
