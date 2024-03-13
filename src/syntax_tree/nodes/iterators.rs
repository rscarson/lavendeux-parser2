use super::Node;
use crate::{
    error::{ErrorDetails, WrapExternalError},
    functions::{ParserFunction, UserDefinedFunction},
    pest::NodeExt,
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

        KeywordBreak(value: Option<Node<'i>>) {
            build = (pairs, token, _state) {
                pairs.next(); // Skip the break keyword
                let value = pairs.next().map(|p| p.into_node(_state)).transpose()?;
                Ok(Self { value, token }.into())
            },
            eval = (this, state) {
                let value = this.value.clone().map(|v| v.evaluate(state)).transpose()?;
                oops!(Break { value }, this.token.clone())
            },
            owned = (this) {
                Self::Owned {
                    value: this.value.map(|v| v.into_owned()),
                    token: this.token.into_owned(),
                }
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
            iterable: Node<'i>,
            body: Node<'i>,
            condition: Option<Node<'i>>
        ) {
            build = (pairs, token, state) {
                pairs.next(); // Skip the for keyword

                // Assignment
                let variable = match pairs.peek() {
                    Some(p) if p.as_rule() == Rule::for_assignment => {
                        let mut p = unwrap_next!(pairs, token);
                        let p = unwrap_next!(p, token);
                        Some(p.as_str().to_string())
                    },
                    _ => None,
                };

                // The actual iterable
                let iterable = unwrap_node!(pairs, state, token)?;

                // Do keyword?
                if let Some(p) = pairs.peek() {
                    if p.as_rule() == Rule::do_keyword {
                        pairs.next(); // Skip the do keyword
                    }
                }

                // The body
                let body = unwrap_node!(pairs, state, token)?;

                // Condition?
                let condition = match pairs.peek() {
                    Some(p) if p.as_rule() == Rule::for_conditional => {
                        let mut p = unwrap_next!(pairs, token);
                        p.next(); // Skip the if keyword
                        Some(unwrap_node!(p, state, token)?)
                    },
                    _ => None,
                };

                Ok(Self { variable, iterable, body, condition, token }.into())
            },

            eval = (this, state) {
                let iterable = this.iterable.evaluate(state).with_context(this.token())?;
                match iterable.own_type() {
                    ValueType::Range => {
                        let iterable = iterable.as_a::<Range>().with_context(this.token())?.into_inner();
                        let values = iterable.into_iter().map(|i| {
                            state.check_timer()?;
                            Ok::<_, Error>(Value::from(i))
                        }).collect::<Result<Vec<_>, _>>().with_context(this.token())?;
                        iterate_over(values.into_iter(), state, this)
                    },

                    ValueType::Object => {
                        let iterable = iterable.as_a::<Object>().with_context(this.token())?;
                        let iterable = iterable.keys().into_iter().cloned();
                        iterate_over(iterable, state, this)
                    },

                    _ => {
                        let iterable = iterable.as_a::<Vec<Value>>().with_context(this.token())?;
                        iterate_over(iterable.into_iter(), state, this)
                    }
                }
            },

            owned = (this) {
                Self::Owned {
                    variable: this.variable,
                    iterable: this.iterable.into_owned(),
                    body: this.body.into_owned(),
                    condition: this.condition.map(|c| c.into_owned()),
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

fn iterate_over(
    iterable: impl Iterator<Item = Value>,
    state: &mut crate::State,
    this: &ForLoopExpression,
) -> Result<Value, Error> {
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
                }
                Err(e) => {
                    state.scope_out();
                    return Err(e);
                }
                _ => {}
            }
        }

        let value = this.body.evaluate(state);
        state.scope_out();
        match value {
            Ok(value) => result.push(value),
            Err(e) if error_matches!(e, Skip) => {}
            Err(e) => {
                if let ErrorDetails::Break { value } = e.details {
                    if let Some(value) = value {
                        result.push(value);
                    }
                    break;
                } else {
                    return Err(e);
                }
            }
        }
    }

    Ok(Value::array(result))
}
