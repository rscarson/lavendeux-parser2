use super::Node;
use crate::{error::WrapExternalError, syntax_tree::traits::IntoNode, Rule};
use polyvalue::operations::{BooleanOperation, BooleanOperationExt};

define_ast!(Boolean {
    BooleanNot(value: Node<'i>) {
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
            value.boolean_not().with_context(this.token())
        },
        owned = (this) {
            Self::Owned {
                value: this.value.into_owned(),
                token: this.token.into_owned(),
            }
        },

        docs = {
            name: "Unary Boolean Not",
            symbols = ["not"],
            description: "
                Negates a boolean value.
                If the value is not a boolean, it is cooerced to boolean first.
            ",
            examples: "
                !true == false
                !'test' == false
                !0 == true
            ",
        }
    },

    BooleanExpr(lhs: Node<'i>, op: BooleanOperation, rhs: Node<'i>) {
        build = (pairs, token, state) {
            let lhs = unwrap_node!(pairs, state, token)?;

            let op = unwrap_next!(pairs, token);
            let op = match op.as_rule() {
                Rule::OP_BOOL_OR => BooleanOperation::Or,
                Rule::OP_BOOL_AND => BooleanOperation::And,
                Rule::OP_BOOL_SEQ => BooleanOperation::StrictEQ,
                Rule::OP_BOOL_SNE => BooleanOperation::StrictNEQ,
                Rule::OP_BOOL_EQ => BooleanOperation::EQ,
                Rule::OP_BOOL_NE => BooleanOperation::NEQ,
                Rule::OP_BOOL_LE => BooleanOperation::LTE,
                Rule::OP_BOOL_GE => BooleanOperation::GTE,
                Rule::OP_BOOL_LT => BooleanOperation::LT,
                Rule::OP_BOOL_GT => BooleanOperation::GT,
                _ => {
                    return oops!(
                        Internal {
                            msg: format!("Unrecognize boolean operator {}", op.as_str())
                        },
                        token
                    )
                }
            };

            let rhs = unwrap_node!(pairs, state, token)?;

            Ok(Self { lhs, op, rhs, token }.into())
        },
        eval = (this, state) {
            let lhs = this.lhs.evaluate(state).with_context(this.token())?;

            // Short-circuiting
            if this.op == BooleanOperation::Or && lhs.is_truthy() {
                return Ok(true.into())
            } else if this.op == BooleanOperation::And && !lhs.is_truthy() {
                return Ok(false.into())
            }

            let rhs = this.rhs.evaluate(state).with_context(this.token())?;
            lhs.boolean_op(rhs, this.op).with_context(this.token())
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
            name: "Boolean",
            symbols = ["or", "and", "==", "!=", "===", "!==", "<=", ">=", "<", ">"],
            description: "
                Performs an infix boolean comparison between two values.
                Comparisons are weak, meaning that the types of the values are not checked.
                Result are always a boolean value.
                And and Or are short-circuiting.
                All are left-associative.

                Most comparisons are weak, meaning that the types of the values are not checked.
                Strict comparisons (=== and !==) are also available, which checks the type of the values before comparing.
                Comparing collections (arrays and objects) will perform strict comparisons on their contents.
            ",
            examples: "
                true || false
                1 < 2
                assert(false == 0)
                assert(false !== 0)
            ",
        }
    }
});
