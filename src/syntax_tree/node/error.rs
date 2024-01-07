//! Error nodes
//!
//! These nodes are used to represent errors in the syntax tree, and hold
//! no value. They are used to exit early from the parsing process in a
//! more useful way
use crate::{AstNode, Error, Rule};

macro_rules! error_node {
    ($name:ident, $rule:ident) => {
        define_node!(
            $name,
            rules = [$rule],
            new = |_: pest::iterators::Pair<Rule>| { Err(Error::$name) },
            value = |_: &Self, _state: &mut crate::State| {
                Err(Error::Internal(
                    "Attempt to get value from an error".to_string(),
                ))
            }
        );
    };
}

error_node!(UnterminatedLinebreak, ERROR_UNTERMINATED_LINEBREAK);
error_node!(UnterminatedLiteral, ERROR_UNTERMINATED_LITERAL);
error_node!(UnterminatedComment, ERROR_UNTERMINATED_COMMENT);
error_node!(UnterminatedArray, ERROR_UNTERMINATED_ARRAY);
error_node!(UnterminatedObject, ERROR_UNTERMINATED_OBJECT);
error_node!(UnterminatedParen, ERROR_UNTERMINATED_PAREN);
error_node!(UnexpectedDecorator, ERROR_UNEXPECTED_DECORATOR);

#[cfg(test)]
mod test {
    use super::*;
    use crate::assert_tree_error;

    #[test]
    fn test_errors() {
        assert_tree_error!("1 +\\", UnterminatedLinebreak);
        assert_tree_error!("1 + \"", UnterminatedLiteral);
        assert_tree_error!("1 + /*", UnterminatedComment);
        assert_tree_error!("1 + [", UnterminatedArray);
        assert_tree_error!("1 + {", UnterminatedObject);
        assert_tree_error!("1 + (", UnterminatedParen);
        assert_tree_error!("@1 + 1", UnexpectedDecorator);
    }
}
