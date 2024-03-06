use super::Node;
use crate::{error::WrapExternalError, syntax_tree::traits::IntoNode, Rule};
use polyvalue::operations::{BooleanOperation, BooleanOperationExt};

define_ast!(Boolean {
    BooleanNot(value: Box<Node<'i>>) {
        build = (pairs, token, state) {
            pairs.next(); // Skip the operator
            let value = pairs.next().unwrap().into_node(state).with_context(&token)?;
            Ok(Self {
                value: Box::new(value),
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
                value: Box::new(this.value.into_owned()),
                token: this.token.into_owned(),
            }
            .into()
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

    BooleanExpr(lhs: Box<Node<'i>>, op: BooleanOperation, rhs: Box<Node<'i>>) {
        build = (pairs, token, state) {
            let lhs = pairs.next().unwrap().into_node(state).with_context(&token)?;

            let op = pairs.next().unwrap();
            let op = match op.as_rule() {
                Rule::OP_BOOL_OR => BooleanOperation::Or,
                Rule::OP_BOOL_AND => BooleanOperation::And,
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

            let rhs = pairs.next().unwrap().into_node(state).with_context(&token)?;

            Ok(Self {
                lhs: Box::new(lhs),
                op: op.into(),
                rhs: Box::new(rhs),
                token,
            }
            .into())
        },
        eval = (this, state) {
            let lhs = this.lhs.evaluate(state).with_context(this.token())?;
            let rhs = this.rhs.evaluate(state).with_context(this.token())?;
            lhs.boolean_op(rhs, this.op).with_context(this.token())
        },
        owned = (this) {
            Self::Owned {
                lhs: Box::new(this.lhs.into_owned()),
                op: this.op,
                rhs: Box::new(this.rhs.into_owned()),
                token: this.token.into_owned(),
            }
            .into()
        },

        docs = {
            name: "Boolean",
            symbols = ["or", "and", "==", "!=", "<=", ">=", "<", ">"],
            description: "
                Performs an infix boolean comparison between two values.
                Comparisons are weak, meaning that the types of the values are not checked.
                Result are always a boolean value.
                And and Or are short-circuiting.
                All are left-associative.
            ",
            examples: "
                true || false
                1 < 2
            ",
        }
    }
});
