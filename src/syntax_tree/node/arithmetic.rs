//! Infix Nodes
//!
//! Nodes for infix mathematical operations
//!
use super::*;
use crate::{Rule, State, ToToken, Value};
use pest::iterators::Pair;
use polyvalue::operations::*;

define_node!(
    ArithmeticExpression {
        operand_stack: Vec<Node>,
        operator_stack: Vec<ArithmeticOperation>
    },
    rules = [
        ARITHMETIC_AS_EXPRESSION,
        ARITHMETIC_MD_EXPRESSION,
        ARITHMETIC_IMPLICIT_MUL_EXPRESSION,
        ARITHMETIC_EXPONENTIATION_EXPRESSION
    ],
    new = |input: Pair<Rule>| {
        let rule = input.as_rule();
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
            let operation = if rule == Rule::ARITHMETIC_IMPLICIT_MUL_EXPRESSION {
                // Implicit multiplication is just multiplication
                // Do not consume a child for the operation
                ArithmeticOperation::Multiply
            } else {
                let operation = children.next().unwrap().as_str();
                match operation {
                    "+" => ArithmeticOperation::Add,
                    "-" => ArithmeticOperation::Subtract,
                    "*" => ArithmeticOperation::Multiply,
                    "/" => ArithmeticOperation::Divide,
                    "%" => ArithmeticOperation::Modulo,
                    "**" => ArithmeticOperation::Exponentiate,
                    _ => {
                        return Err(Error::Internal(format!(
                            "Invalid arithmetic operation {:?}",
                            operation
                        )))
                    }
                }
            };
            expr.operator_stack.push(operation);

            let operand = children.next().unwrap().to_ast_node()?;

            expr.operand_stack.push(operand);
        }

        Ok(expr.boxed())
    },

    value = |this: &mut ArithmeticExpression, state: &mut State| {
        let mut operands = this.operand_stack.iter_mut().rev().peekable();
        let mut operators = this.operator_stack.iter_mut().rev().peekable();

        let mut left = operands.next().unwrap().get_value(state)?;
        while let Some(op) = operators.next() {
            let right = operands.next().unwrap().get_value(state)?;
            left = Value::arithmetic_op(&left, &right, *op)?;
        }

        Ok(left)
    }
);

define_node!(
    ArithmeticNegExpression { expression: Node },
    rules = [ARITHMETIC_NEG_EXPRESSION],
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
    value = |this: &mut ArithmeticNegExpression, state: &mut State| {
        let value = this.expression.get_value(state)?;
        let value = Value::arithmetic_op(&value, &value.clone(), ArithmeticOperation::Negate)?;
        Ok(value)
    }
);

#[cfg(test)]
mod test {
    use super::*;
    use crate::assert_tree;

    #[test]
    fn test_arithmetic_expr() {
        // Test a big ugly nested expression, to rule out stack overflows
        assert_tree!(
            "2*(2*(2*(2*(2*(2*(2*(2*(2*(2*(2*(2*(22*(2*(2*(2*(2))))))))))))))))",
            TOPLEVEL_EXPRESSION,
            ArithmeticExpression,
            |_: &mut ArithmeticExpression| {}
        );

        // Test implicit multiplication
        assert_tree!(
            "2 2",
            TOPLEVEL_EXPRESSION,
            ArithmeticExpression,
            |tree: &mut ArithmeticExpression| {
                assert_eq!(tree.operand_stack.len(), 2);
                assert_eq!(tree.operator_stack.len(), 1);
                assert_eq!(tree.operator_stack[0], ArithmeticOperation::Multiply);
            }
        );

        // Test precedence
        assert_tree!(
            "2+2*2",
            TOPLEVEL_EXPRESSION,
            ArithmeticExpression,
            |tree: &mut ArithmeticExpression| {
                assert_eq!(tree.operand_stack.len(), 2);
                assert_eq!(tree.operator_stack.len(), 1);
                assert_eq!(tree.operator_stack[0], ArithmeticOperation::Add);

                assert_tree!(
                    &mut tree.operand_stack[0],
                    ArithmeticExpression,
                    |tree: &mut ArithmeticExpression| {
                        assert_eq!(tree.operand_stack.len(), 2);
                        assert_eq!(tree.operator_stack.len(), 1);
                        assert_eq!(tree.operator_stack[0], ArithmeticOperation::Multiply);
                    }
                );

                let value = tree.get_value(&mut State::new()).unwrap();
                assert_eq!(value.to_string(), "6");
            }
        );
    }

    #[test]
    fn test_arithmetic_neg_expr() {
        assert_tree!(
            "-2",
            TOPLEVEL_EXPRESSION,
            ArithmeticNegExpression,
            |tree: &mut ArithmeticNegExpression| {
                let value = tree.get_value(&mut State::new()).unwrap();
                assert_eq!(value.to_string(), "-2");
            }
        );

        assert_tree!(
            "--2",
            TOPLEVEL_EXPRESSION,
            ValueLiteral,
            |tree: &mut ValueLiteral| {
                let value = tree.get_value(&mut State::new()).unwrap();
                assert_eq!(value.to_string(), "2");
            }
        );
    }
}
