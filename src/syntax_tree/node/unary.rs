//! Unary Nodes
//!
//! Nodes for unary operations
//!
use super::*;
use crate::{error::WrapError, Rule, State, ToToken};
use pest::iterators::Pair;
use polyvalue::{operations::*, ValueType};

define_node!(
    IndexingExpression {
        base: Node,
        index: Node
    },
    rules = [INDEXING_EXPRESSION],
    new = |input: Pair<Rule>| {
        let token = input.to_token();
        let mut children = input.into_inner();

        let mut base = children.next().unwrap().to_ast_node()?;
        for index in children {
            let index = index.to_ast_node()?;
            base = Self {
                base,
                index,
                token: token.clone(),
            }
            .boxed();
        }

        Ok(base)
    },
    value = |this: &IndexingExpression, state: &mut State| {
        let base = this.base.get_value(state)?;
        let index = this.index.get_value(state)?;

        if index.is_a(ValueType::Compound) && !index.is_a(ValueType::String) {
            Ok(base.get_indices(&index).to_error(&this.token)?)
        } else {
            Ok(base.get_index(&index).to_error(&this.token)?)
        }
    }
);

#[cfg(test)]
mod test {
    use super::*;
    use crate::assert_tree;

    #[test]
    fn test_indexing_expr() {
        assert_tree!(
            "a[0]",
            INDEXING_EXPRESSION,
            IndexingExpression,
            |node: &mut IndexingExpression| {
                assert_eq!(node.base.to_string(), "a");
                assert_eq!(node.index.to_string(), "0");
            }
        );

        assert_tree!(
            "a[0][1]",
            INDEXING_EXPRESSION,
            IndexingExpression,
            |node: &mut IndexingExpression| {
                assert_tree!(
                    &mut node.base,
                    IndexingExpression,
                    |node: &mut IndexingExpression| {
                        assert_eq!(node.base.to_string(), "a");
                        assert_eq!(node.index.to_string(), "0");
                    }
                );
                assert_eq!(node.index.to_string(), "1");
            }
        );
    }
}
