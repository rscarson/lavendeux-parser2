use super::*;
use crate::{error::WrapExternalError, pest::Rule};
use polyvalue::{
    operations::{BitwiseOperation, BitwiseOperationExt},
    Value,
};

define_prattnode!(
    InfixBitwise {
        left: Node<'i>,
        right: Node<'i>,
        operator: BitwiseOperation
    },
    rules = [OP_BIT_OR, OP_BIT_XOR, OP_BIT_AND, OP_BIT_SL, OP_BIT_SR],
    new = (input) {
        let token = input.as_token();
        let mut children = input.into_inner();
        let left = children.next().unwrap().to_ast_node()?;
        let operator = children.next().unwrap().as_rule();
        let right = children.next().unwrap().to_ast_node()?;

        let operator = match operator {
            Rule::OP_BIT_OR => BitwiseOperation::Or,
            Rule::OP_BIT_XOR => BitwiseOperation::Xor,
            Rule::OP_BIT_AND => BitwiseOperation::And,
            Rule::OP_BIT_SL => BitwiseOperation::LeftShift,
            Rule::OP_BIT_SR => BitwiseOperation::RightShift,
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
    value = (this, state) {
        Value::bitwise_op(
            &this.left.get_value(state)?,
            &this.right.get_value(state)?,
            this.operator,
        )
        .with_context(this.token())
    },

    docs = {
        name: "Bitwise",
        symbols = ["|", "^", "&", "<<", ">>"],
        description: "
            A left-associative infix operator that performs bitwise operations on two values.
            Values are first converted to integers.
            Shifts are arithmetic for signed integers and logical for unsigned integers.
            A larger set of bitwise operations are available in the 'bitwise' category of the standard library.
        ",
        examples: "
            5 | 3 & 3
            5 ^ 3
            5 << 3 >> 3
        ",
    }
);

define_prattnode!(
    BitwiseNot { base: Node<'i> },
    rules = [PREFIX_BIT_NOT],
    new = (input) {
        let token = input.as_token();
        let mut children = input.into_inner();
        children.next(); // Skip the operator
        let base = children.next().unwrap().to_ast_node()?;
        Ok(Self { base, token }.boxed())
    },
    value = (this, state) {
        Value::bitwise_not(&this.base.get_value(state)?).with_context(this.token())
    },

    docs = {
        name: "Unary Bitwise Not",
        symbols = ["~"],
        description: "
            A prefix operator that performs bitwise NOT on a value.
            The value is first converted to an integer.
            A larger set of bitwise operations are available in the 'bitwise' category of the standard library.
        ",
        examples: "~5",
    }
);
