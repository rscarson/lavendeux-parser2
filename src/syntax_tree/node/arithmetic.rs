//! Infix Nodes
//!
//! Nodes for infix mathematical operations
//!
use super::*;
use crate::{error::WrapError, Rule, State, ToToken, Value};
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
        ARITHMETIC_EXPONENTIATION_EXPRESSION
    ],
    new = |input: Pair<Rule>| {
        let token = input.to_token();
        let mut children = input.into_inner().rev().peekable();

        let mut expr = Self {
            operand_stack: Vec::new(),
            operator_stack: Vec::new(),
            token: token.clone(),
        };

        // We will build up a stack of operands and operators
        expr.operand_stack.push(children.next().unwrap().to_ast_node()?);
        while children.peek().is_some() {
            let operation = children.next().unwrap().as_str();
            let operation = match operation {
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
            };

            expr.operator_stack.push(operation);
            let operand = children.next().unwrap().to_ast_node()?;
            expr.operand_stack.push(operand);
        }

        Ok(expr.boxed())
    },

    value = |this: &ArithmeticExpression, state: &mut State| {
        let mut operands = this.operand_stack.iter().rev().peekable();
        let mut operators = this.operator_stack.iter().rev().peekable();

        let mut left = operands.next().unwrap().get_value(state)?;
        while let Some(op) = operators.next() {
            let right = operands.next().unwrap().get_value(state)?;
            left = Value::arithmetic_op(&left, &right, *op).to_error(&this.token)?;
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
    value = |this: &ArithmeticNegExpression, state: &mut State| {
        let value = this.expression.get_value(state)?;
        let value = Value::arithmetic_op(&value, &value.clone(), ArithmeticOperation::Negate)
            .to_error(&this.token)?;
        Ok(value)
    }
);

#[cfg(test)]
mod test {
    use super::*;
    use crate::{assert_tree, assert_tree_value};

    #[test]
    fn test_arithmetic_expr() {
        // Test a big ugly nested expression, to rule out stack overflows
        assert_tree!(
            "2*(2*(2*(2*(2*(2*(2*(2*(2*(2*(2*(2*(22*(2*(2*(2*(2))))))))))))))))",
            TOPLEVEL_EXPRESSION,
            ArithmeticExpression,
            |_: &mut ArithmeticExpression| {}
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

        assert_tree_value!("4/2", 2i64.into());
        assert_tree_value!("4%2", 0i64.into());
        assert_tree_value!("2**3", 8i64.into());
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
