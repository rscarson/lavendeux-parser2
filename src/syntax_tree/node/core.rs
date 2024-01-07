//! Core Node
//!
//! High-level nodes that are used to build the syntax tree.
//! These nodes are how the user will interact with the syntax tree.
//!
use super::*;
use crate::{Error, Rule, State, ToToken, Value};
use pest::iterators::Pair;
use polyvalue::types::{Array, Bool, Range};
use polyvalue::{ValueTrait, ValueType};

// LINE* ~ EOI
define_node!(
    Script {
        statements: Vec<Node>
    },
    rules = [SCRIPT],

    new = |input:Pair<Rule>| {
        let token = input.to_token();


        let statements = input
            .into_inner()
            .filter(|child| child.as_rule() != Rule::EOI && child.as_rule() != Rule::EOL)
            .map(|child| Ok(child.to_ast_node()?))
            .collect::<Result<Vec<Node>, Error>>()?;

        Ok(Self {
            statements,
            token
        }.boxed())
    },

    value = |script: &Script, state: &mut State| {
        let values = script.statements.iter().map(|node| node.get_value(state)).collect::<Result<Vec<_>, _>>()?;
        Ok(Value::Array(values.into()))
    }
);

// EXPRESSION ~ "@" ~ identifier ~ EOL
// | EXPRESSION ~ EOL
// | EOL
define_node!(
    Line {
        expression: Option<Node>,
        decorator: Option<String>
    },
    rules = [LINE],

    new = |input:Pair<Rule>| {
        let token = input.to_token();
        let mut children = input.into_inner();

        let next = children.next();
        if next.is_none() {
            return Ok(Self {
                expression: None,
                decorator: None,
                token
            }.boxed());
        } else {
            let expression = next.unwrap().to_ast_node()?;
            let decorator = children.next().and_then(|c| Some(c.as_str().to_string()));

            Ok(Self {
                expression: Some(expression),
                decorator,
                token
            }.boxed())
        }
    },

    value = |line: &Line, state: &mut State| {
        if let Some(expression) = &line.expression {
            let value = expression.get_value(state)?;
            if let Some(decorator) = &line.decorator {
                let result = state.decorate(decorator, line.token(), value)?;
                Ok(Value::from(result).into())
            } else {
                Ok(value)
            }
        } else {
            Ok(Value::from("").into())
        }
    }
);

// BOOLEAN_OR_EXPRESSION ~ ("?" ~ BOOLEAN_OR_EXPRESSION ~ ":" ~ BOOLEAN_OR_EXPRESSION)*
define_node!(
    TernaryExpression {
        condition: Node,
        if_true: Node,
        if_false: Node
    },
    rules = [TERNARY_EXPRESSION],
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
        let condition = *condition.as_a::<Bool>()?.inner();
        if condition {
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
                let iterable = iterable.as_a::<Range>()?;
                let mut result = vec![];
                state.scope_into()?;
                for i in iterable.inner().clone() {
                    let value = Value::from(i);
                    if let Some(variable) = &for_loop.variable {
                        state.set_variable(variable, value.clone());
                    }

                    let value = for_loop.body.get_value(state).or_else(|e| {
                        state.scope_out();
                        Err(e)
                    })?;
                    result.push(value);
                }
                state.scope_out();

                Ok(Value::Array(result.into()).into())
            },

            _ => {
                let iterable = iterable.as_a::<Array>()?;

                let mut result = vec![];
                state.scope_into()?;
                for value in iterable.inner() {
                    if let Some(variable) = &for_loop.variable {
                        state.set_variable(variable, value.clone());
                    }

                    let value = for_loop.body.get_value(state).or_else(|e| {
                        state.scope_out();
                        Err(e)
                    })?;
                    result.push(value);
                }
                state.scope_out();

                Ok(Value::Array(result.into()).into())
            }
        }
    }
);

#[cfg(test)]
mod test {
    use super::*;
    use crate::assert_tree;

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

    #[test]
    fn test_line() {
        assert_tree!("1", LINE, Line, |node: &mut Line| {
            assert_eq!(node.expression.as_ref().unwrap().to_string(), "1");
            assert_eq!(node.decorator, None);
            assert_eq!(node.get_value(&mut State::new()).unwrap().to_string(), "1");
        });

        assert_tree!("1 @bool", LINE, Line, |node: &mut Line| {
            assert_eq!(node.expression.as_ref().unwrap().to_string(), "1");
            assert_eq!(node.decorator.as_ref().unwrap(), "bool");
            assert_eq!(
                node.get_value(&mut State::new()).unwrap().to_string(),
                "true"
            );
        });

        assert_tree!("\n", LINE, Line, |node: &mut Line| {
            assert!(node.expression.as_ref().is_none());
            assert_eq!(node.decorator.as_ref(), None);
            assert_eq!(node.get_value(&mut State::new()).unwrap().to_string(), "");
        });
    }

    #[test]
    fn test_script() {
        assert_tree!("1\n2", SCRIPT, Script, |node: &mut Script| {
            assert_eq!(node.statements.len(), 2);
            assert_eq!(node.statements[0].to_string(), "1\n");
            assert_eq!(node.statements[1].to_string(), "2");
            assert_eq!(
                node.get_value(&mut State::new()).unwrap().to_string(),
                "[1, 2]"
            );
        });

        assert_tree!("1 \\\n@bool", SCRIPT, Script, |node: &mut Script| {
            assert_eq!(node.statements.len(), 1);
            assert_eq!(
                node.get_value(&mut State::new()).unwrap().to_string(),
                "[true]"
            );
        });

        assert_tree!("", SCRIPT, Script, |node: &mut Script| {
            assert_eq!(node.statements.len(), 0);
        });
    }
}
