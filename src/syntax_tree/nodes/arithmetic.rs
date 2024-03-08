use super::Node;
use crate::{error::WrapExternalError, syntax_tree::traits::IntoNode, Rule};
use polyvalue::{
    operations::{ArithmeticOperation, ArithmeticOperationExt},
    Value,
};

define_ast!(
    Arithmetic {
        ArithmeticNeg(value: Box<Node<'i>>) {
            build = (pairs, token, state) {
                pairs.next(); // Skip the operator
                let value = unwrap_node!(pairs, state, token)?;
                Ok(Self {
                    value: Box::new(value),
                    token,
                }
                .into())
            },
            eval = (this, state) {
                let value = this.value.evaluate(state).with_context(this.token())?;
                value.arithmetic_neg().with_context(this.token())
            },
            owned = (this) {
                Self::Owned {
                    value: Box::new(this.value.into_owned()),
                    token: this.token.into_owned(),
                }
            },

            docs = {
                name: "Unary Negation",
                symbols = ["-"],
                description: "Negates a value.",
                examples: "-1",
            }
        },

        ArithmeticExpr(lhs: Box<Node<'i>>, op: ArithmeticOperation, rhs: Box<Node<'i>>) {
            build = (pairs, token, state) {
                let lhs = unwrap_node!(pairs, state, token)?;

                let op = unwrap_next!(pairs, token);
                let op = match op.as_rule() {
                    Rule::OP_ADD => ArithmeticOperation::Add,
                    Rule::OP_SUB => ArithmeticOperation::Subtract,
                    Rule::OP_POW => ArithmeticOperation::Exponentiate,
                    Rule::OP_DIV => ArithmeticOperation::Divide,
                    Rule::OP_MOD => ArithmeticOperation::Modulo,
                    Rule::OP_MUL => ArithmeticOperation::Multiply,
                    _ => {
                        return oops!(
                            Internal {
                                msg: format!("Unrecognize arithmetic operator {}", op.as_str())
                            },
                            token
                        )
                    }
                };

                let rhs = unwrap_node!(pairs, state, token)?;

                Ok(Self {
                    lhs: Box::new(lhs),
                    op,
                    rhs: Box::new(rhs),
                    token,
                }
                .into())
            },
            eval = (this, state) {
                let lhs = this.lhs.evaluate(state).with_context(this.token())?;
                let rhs = this.rhs.evaluate(state).with_context(this.token())?;
                lhs.arithmetic_op(rhs, this.op).with_context(this.token())
            },
            owned = (this) {
                Self::Owned {
                    lhs: Box::new(this.lhs.into_owned()),
                    op: this.op,
                    rhs: Box::new(this.rhs.into_owned()),
                    token: this.token.into_owned(),
                }
            },

            docs = {
                name: "Arithmetic Expression",
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
        }
    }
);

#[cfg(test)]
mod test {
    use crate::lav;

    lav!(test_negation(a = -1i64, b = 1i64) r#"
        a = -1;
        b = -a
    "#);

    lav!(test_expr(a = 8i64, b = 0i64, c = 8i64) r#"
        a = 2 + 3 * 2;
        b = 2 - 4 / 2;
        c = 2 ** 3;
    "#);
}
