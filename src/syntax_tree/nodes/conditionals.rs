use super::Node;
use crate::{
    error::{ErrorDetails, WrapExternalError},
    functions::{ParserFunction, UserDefinedFunction},
    syntax_tree::traits::{IntoNode, IntoOwned},
    Error, Rule, Token,
};
use polyvalue::{
    types::{Object, Range},
    Value, ValueTrait, ValueType,
};

define_ast!(
    Conditionals {
        IfExpression(
            condition: Box<Node<'i>>,
            then_branch: Box<Node<'i>>,
            else_branch: Box<Node<'i>>
        ) {
            build = (pairs, token, state) {
                if pairs.len() % 2 == 0 {
                    // We parse as a set of (if, then) pairs ending with an else
                    // if the number of children is even, we have no else
                    return oops!(NoElseBlock, token.clone());
                }

                // We will begin at the end, the final right-side expression
                // Then we will work backwards, grabbing pairs of expressions
                // And turning them into ternary expressions with the previous
                // iteration as the false side until we run out of children
                let mut else_branch = pairs.last_child().unwrap().into_node(state).with_context(&token)?;
                while pairs.peek().is_some() {
                    let then_branch = pairs.last_child().unwrap().into_node(state).with_context(&token)?;
                    let condition = pairs.last_child().unwrap().into_node(state).with_context(&token)?;

                    else_branch = Self {
                        condition: Box::new(condition),
                        then_branch: Box::new(then_branch),
                        else_branch: Box::new(else_branch),
                        token: token.clone(),
                    }.into();
                }

                Ok(else_branch)
            },
            eval = (this, state) {
                let condition = this.condition.evaluate(state).with_context(this.token())?;
                state.scope_into().with_context(this.token())?;
                let result = if condition.is_truthy() {
                    this.then_branch.evaluate(state)
                } else {
                    this.else_branch.evaluate(state)
                };

                state.scope_out();
                result
            },
            owned = (this) {
                Self::Owned {
                    condition: Box::new(this.condition.into_owned()),
                    then_branch: Box::new(this.then_branch.into_owned()),
                    else_branch: Box::new(this.else_branch.into_owned()),
                    token: this.token.into_owned(),
                }
            },

            docs = {
                name: "If",
                symbols = ["if <condition> then <block> else <block>", "if <condition> {block} else {block}"],
                description: "
                    A conditional expression that evaluates a condition and then one of two branches.
                    body can be either a block or a single expression. The last expression is returned from a block.
                    Since all expressions in lavendeux return a value, the if expression will return the value of the branch that is executed.
                    As such, all if expressions must have both a then and an else branch.
                    If a condition doesn't need to return a value (side-effect `if`s for example), use `nil`
                ",
                examples: "
                    a = 6
                    if a > 5 { a } else { 5 }
                    if a == 4 {
                        a
                    } else if a == 5 {
                        5
                    } else nil
                ",
            }
        },

        SwitchExpression(
            match_on: Box<Node<'i>>,
            cases: Vec<SwitchCase<'i>>
        ) {
            build = (pairs, token, state) {
                let match_on = Box::new(pairs.next().unwrap().into_node(state).with_context(&token)?);
                let mut cases = vec![];

                while let Some(case) = pairs.next()  {
                    let body = pairs.next().unwrap().into_node(state).with_context(&token)?;

                    if case.as_str() == "_" {
                        cases.push(SwitchCase::Default(body));
                        if pairs.next().is_some() {
                            return oops!(UnreachableSwitchCase, token);
                        }

                        break;
                    } else {
                        cases.push(SwitchCase::Case(case.into_node(state).with_context(&token)?, body));
                    }
                }

                Ok(Self {
                    match_on,
                    cases,
                    token,
                }.into())
            },
            eval = (this, state) {
                let match_on = this.match_on.evaluate(state).with_context(this.token())?;

                for case in &this.cases {
                    match case {
                        SwitchCase::Default(body) => {
                            state.scope_into().with_context(this.token())?;
                            let result = body.evaluate(state);

                            state.scope_out();
                            return result;
                        },

                        SwitchCase::Case(value, body) => {
                            let value = value.evaluate(state).with_context(this.token())?;

                            if value.own_type() != match_on.own_type() {
                                return oops!(SwitchCaseTypeMismatch {
                                    case: value,
                                    expected_type: match_on.own_type()
                                }, this.token.clone());
                            }

                            if value == match_on {
                                state.scope_into().with_context(this.token())?;
                                let result = body.evaluate(state);

                                state.scope_out();
                                return result;
                            }
                        }
                    }
                }

                oops!(NonExhaustiveSwitch, this.token.clone())
            },
            owned = (this) {
                Self::Owned {
                    match_on: Box::new(this.match_on.into_owned()),
                    cases: this.cases.into_iter().map(|c| c.into_owned()).collect(),
                    token: this.token.into_owned(),
                }
            },
            docs = {
                name: "match",
                symbols = ["match <value> { <condition> => <block>, _ => <block> }"],
                description: "
                    A conditional expression that evaluates a value and then one of several cases.
                    match blocks must be exhaustive, and therefore must end in a default case
                ",
                examples: "
                    a = 6
                    match a {
                        5 => { 'five' },
                        6 => { 'six' },
                        _ => { 'other' }
                    }
                ",
            }
        }
    }
);

#[derive(Debug, Clone)]
pub enum SwitchCase<'i> {
    Default(Node<'i>),
    Case(Node<'i>, Node<'i>),
}
impl IntoOwned for SwitchCase<'_> {
    type Owned = SwitchCase<'static>;
    fn into_owned(self) -> Self::Owned {
        match self {
            Self::Default(node) => Self::Owned::Default(node.into_owned()),
            Self::Case(condition, body) => {
                Self::Owned::Case(condition.into_owned(), body.into_owned())
            }
        }
    }
}

define_handler!(
    TernaryExpression(pairs, token, state) {
        let condition = pairs.next().unwrap().into_node(state).with_context(&token)?;

        let mut then_pair = pairs.next().unwrap();
        let then_branch = then_pair.next().unwrap().into_node(state).with_context(&token)?;
        let else_branch = pairs.next().unwrap().into_node(state).with_context(&token)?;

        Ok(IfExpression {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
            token,
        }.into())
    }
);
document_operator!(
    name = "Ternary",
    rules = [],
    symbols = ["condition ? then : else"],
    description = "
        A right-associative ternary operator.
        The condition is evaluated first, then either the then or else branch is evaluated.
    ",
    examples = "true ? 1 : 2",
);
