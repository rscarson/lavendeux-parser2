use super::*;
use crate::{error::WrapExternalError, pest::Rule, State};
use polyvalue::{
    operations::{ArithmeticOperation, ArithmeticOperationExt},
    Value,
};

define_prattnode!(
    InfixArithmetic {
        left: Node,
        right: Node,
        operator: ArithmeticOperation
    },
    rules = [OP_ADD, OP_SUB, OP_POW, OP_DIV, OP_MOD, OP_MUL],
    new = |input: PrattPair| {
        let token = input.as_token();
        let mut children = input.into_inner();
        let left = children.next().unwrap().to_ast_node()?;
        let operator = children.next().unwrap().as_rule();
        let right = children.next().unwrap().to_ast_node()?;

        let operator = match operator {
            Rule::OP_ADD => ArithmeticOperation::Add,
            Rule::OP_SUB => ArithmeticOperation::Subtract,
            Rule::OP_POW => ArithmeticOperation::Exponentiate,
            Rule::OP_DIV => ArithmeticOperation::Divide,
            Rule::OP_MOD => ArithmeticOperation::Modulo,
            Rule::OP_MUL => ArithmeticOperation::Multiply,
            _ => {
                return oops!(
                    Internal {
                        msg: format!("Unrecognize bitwise operator {operator:?}")
                    },
                    token
                )
            }
        };

        Ok(Self {
            left,
            right,
            operator,
            token,
        }
        .boxed())
    },
    value = |this: &Self, state: &mut State| {
        Value::arithmetic_op(
            &this.left.get_value(state)?,
            &this.right.get_value(state)?,
            this.operator,
        )
        .with_context(this.token())
    },

    docs = {
        name: "Arithmetic",
        symbols = ["+", "-", "*", "/", "%", "**"],
        description: "
            Performs arithmetic operations on two values.
            All but exponentiation are left-associative.
        ",
        examples: "
            1 + 2 / 3
            2 ** 3
        ",
    }
);

define_prattnode!(
    ArithmeticNeg { base: Node },
    rules = [PREFIX_NEG],
    new = |input: PrattPair| {
        let token = input.as_token();
        let mut children = input.into_inner();
        children.next(); // Skip the operator
        let base = children.next().unwrap().to_ast_node()?;
        Ok(Self { base, token }.boxed())
    },
    value = |this: &Self, state: &mut State| {
        Value::arithmetic_neg(&this.base.get_value(state)?).with_context(this.token())
    },

    docs = {
        name: "Unary Negation",
        symbols = ["-"],
        description: "Negates a value.",
        examples: "-1",
    }
);

define_prattnode!(
    ArithmeticIncDec {
        base: String,
        is_postfix: bool,
        is_increment: bool
    },
    rules = [PREFIX_INC, PREFIX_DEC, POSTFIX_INC, POSTFIX_DEC],
    new = |input: PrattPair| {
        let token = input.as_token();
        let (base, is_postfix, is_increment) = match input {
            PrattPair::Prefix(o, v) => (v, false, matches!(o.as_rule(), Rule::PREFIX_INC)),
            PrattPair::Postfix(v, o) => (v, true, matches!(o.as_rule(), Rule::POSTFIX_INC)),
            _ => unreachable!(),
        };
        if base.as_rule() != Rule::identifier {
            return oops!(ConstantValue, base.as_token());
        }
        let base = base.first_pair().as_str().to_string();

        Ok(Self {
            base: base,
            is_postfix,
            is_increment,
            token,
        }
        .boxed())
    },
    value = |this: &Self, state: &mut State| {
        let value = state.get_variable(&this.base).unwrap_or(Value::i64(0));
        let operation = if this.is_increment {
            ArithmeticOperation::Add
        } else {
            ArithmeticOperation::Subtract
        };

        let result = Value::arithmetic_op(&value, &Value::i64(1), operation)?;

        state.set_variable(&this.base, result.clone());
        if this.is_postfix {
            Ok(value)
        } else {
            Ok(result)
        }
    },

    docs = {
        name: "Unary Increment/Decrement",
        symbols = ["++", "--"],
        description: "Increments or decrements a variable by 1.",
        examples: "
            a = 0
            assert_eq(a++, 0)
            assert_eq(--a, 0)
        ",
    }
);
