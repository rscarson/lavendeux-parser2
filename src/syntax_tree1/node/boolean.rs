use super::*;
use crate::{error::WrapExternalError, pest::Rule};
use polyvalue::{
    operations::{BooleanOperation, BooleanOperationExt},
    Value,
};

define_prattnode!(
    InfixBoolean {
        left: Node<'i>,
        right: Node<'i>,
        operator: BooleanOperation
    },
    rules = [
        OP_BOOL_OR,
        OP_BOOL_AND,
        OP_BOOL_EQ,
        OP_BOOL_NE,
        OP_BOOL_LE,
        OP_BOOL_GE,
        OP_BOOL_LT,
        OP_BOOL_GT
    ],
    new = (input) {
        let token = input.as_token();
        let mut children = input.into_inner();
        let left = children.next().unwrap().to_ast_node()?;
        let operator = children.next().unwrap().as_rule();
        let right = children.next().unwrap().to_ast_node()?;

        let operator = match operator {
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
                        msg: format!("Unrecognize boolean operator {operator:?}")
                    },
                    token
                )
            }
        };

        Ok(Self {
            left,
            right,
            operator,
            token: token,
        }
        .boxed())
    },
    value = (this, state) {
        // Short-circuit evaluation
        if this.operator == BooleanOperation::Or {
            let left = this.left.get_value(state)?;
            if left.is_truthy() {
                return Ok(true.into());
            }
        } else if this.operator == BooleanOperation::And {
            let left = this.left.get_value(state)?;
            if !left.is_truthy() {
                return Ok(false.into());
            }
        }

        Value::boolean_op(
            &this.left.get_value(state)?,
            &this.right.get_value(state)?,
            this.operator,
        )
        .with_context(this.token())
    },

    into_owned = (this) {
        Self {
            left: this.left.into_owned(),
            right: this.right.into_owned(),
            operator: this.operator,
            token: this.token.clone(),
        }
        .boxed()
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
);

define_prattnode!(
    BooleanNot { base: Node<'i> },
    rules = [PREFIX_BOOL_NOT],
    new = (input) {
        let token = input.as_token();
        let mut children = input.into_inner();
        children.next(); // Skip the operator
        let base = children.next().unwrap().to_ast_node()?;
        Ok(Self { base, token }.boxed())
    },
    value = (this, state) {
        Value::boolean_not(&this.base.get_value(state)?).with_context(this.token())
    },

    into_owned = (this) {
        Self {
            base: this.base.into_owned(),
            token: this.token.clone(),
        }
        .boxed()
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
);
