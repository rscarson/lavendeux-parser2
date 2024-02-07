//! Function Nodes
//!
//! Node types for function calls
//!
use super::*;
use crate::{pest::parse_input, user_function::UserFunction, Error, Rule, State, ToToken, Value};
use pest::iterators::Pair;

define_node!(
    FunctionCall {
        name: String,
        arguments: Vec<Node>
    },
    rules = [FUNCTION_CALL_EXPRESSION, POSTFIX_FUNCTION_CALL_EXPRESSION],

    new = |input:Pair<Rule>| {
        let rule = input.as_rule();
        match rule {
            Rule::FUNCTION_CALL_EXPRESSION => {
                let mut token = input.to_token();
                let mut children = input.into_inner();

                let name = children.next().unwrap().as_str().to_string();
                children.next().unwrap(); // skip marker

                let arguments = children
                .map(|child| Ok(child.to_ast_node()?))
                .collect::<Result<Vec<Node>, Error>>()?;

                // Function arguments can have variable references to the first argument in order to
                // update values. This is especially useful for array functions like push, pop, etc.
                let effective_reference = arguments.iter().next().and_then(|c| c.token().references.clone());
                token.references = effective_reference;

                Ok(Self { name, arguments, token }.boxed())
            },

            Rule::POSTFIX_FUNCTION_CALL_EXPRESSION => {
                let mut token = input.to_token();
                let mut children = input.into_inner();

                let mut arguments = vec![children.next().unwrap().to_ast_node()?];
                let name = children.next().unwrap().as_str().to_string();
                for child in children {
                    arguments.push(child.to_ast_node()?);
                }

                // Function arguments can have variable references to the first argument in order to
                // update values. This is especially useful for array functions like push, pop, etc.
                let effective_reference = arguments.iter().next().and_then(|c| c.token().references.clone());
                token.references = effective_reference;

                Ok(Self { name, arguments, token }.boxed())
            },
            _ => unreachable!("Grammar issue: unexpected rule for FunctionCall node")
        }
    },

    value = |call: &FunctionCall, state: &mut State| {
        if &call.name == "help" {
            let filter = call.arguments.get(0);
            let filter = filter.as_ref().map(|f| Ok::<_, Error>(f.get_value(state)?.to_string())).transpose()?;
            let help_text = state.help(filter);
            return Ok(Value::from(help_text));
        }

        let function = state.get_function(&call.name).ok_or(Error::FunctionName {
            name: call.name.clone(),
            token: call.token().clone()
        })?;

        // Collect arguments
        let mut arguments = Vec::new();
        for argument in call.arguments.iter() {
            arguments.push(argument.get_value(state)?);
        }

        match function.execute(state, arguments, call.token()) {
            Ok(value) => Ok(value),
            Err(Error::Return{ value, ..}) => Ok(value),
            Err(e) => {
                let token = call.token();
                Err(Error::FunctionCall {
                    name: call.name.clone(),
                    token: token.clone(),
                    source: Box::new(e)
                })
            }
        }
    }
);

// identifier ~ "(" ~ ")" ~ "=" ~ TOPLEVEL_EXPRESSION |
// identifier ~ "(" ~ identifier ~ ("," ~ identifier)* ~ ")" ~ "=" ~ TOPLEVEL_EXPRESSION
define_node!(
    FunctionAssignment {
        name: String,
        arguments: Vec<String>,
        expressions: Vec<String>
    },
    rules = [FUNCTION_ASSIGNMENT_STATEMENT],

    new = |input:Pair<Rule>| {
        let token = input.to_token();
        let mut children = input.into_inner();

        // Name of the function is the first child
        let name = children.next().unwrap().as_str().to_string();

        // Parse arguments
        let mut arguments = Vec::new();
        while children.peek().is_some() {
            let arg = children.next().unwrap();
            if arg.as_rule() == Rule::FUNCTION_EQUAL_OPERATOR {
                break;
            }

            arguments.push(arg.as_str().to_string());
        }

        // Confirm validity of the function body by parsing it here
        let expressions = children.filter(|pair| pair.as_str().trim() != "").map(|l| l.as_str().to_string()).collect::<Vec<_>>();
        for line in &expressions {
            parse_input(line, Rule::TOPLEVEL_EXPRESSION)?;
        }

        Ok(Self {
            name,
            arguments,
            expressions,
            token
        }.boxed())
    },

    value = |assignment: &FunctionAssignment, state: &mut State| {
        let function = UserFunction::new(&assignment.name, assignment.arguments.clone(), assignment.expressions.clone())?;
        let sig = function.to_string();
        state.set_user_function(function);
        Ok(Value::from(sig))
    }
);
