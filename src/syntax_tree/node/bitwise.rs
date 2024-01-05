//! Unary Nodes
//!
//! Nodes for unary operations
//!
use super::*;
use crate::{Rule, State, ToToken, Value};
use pest::iterators::Pair;
use polyvalue::operations::*;

define_node!(
    BitwiseExpression {
        operand_stack: Vec<Node>,
        operator_stack: Vec<BitwiseOperation>
    },
    rules = [BITWISE_AND_EXPRESSION, BITWISE_XOR_EXPRESSION, BITWISE_OR_EXPRESSION, BITWISE_SHIFT_EXPRESSION],
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
                "|" => BitwiseOperation::Or,
                "&" => BitwiseOperation::And,
                "^" => BitwiseOperation::Xor,
                "<<" => BitwiseOperation::LeftShift,
                ">>" => BitwiseOperation::RightShift,
                _ => {
                    return Err(Error::Internal(format!(
                        "Invalid bitwise operation {:?}",
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
    value = |this: &mut BitwiseExpression, state: &mut State| {
        let mut operands = this.operand_stack.iter_mut().rev().peekable();
        let mut operators = this.operator_stack.iter_mut().rev().peekable();

        let mut left = operands.next().unwrap().get_value(state)?;
        while let Some(op) = operators.next() {
            let right = operands.next().unwrap().get_value(state)?;
            left = Value::bitwise_op(&left, &right, *op)?;
        }

        Ok(left)
    }
);

define_node!(
    BitwiseNotExpression { expression: Node },
    rules = [BITWISE_NOT_EXPRESSION],
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
    value = |this: &mut BitwiseNotExpression, state: &mut State| {
        let value = this.expression.get_value(state)?;
        let value = Value::bitwise_op(&value, &value.clone(), BitwiseOperation::Not)?;
        Ok(value)
    }
);

#[cfg(test)]
mod test {
    use super::*;
    use crate::assert_tree;

    #[test]
    fn test_bitwise_expr() {
        assert_tree!(
            "0x0A | 0x05",
            TOPLEVEL_EXPRESSION,
            BitwiseExpression,
            |tree: &mut BitwiseExpression| {
                let value = tree.get_value(&mut State::new()).unwrap();
                assert_eq!(value.to_string(), "15");
            }
        );

        assert_tree!(
            "0x0A & 0x05",
            TOPLEVEL_EXPRESSION,
            BitwiseExpression,
            |tree: &mut BitwiseExpression| {
                let value = tree.get_value(&mut State::new()).unwrap();
                assert_eq!(value.to_string(), "0");
            }
        );

        assert_tree!(
            "0x0A ^ 0x05",
            TOPLEVEL_EXPRESSION,
            BitwiseExpression,
            |tree: &mut BitwiseExpression| {
                let value = tree.get_value(&mut State::new()).unwrap();
                assert_eq!(value.to_string(), "15");
            }
        );

        // Test precedence
        assert_tree!(
            "0x0A | 0x05 & 0x0F",
            TOPLEVEL_EXPRESSION,
            BitwiseExpression,
            |tree: &mut BitwiseExpression| {
                assert_eq!(BitwiseOperation::Or, tree.operator_stack[0]);
            }
        );
    }

    #[test]
    fn test_bitwise_not_expr() {
        assert_tree!(
            "~0xA0",
            TOPLEVEL_EXPRESSION,
            BitwiseNotExpression,
            |tree: &mut BitwiseNotExpression| {
                let value = tree.get_value(&mut State::new()).unwrap();
                assert_eq!(value.to_string(), "95");
            }
        );

        assert_tree!(
            "~~0x0A",
            TOPLEVEL_EXPRESSION,
            ValueLiteral,
            |tree: &mut ValueLiteral| {
                let value = tree.get_value(&mut State::new()).unwrap();
                assert_eq!(value.to_string(), "10");
            }
        );
    }
}
