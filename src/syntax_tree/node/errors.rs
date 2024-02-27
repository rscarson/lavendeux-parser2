//! Error Nodes
//!
//! High-level nodes that are used to build the syntax tree.
//!
use super::*;
use crate::{Rule, State, ToToken};
use pest::iterators::Pair;

macro_rules! define_errornode {
    ($rule:ident, $error:ident) => {
        define_node!(
            $error,
            rules = [$rule],
            new = |input: Pair<Rule>| { oops!($error, input.to_token()) },
            value = |_: &Self, _: &mut State| unreachable!()
        );
    };
}

define_errornode!(UNTERMINATED_BLOCK_COMMENT, UnterminatedComment);
define_errornode!(UNTERMINATED_STRING_LITERAL, UnterminatedLiteral);
define_errornode!(UNCLOSED_BRACKET, UnterminatedArray);
define_errornode!(UNCLOSED_BRACE, UnterminatedObject);
define_errornode!(UNCLOSED_PAREN, UnterminatedParen);
define_errornode!(MISSING_LINEBREAK, UnterminatedLinebreak);
