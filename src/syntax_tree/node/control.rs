//! Core Node
//!
//! High-level nodes that are used to build the syntax tree.
//! These nodes are how the user will interact with the syntax tree.
//!
use super::*;
use crate::{error::WrapError, Error, Rule, State, ToToken, Value};
use pest::iterators::Pair;
use polyvalue::types::{Array, Object, Range};
use polyvalue::{ValueTrait, ValueType};

define_node!(
    BreakExpression,
    rules = [BREAK_KEYWORD],
    new = |input: Pair<Rule>| {
        let token = input.to_token();
        let mut children = input.into_inner();
        children.next(); // Break keyword

        Ok(Self { token }.boxed())
    },
    value = |this: &BreakExpression, _state: &mut State| {
        Err(Error::Break {
            token: this.token.clone(),
        })
    }
);

define_node!(
    SkipExpression,
    rules = [SKIP_KEYWORD],
    new = |input: Pair<Rule>| {
        let token = input.to_token();
        let mut children = input.into_inner();
        children.next(); // skip keyword

        Ok(Self { token }.boxed())
    },
    value = |this: &SkipExpression, _state: &mut State| {
        Err(Error::Skip {
            token: this.token.clone(),
        })
    }
);

define_node!(
    ReturnExpression { expression: Node },
    rules = [RETURN_EXPRESSION],
    new = |input: Pair<Rule>| {
        let token = input.to_token();
        let mut children = input.into_inner();

        children.next(); // skip keyword
        let expression = children.next().unwrap().to_ast_node()?;

        Ok(Self { expression, token }.boxed())
    },
    value = |return_expr: &ReturnExpression, state: &mut State| {
        let value = return_expr.expression.get_value(state)?;
        Err(Error::Return {
            value,
            token: return_expr.token.clone(),
        })
    }
);

// BOOLEAN_OR_EXPRESSION ~ ("?" ~ BOOLEAN_OR_EXPRESSION ~ ":" ~ BOOLEAN_OR_EXPRESSION)*
define_node!(
    TernaryExpression {
        condition: Node,
        if_true: Node,
        if_false: Node
    },
    rules = [TERNARY_EXPRESSION, IF_EXPRESSION],
    new = |input: Pair<Rule>| {
        let token = input.to_token();
        let mut children = input.into_inner().rev().peekable();

        // We will begin at the end, the final right-side expression
        // Then we will work backwards, grabbing pairs of expressions
        // And turning them into ternary expressions with the previous
        // iteration as the false side until we run out of children
        let mut if_false = children.next().unwrap().to_ast_node()?;
        while children.peek().is_some() {
            let if_true = children.next().unwrap().to_ast_node()?;
            let condition = children.next().unwrap().to_ast_node()?;

            if_false = Box::new(Self {
                condition,
                if_true,
                if_false,
                token: token.clone(),
            });
        }

        Ok(if_false)
    },
    value = |ternary: &TernaryExpression, state: &mut State| {
        let condition = ternary.condition.get_value(state)?;
        if condition.is_truthy() {
            ternary.if_true.get_value(state)
        } else {
            ternary.if_false.get_value(state)
        }
    }
);

define_node!(
    ForLoopExpression {
        variable: Option<String>,
        iterable: Node,
        body: Node
    },
    rules = [FOR_LOOP_EXPRESSION],

    new = |input:Pair<Rule>| {
        let token = input.to_token();
        let mut children = input.into_inner().rev();

        let body = children.next().unwrap().to_ast_node()?;
        let iterable = children.next().unwrap().to_ast_node()?;
        let variable = children.next().map(|c| c.as_str().to_string());

        Ok(Self {
            variable,
            iterable,
            body,
            token
        }.boxed())
    },

    value = |for_loop: &ForLoopExpression, state: &mut State| {
        let iterable = for_loop.iterable.get_value(state)?;

        match iterable.own_type() {
            ValueType::Range => {
                let iterable = iterable.as_a::<Range>().to_error(&for_loop.token)?;

                let mut result = vec![];
                state.scope_into(for_loop.token())?;
                for i in iterable.inner().clone() {
                    if let Some(variable) = &for_loop.variable {
                        state.set_variable(variable, i.into());
                    }

                    let value = for_loop.body.get_value(state);
                    match value {
                        Ok(value) => result.push(value),
                        Err(Error::Skip { .. }) => {},
                        Err(Error::Break { .. }) => {
                            break;
                        },

                        Err(e) => {
                            state.scope_out();
                            return Err(e);
                        }
                    }
                }
                state.scope_out();

                Ok(Value::array(result))
            },

            ValueType::Object => {
                let iterable = iterable.as_a::<Object>().to_error(&for_loop.token)?;

                let mut result = vec![];
                state.scope_into(for_loop.token())?;
                for value in iterable.inner().keys() {
                    if let Some(variable) = &for_loop.variable {
                        state.set_variable(variable, value.clone());
                    }

                    let value = for_loop.body.get_value(state);
                    match value {
                        Ok(value) => result.push(value),
                        Err(Error::Skip { .. }) => {},
                        Err(Error::Break { .. }) => {
                            break;
                        },

                        Err(e) => {
                            state.scope_out();
                            return Err(e);
                        }
                    }
                }
                state.scope_out();

                Ok(Value::array(result))
            }

            _ => {
                let iterable = iterable.as_a::<Array>().to_error(&for_loop.token)?;

                let mut result = vec![];
                state.scope_into(for_loop.token())?;
                for value in iterable.inner() {
                    if let Some(variable) = &for_loop.variable {
                        state.set_variable(variable, value.clone());
                    }

                    let value = for_loop.body.get_value(state);
                    match value {
                        Ok(value) => result.push(value),
                        Err(Error::Skip { .. }) => {},
                        Err(Error::Break { .. }) => {
                            break;
                        },

                        Err(e) => {
                            state.scope_out();
                            return Err(e);
                        }
                    }
                }
                state.scope_out();

                Ok(Value::array(result))
            }
        }
    }
);

#[derive(Debug)]
pub enum SwitchCase {
    Default(Node),
    Case(Node, Node),
}

define_node!(
    SwitchExpression {
        match_on: Node,
        cases: Vec<SwitchCase>
    },
    rules = [SWITCH_EXPRESSION],

    new = |input:Pair<Rule>| {
        let token = input.to_token();
        let mut children = input.into_inner();

        let match_on = children.next().unwrap().to_ast_node()?;
        let mut cases = vec![];

        while let Some(case) = children.next() {
            let body = children.next().unwrap().to_ast_node()?;

            if case.as_str() == "_" {
                cases.push(SwitchCase::Default(body));
                if children.next().is_some() {
                    return Err(Error::UnreachableSwitchCase {
                        token: token.clone(),
                    });
                }

                break;
            } else {
                cases.push(SwitchCase::Case(case.to_ast_node()?, body));
            }
        }

        Ok(Self {
            match_on,
            cases,
            token
        }.boxed())
    },

    value = |switch: &SwitchExpression, state: &mut State| {
        let match_on = switch.match_on.get_value(state)?;

        for case in &switch.cases {
            match case {
                SwitchCase::Default(body) => {
                    return body.get_value(state);
                },

                SwitchCase::Case(value, body) => {
                    let value = value.get_value(state)?;
                    if value.own_type() != match_on.own_type() {
                        return Err(Error::SwitchCaseTypeMismatch {
                            case: value,
                            expected_type: match_on.own_type(),
                            token: switch.token.clone(),
                        });
                    }

                    if value == match_on {
                        return body.get_value(state);
                    }
                }
            }
        }

        Err(Error::NonExhaustiveSwitch {
            token: switch.token.clone(),
        })
    }
);

#[cfg(test)]
mod test {
    use super::*;
    use crate::{assert_tree, assert_tree_error, assert_tree_value};

    #[test]
    fn test_switch() {
        assert_tree!(
            "match 1 {
                1 => 1,
                2 => 2,
                _ => 3
            }",
            SWITCH_EXPRESSION,
            SwitchExpression,
            |node: &mut SwitchExpression| {
                assert_eq!(node.match_on.to_string(), "1");
                assert_eq!(node.cases.len(), 3);
            }
        );

        assert_tree!(
            "match 2 {
                1 => 1,
                2 => 2,
                _ => 3
            }",
            SWITCH_EXPRESSION,
            SwitchExpression,
            |node: &mut SwitchExpression| {
                assert_eq!(node.match_on.to_string(), "2");
                assert_eq!(node.cases.len(), 3);
            }
        );

        assert_tree_error!(
            "match 3 {
                1 => 1,
                2 => 2,
            }",
            NonExhaustiveSwitch
        );

        assert_tree_error!(
            "match 3 {
                1 => 1,
                _ => 2,
                3 => 3
            }",
            UnreachableSwitchCase
        );

        assert_tree_value!("match 3 { 1 => 1, 2 => 2, _ => 3 }", Value::from(3i64));
    }

    #[test]
    fn test_for() {
        assert_tree!(
            "for a in [1, 2, 3] do a",
            FOR_LOOP_EXPRESSION,
            ForLoopExpression,
            |node: &mut ForLoopExpression| {
                assert_eq!(node.variable.as_ref().unwrap(), "a");
                assert_eq!(node.iterable.to_string(), "[1, 2, 3]");
                assert_eq!(node.body.to_string(), "a");

                assert_eq!(
                    node.get_value(&mut State::new()).unwrap().to_string(),
                    "[1, 2, 3]"
                );
            }
        );

        assert_tree!(
            "for a in [1, 2, 3] do a + 1",
            FOR_LOOP_EXPRESSION,
            ForLoopExpression,
            |node: &mut ForLoopExpression| {
                assert_eq!(node.variable.as_ref().unwrap(), "a");
                assert_eq!(node.iterable.to_string(), "[1, 2, 3]");
                assert_eq!(node.body.to_string(), "a + 1");

                assert_eq!(
                    node.get_value(&mut State::new()).unwrap().to_string(),
                    "[2, 3, 4]"
                );
            }
        );

        assert_tree!(
            "for a in [1, 2, 3] do a + 1",
            FOR_LOOP_EXPRESSION,
            ForLoopExpression,
            |node: &mut ForLoopExpression| {
                assert_eq!(node.variable.as_ref().unwrap(), "a");
                assert_eq!(node.iterable.to_string(), "[1, 2, 3]");
                assert_eq!(node.body.to_string(), "a + 1");

                assert_eq!(
                    node.get_value(&mut State::new()).unwrap().to_string(),
                    "[2, 3, 4]"
                );
            }
        );

        assert_tree!(
            "for a in [1, 2, 3] do a + 1",
            FOR_LOOP_EXPRESSION,
            ForLoopExpression,
            |node: &mut ForLoopExpression| {
                assert_eq!(node.variable.as_ref().unwrap(), "a");
                assert_eq!(node.iterable.to_string(), "[1, 2, 3]");
                assert_eq!(node.body.to_string(), "a + 1");

                assert_eq!(
                    node.get_value(&mut State::new()).unwrap().to_string(),
                    "[2, 3, 4]"
                );
            }
        );
    }

    #[test]
    fn test_ternary_expr() {
        assert_tree!(
            "true ? 1 : 0",
            TERNARY_EXPRESSION,
            TernaryExpression,
            |node: &mut TernaryExpression| {
                assert_eq!(node.condition.to_string(), "true");
                assert_eq!(node.if_true.to_string(), "1");
                assert_eq!(node.if_false.to_string(), "0");

                assert_eq!(node.get_value(&mut State::new()).unwrap().to_string(), "1");
            }
        );

        assert_tree!(
            "false ? 1 : 0",
            TERNARY_EXPRESSION,
            TernaryExpression,
            |node: &mut TernaryExpression| {
                assert_eq!(node.condition.to_string(), "false");
                assert_eq!(node.if_true.to_string(), "1");
                assert_eq!(node.if_false.to_string(), "0");

                assert_eq!(node.get_value(&mut State::new()).unwrap().to_string(), "0");
            }
        );

        // Test that short-circuiting works, to prevent a side-effect
        assert_tree!(
            "true ? 0 : pop(a)",
            TERNARY_EXPRESSION,
            TernaryExpression,
            |node: &mut TernaryExpression| {
                assert_eq!(node.condition.to_string(), "true");
                assert_eq!(node.if_true.to_string(), "0");
                assert_tree!(
                    &mut node.if_false,
                    FunctionCall,
                    |node: &mut FunctionCall| {
                        assert_eq!(node.name, "pop");
                        assert_eq!(node.arguments.len(), 1);
                        assert_eq!(node.arguments[0].to_string(), "a");
                    }
                );

                let mut state = State::new();
                state.set_variable("a", Value::from(vec![Value::from("0")]));

                assert_eq!(node.get_value(&mut State::new()).unwrap().to_string(), "0");

                assert_eq!(state.get_variable("a").unwrap().to_string(), "[0]");
            }
        )
    }
}
