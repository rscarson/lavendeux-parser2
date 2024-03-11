use super::Node;
use crate::{
    error::{ErrorDetails, WrapExternalError},
    functions::{ParserFunction, UserDefinedFunction},
    syntax_tree::traits::IntoNode,
    Error, Rule, Token,
};
use polyvalue::{
    types::{Object, Range},
    Value, ValueTrait, ValueType,
};

define_ast!(
    Iterators {
        KeywordContinue() {
            build = (_pairs, token, _state) {
                Ok(Self { token }.into())
            },
            eval = (this, _state) {
                oops!(Skip, this.token.clone())
            },
            owned = (this) {
                Self::Owned { token: this.token.into_owned() }
            },
            docs = {
                name: "Continue",
                symbols = ["continue"],
                description: "Skips the current iteration of a loop",
                examples: "
                    for i in 0..10 { if i == 5 { continue } else {i} } 
                ",
            }
        },

        KeywordBreak() {
            build = (_pairs, token, _state) {
                Ok(Self { token }.into())
            },
            eval = (this, _state) {
                oops!(Break, this.token.clone())
            },
            owned = (this) {
                Self::Owned { token: this.token.into_owned() }
            },
            docs = {
                name: "Break",
                symbols = ["break"],
                description: "Breaks out of a loop",
                examples: "
                    for i in 0..10 { if i == 5 { break } else {i} }
                ",
            }
        },

        ForLoopExpression(
            variable: Option<String>,
            iterable: Box<Node<'i>>,
            body: Box<Node<'i>>,
            condition: Option<Box<Node<'i>>>
        ) {
            build = (pairs, token, state) {
                let condition = match pairs.peek_last() {
                    Some(p) if p.as_rule() == Rule::for_conditional => {
                        Some(Box::new(
                            unwrap_node!(
                                unwrap_last!(pairs, token),
                                state,
                                token
                            )?
                        ))
                    },
                    _ => None,
                };

                let body = Box::new(pairs.last_child().unwrap().into_node(state).with_context(&token)?);
                let iterable = Box::new(pairs.last_child().unwrap().into_node(state).with_context(&token)?);
                let variable = pairs.last_child().map(|p| p.as_str().to_string());

                Ok(Self { variable, iterable, body, condition, token }.into())
            },

            eval = (this, state) {
                let iterable = this.iterable.evaluate(state).with_context(this.token())?;

                // An iterator over Into<Value>
                let iterable = match iterable.own_type() {
                    ValueType::Range => {
                        let iterable = iterable.as_a::<Range>().with_context(this.token())?.inner().clone();
                        let mut values = vec![];
                        for i in iterable {
                            values.push(i.into());
                            state.check_timer().with_context(this.token())?; // Potentially long-running operation
                        }
                        values.into_iter()
                    },

                    ValueType::Object => {
                        let iterable = iterable.as_a::<Object>().with_context(this.token())?;
                        iterable.inner().keys().cloned().collect::<Vec<_>>().into_iter()
                    },

                    _ => {
                        let iterable = iterable.as_a::<polyvalue::types::Array>().with_context(this.token())?;
                        iterable.inner().clone().into_iter()
                    }
                };

                let mut result = vec![];
                for v in iterable {
                    state.check_timer().with_context(this.token())?; // Potentially long-running operation

                    state.scope_into().with_context(this.token())?;
                    if let Some(variable) = &this.variable {
                        state.set_variable(variable, v);
                    }
                    if let Some(condition) = &this.condition {
                        let condition = condition.evaluate(state).with_context(this.token());
                        match condition {
                            Ok(condition) if !condition.is_truthy() => {
                                state.scope_out();
                                continue;
                            },
                            Err(e) => {
                                state.scope_out();
                                return Err(e)
                            },
                            _ => {}
                        }
                    }

                    let value = this.body.evaluate(state);
                    state.scope_out();
                    match value {
                        Ok(value) => result.push(value),
                        Err(e) if error_matches!(e, Skip) => {},
                        Err(e) if error_matches!(e, Break) => {
                            break;
                        },

                        Err(e) => {
                            return Err(e);
                        }
                    }
                }

                Ok(Value::array(result))
            },

            owned = (this) {
                Self::Owned {
                    variable: this.variable.clone(),
                    iterable: Box::new(this.iterable.into_owned()),
                    body: Box::new(this.body.into_owned()),
                    condition: this.condition.map(|c| Box::new(c.into_owned())),
                    token: this.token.into_owned(),
                }
            },

            docs = {
                name: "For",
                symbols = ["for <variable> in <iterable> { <block> }", "for [<variable> in] <iterable> do <block> [if <condition>]"],
                description: "
                    For loops are finite value iterators. This means they map over a range, array, or object, 
                    and return a new array of values.
                    The variable is optional, and if not provided, the loop will not bind a variable.
                    The expression will return an array of the results of the block.
                    Break and skip/continue can be used to exit the loop or skip the current iteration.
                    A condition can be provided to filter the loop.
                ",
                examples: "
                    for i in 0..10 { i }
                    for i in [1, 2, 3] { i }
                    for i in {'a': 1, 'b': 2} { i }
        
                    for a in 0..10 do a if a % 2 == 0
        
                    for 0..10 do '!'
                ",
            }
        }
    }
);
