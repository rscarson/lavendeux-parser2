use super::Node;
use crate::{error::WrapExternalError, syntax_tree::traits::IntoNode, Rule};
use polyvalue::operations::{BitwiseOperation, BitwiseOperationExt};

define_ast!(Bitwise {
    BitwiseNot(value: Node<'i>) {
        build = (pairs, token, state) {
            pairs.next(); // Skip the operator
            let value = unwrap_node!(pairs, state, token)?;
            Ok(Self {
                value,
                token,
            }
            .into())
        },
        eval = (this, state) {
            let value = this.value.evaluate(state).with_context(this.token())?;
            value.bitwise_not().with_context(this.token())
        },
        owned = (this) {
            Self::Owned {
                value: this.value.into_owned(),
                token: this.token.into_owned(),
            }
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
    },

    BitwiseExpr(lhs: Node<'i>, op: BitwiseOperation, rhs: Node<'i>) {
        build = (pairs, token, state) {
            let lhs = unwrap_node!(pairs, state, token)?;

            let op = unwrap_next!(pairs, token);
            let op = match op.as_rule() {
                Rule::OP_BIT_OR => BitwiseOperation::Or,
                Rule::OP_BIT_XOR => BitwiseOperation::Xor,
                Rule::OP_BIT_AND => BitwiseOperation::And,
                Rule::OP_BIT_SL => BitwiseOperation::LeftShift,
                Rule::OP_BIT_SR => BitwiseOperation::RightShift,
                _ => {
                    return oops!(
                        Internal {
                            msg: format!("Unrecognize bitwise operator {}", op.as_str())
                        },
                        token
                    )
                }
            };

            let rhs = unwrap_node!(pairs, state, token)?;

            Ok(Self {lhs, op, rhs, token}.into())
        },
        eval = (this, state) {
            let lhs = this.lhs.evaluate(state).with_context(this.token())?;
            let rhs = this.rhs.evaluate(state).with_context(this.token())?;
            lhs.bitwise_op(rhs, this.op).with_context(this.token())
        },
        owned = (this) {
            Self::Owned {
                lhs: this.lhs.into_owned(),
                op: this.op,
                rhs: this.rhs.into_owned(),
                token: this.token.into_owned(),
            }
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
    }
});
