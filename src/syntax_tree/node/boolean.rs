//! Unary Nodes
//!
//! Nodes for unary operations
//!
use super::*;
use crate::{error::WrapError, Rule, State, ToToken, Value};
use pest::iterators::Pair;
use polyvalue::types::Bool;
use polyvalue::{operations::*, ValueTrait};

define_node!(
    BooleanExpression {
        operand_stack: Vec<Node>,
        operator_stack: Vec<BooleanOperation>
    },
    rules = [BOOLEAN_AND_EXPRESSION, BOOLEAN_OR_EXPRESSION, BOOLEAN_CMP_EXPRESSION],
    new = |input: Pair<Rule>| {
        let token = input.to_token();
        let mut children = input.into_inner().rev().peekable();

        let mut expr = Self {
            operand_stack: Vec::new(),
            operator_stack: Vec::new(),
            token,
        };

        // We will build up a stack of operands and operators
        expr.operand_stack.push(children.next().unwrap().to_ast_node()?);
        while children.peek().is_some() {
            let operation = children.next().unwrap().as_str();
            let operation = match operation {
                "==" => BooleanOperation::EQ,
                "!=" => BooleanOperation::NEQ,
                "<" => BooleanOperation::LT,
                "<=" => BooleanOperation::LTE,
                ">" => BooleanOperation::GT,
                ">=" => BooleanOperation::GTE,
                "&&" => BooleanOperation::And,
                "||" => BooleanOperation::Or,
                _ => {
                    return Err(Error::Internal(format!(
                        "Invalid boolean operation {:?}",
                        operation
                    )))
                }
            };
            expr.operator_stack.push(operation);

            let operand = children.next().unwrap().to_ast_node()?;
            expr.operand_stack.push(operand);
        }

        Ok(expr.boxed())
    },
    value = |this: &BooleanExpression, state: &mut State| {
        let mut operands = this.operand_stack.iter().rev().peekable();
        let mut operators = this.operator_stack.iter().rev().peekable();

        let mut left = operands.next().unwrap().get_value(state)?;
        while let Some(op) = operators.next() {
            let ss_eval_op = *left.as_a::<Bool>().to_error(&this.token)?.inner();
            if *op == BooleanOperation::And && !ss_eval_op {
                // Short circuit
                left = Value::from(false);
            } else if *op == BooleanOperation::Or && ss_eval_op {
                // Short circuit
                left = Value::from(true);
            } else {
                let right = operands.next().unwrap().get_value(state)?;
                left = Value::boolean_op(&left, &right, *op).to_error(&this.token)?;
            }
        }

        Ok(left)
    }
);

define_node!(
    BooleanNotExpression { expression: Node },
    rules = [BOOLEAN_NOT_EXPRESSION],
    new = |input: Pair<Rule>| {
        let token = input.to_token();
        let mut children = input.into_inner().rev();
        let expression = children.next().unwrap().to_ast_node()?;

        // If there are an odd number of negations, we need to invert the value
        let mut do_invert = false;
        while children.next().is_some() {
            do_invert = !do_invert;
        }

        if do_invert {
            Ok(Self { expression, token }.boxed())
        } else {
            // If there are an even number of negations, we can just return the inner expression
            Ok(expression)
        }
    },
    value = |this: &BooleanNotExpression, state: &mut State| {
        let value = this.expression.get_value(state)?;
        let value = Value::boolean_op(&value, &value.clone(), BooleanOperation::Not)
            .to_error(&this.token)?;
        Ok(value)
    }
);

define_node!(
    MatchingExpression {
        value: Node,
        operation: MatchingOperation,
        pattern: Node
    },
    rules = [MATCHING_EXPRESSION],
    new = |input: Pair<Rule>| {
        let token = input.to_token();
        let mut children = input.into_inner();

        let value = children.next().unwrap().to_ast_node()?;
        let operation = children.next().unwrap().as_str();
        let operation = match operation {
            "contains" => MatchingOperation::Contains,
            "matches" => MatchingOperation::Matches,
            "is" => MatchingOperation::Is,
            "startswith" | "starts_with" => MatchingOperation::StartsWith,
            "endswith" | "ends_with" => MatchingOperation::EndsWith,
            _ => {
                return Err(Error::Internal(format!(
                    "Invalid matching operation {:?}",
                    operation
                )))
            }
        };
        let pattern = children.next().ok_or(Error::IncompleteMatchingExpression {
            token: token.clone(),
        })?;
        let pattern = pattern.to_ast_node()?;

        Ok(Self {
            value,
            operation,
            pattern,
            token,
        }
        .boxed())
    },
    value = |this: &MatchingExpression, state: &mut State| {
        let value = this.value.get_value(state)?;

        // is is a special case because it is the only operation that can accept an identifier
        // as a pattern - the type name can be a string, but does not need to be
        let pattern = match this.pattern.get_value(state) {
            Ok(pattern) => pattern,
            Err(Error::VariableName { name, .. }) if this.operation == MatchingOperation::Is => {
                Value::from(name)
            }
            Err(e) => return Err(e),
        };

        Ok(Value::matching_op(&value, &pattern, this.operation).to_error(&this.token)?)
    }
);

#[cfg(test)]
mod test {
    use super::*;
    use crate::{assert_tree, assert_tree_value, Lavendeux};

    #[test]
    fn test_matching_expr() {
        assert_tree!(
            "'a' contains 'b'",
            MATCHING_EXPRESSION,
            MatchingExpression,
            |tree: &mut MatchingExpression| {
                let value = tree.get_value(&mut State::new()).unwrap();
                assert_eq!(value.to_string(), "false");
            }
        );

        assert_tree_value!("'a' contains 'a'", true.into());
        assert_tree_value!("'a' matches 'a'", true.into());
        assert_tree_value!("'a' is string", true.into());
        assert_tree_value!("'abc' startswith 'a'", true.into());
        assert_tree_value!("'abc' ends_with 'c'", true.into());
    }

    #[test]
    fn test_boolean_expr() {
        assert_tree!(
            "true && false",
            TOPLEVEL_EXPRESSION,
            BooleanExpression,
            |tree: &mut BooleanExpression| {
                let value = tree.get_value(&mut State::new()).unwrap();
                assert_eq!(value.to_string(), "false");
            }
        );

        assert_tree!(
            "true || false",
            TOPLEVEL_EXPRESSION,
            BooleanExpression,
            |tree: &mut BooleanExpression| {
                let value = tree.get_value(&mut State::new()).unwrap();
                assert_eq!(value.to_string(), "true");
            }
        );

        assert_tree!(
            "true && false || true",
            TOPLEVEL_EXPRESSION,
            BooleanExpression,
            |tree: &mut BooleanExpression| {
                let value = tree.get_value(&mut State::new()).unwrap();
                assert_eq!(value.to_string(), "true");
            }
        );

        assert_tree!(
            "3 > 1",
            TOPLEVEL_EXPRESSION,
            BooleanExpression,
            |tree: &mut BooleanExpression| {
                let value = tree.get_value(&mut State::new()).unwrap();
                assert_eq!(value.to_string(), "true");
            }
        );

        assert_tree!(
            "3 >= 3",
            TOPLEVEL_EXPRESSION,
            BooleanExpression,
            |tree: &mut BooleanExpression| {
                let value = tree.get_value(&mut State::new()).unwrap();
                assert_eq!(value.to_string(), "true");
            }
        );

        assert_tree!(
            "3 < 1",
            TOPLEVEL_EXPRESSION,
            BooleanExpression,
            |tree: &mut BooleanExpression| {
                let value = tree.get_value(&mut State::new()).unwrap();
                assert_eq!(value.to_string(), "false");
            }
        );

        assert_tree!(
            "3 <= 3",
            TOPLEVEL_EXPRESSION,
            BooleanExpression,
            |tree: &mut BooleanExpression| {
                let value = tree.get_value(&mut State::new()).unwrap();
                assert_eq!(value.to_string(), "true");
            }
        );

        assert_tree!(
            "3 == 3",
            TOPLEVEL_EXPRESSION,
            BooleanExpression,
            |tree: &mut BooleanExpression| {
                let value = tree.get_value(&mut State::new()).unwrap();
                assert_eq!(value.to_string(), "true");
            }
        );

        assert_tree!(
            "3 != 3",
            TOPLEVEL_EXPRESSION,
            BooleanExpression,
            |tree: &mut BooleanExpression| {
                let value = tree.get_value(&mut State::new()).unwrap();
                assert_eq!(value.to_string(), "false");
            }
        );

        // test short circuiting
        let mut parser = Lavendeux::new(Default::default());
        parser.state_mut().set_variable("a", Value::from(true));
        assert_eq!(
            parser.parse("false && (del a)").unwrap()[0],
            Value::from(false)
        );
        assert_eq!(parser.state().get_variable("a").unwrap(), Value::from(true));

        assert_eq!(
            parser.parse("true || (del a)").unwrap()[0],
            Value::from(true)
        );
        assert_eq!(parser.state().get_variable("a").unwrap(), Value::from(true));
    }

    #[test]
    fn test_boolean_not_expr() {
        assert_tree!(
            "!true",
            TOPLEVEL_EXPRESSION,
            BooleanNotExpression,
            |tree: &mut BooleanNotExpression| {
                let value = tree.get_value(&mut State::new()).unwrap();
                assert_eq!(value.to_string(), "false");
            }
        );

        assert_tree!(
            "!!true",
            TOPLEVEL_EXPRESSION,
            ValueLiteral,
            |tree: &mut ValueLiteral| {
                let value = tree.get_value(&mut State::new()).unwrap();
                assert_eq!(value.to_string(), "true");
            }
        );
    }
}
