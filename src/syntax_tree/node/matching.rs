use super::*;
use crate::{error::WrapExternalError, pest::Rule, State};
use polyvalue::{
    operations::{MatchingOperation, MatchingOperationExt},
    Value,
};

define_prattnode!(
    InfixMatch {
        left: Node,
        right: Node,
        operator: MatchingOperation
    },
    rules = [
        OP_MATCH_CONTAINS,
        OP_MATCH_MATCHES,
        OP_MATCH_IS,
        OP_MATCH_STARTSWITH,
        OP_MATCH_ENDSWITH
    ],
    new = |input: PrattPair| {
        let token = input.as_token();
        let mut children = input.into_inner();
        let left = children.next().unwrap().to_ast_node()?;
        let operator = children.next().unwrap().as_rule();
        let right = children.next().unwrap().to_ast_node()?;

        let operator = match operator {
            Rule::OP_MATCH_CONTAINS => MatchingOperation::Contains,
            Rule::OP_MATCH_MATCHES => MatchingOperation::Matches,
            Rule::OP_MATCH_IS => MatchingOperation::Is,
            Rule::OP_MATCH_STARTSWITH => MatchingOperation::StartsWith,
            Rule::OP_MATCH_ENDSWITH => MatchingOperation::EndsWith,
            _ => {
                return oops!(
                    Internal {
                        msg: format!("Unrecognize matching operator {operator:?}")
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
    value = |this: &Self, state: &mut State| {
        let left = this.left.get_value(state)?;
        let right = if this.operator == MatchingOperation::Is
            && this.right.token().rule == Rule::identifier
        {
            Value::from(this.right.token().input.as_str())
        } else {
            this.right.get_value(state)?
        };

        Value::matching_op(&left, &right, this.operator).with_context(this.token())
    },

    docs = {
        name: "Matching",
        symbols = ["contains", "matches", "is", "starts_with", "ends_with"],
        description: "
            A set of left-associative boolean operators comparing a collection with a pattern
            'is' is a special case that compares type (`value is string` is equivalent `typeof(value) == 'string'`)
            starts/ends with are not applicable to objects, which are not ordered
        ",
        examples: "
            {'name': 'test'} contains 'name'
            'hello' matches 'ell'
            'hello' is string
            'hello' starts_with 'hel'
            [1, 2] endswith 2
        ",
    }
);
