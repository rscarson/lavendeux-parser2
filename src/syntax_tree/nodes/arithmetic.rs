use super::Node;
use crate::{
    error::{ErrorDetails, WrapExternalError, WrapOption},
    syntax_tree::{assignment_target::AssignmentTarget, traits::IntoNode},
    Rule,
};
use polyvalue::{
    operations::{ArithmeticOperation, ArithmeticOperationExt},
    Value,
};

#[derive(Clone, Debug)]
pub enum IncDecType {
    PreI,
    PreD,
    PostI,
    PostD,
}
impl IncDecType {
    fn is_pre(&self) -> bool {
        matches!(self, Self::PreI | Self::PreD)
    }
    fn operation(&self) -> ArithmeticOperation {
        if matches!(self, Self::PreI | Self::PostI) {
            ArithmeticOperation::Add
        } else {
            ArithmeticOperation::Subtract
        }
    }
}

define_ast!(
    Arithmetic {
        IncDec(target: AssignmentTarget<'i>, variant: IncDecType) {
            build = (pairs, token, state) {
                let (op, value) = if matches!(token.rule, Rule::PREFIX_INC | Rule::PREFIX_DEC) {
                    (unwrap_next!(pairs, token).as_rule(), unwrap_node!(pairs, state, token)?)
                } else {
                    (token.rule, unwrap_node!(pairs, state, token)?)
                };

                let target = as_reference!(value).or_error(ErrorDetails::ConstantValue).with_context(&token)?;
                let variant = match op {
                    Rule::PREFIX_INC => IncDecType::PreI,
                    Rule::PREFIX_DEC => IncDecType::PreD,
                    Rule::POSTFIX_INC => IncDecType::PostI,
                    _ => IncDecType::PostD,
                };

                Ok(Self {
                    target,
                    variant,
                    token,
                }.into())
            },
            eval = (this, state) {
                let value = this.target.get_value(state).with_context(this.token())?;
                let increment = Value::from(1).as_type(value.own_type()).with_context(this.token())?;
                let operation = this.variant.operation();

                let new_value = value.clone().arithmetic_op(increment, operation)?;
                this.target.update_value(state, new_value.clone()).with_context(this.token())?;

                if this.variant.is_pre() {
                    Ok(new_value)
                } else {
                    Ok(value)
                }
            },
            owned = (this) {
                Self::Owned {
                    target: this.target.into_owned(),
                    variant: this.variant,
                    token: this.token.into_owned(),
                }
            },

            docs = {
                name: "Increment/Decrement",
                symbols = ["++", "--"],
                description: "
                    Increments or decrements a value.
                    If used as a prefix, the value is updated before the expression is evaluated.
                    If used as a postfix, the value is updated after the expression is evaluated.
                ",
                examples: "
                    a = 1; assert_eq(a++, 1);
                    a = 1; assert_eq(--a, 0);
                ",
            }
        },

        ArithmeticNeg(value: Node<'i>) {
            build = (pairs, token, state) {
                pairs.next(); // Skip the operator
                let value = unwrap_node!(pairs, state, token)?;
                Ok(Self {
                    value: value,
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
                    value: this.value.into_owned(),
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

        ArithmeticExpr(lhs: Node<'i>, op: ArithmeticOperation, rhs: Node<'i>) {
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
                    lhs: lhs,
                    op,
                    rhs: rhs,
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
                    lhs: this.lhs.into_owned(),
                    op: this.op,
                    rhs: this.rhs.into_owned(),
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
