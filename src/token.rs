use crate::Rule;
use pest::iterators::{Pair, Pairs};
use std::borrow::Cow;

/// A struct representing a token in the syntax tree
/// It stores metadata about the token, such as the rule it was parsed from,
/// the input it was parsed from, and the references to variables it contains
#[derive(Debug, Clone)]
pub struct Token<'i> {
    /// Source-code line number
    pub line: usize,

    /// Grammar-rule that this token was parsed from
    /// See [crate::Rule]
    pub rule: Rule,

    /// The input that this token was parsed from
    pub input: Cow<'i, str>,
}

impl Token<'_> {
    /// Creates a new root token from the input
    pub fn dummy() -> Self {
        Token {
            line: 0,
            rule: Rule::SCRIPT,
            input: Cow::Borrowed(""),
        }
    }

    /// Check if this token is a symbol
    pub fn is_symbol(rule: Rule) -> bool {
        matches!(
            rule,
            Rule::symbol_questionmark
                | Rule::symbol_colon
                | Rule::symbol_comma
                | Rule::symbol_arrow
                | Rule::symbol_at
                | Rule::symbol_eq
                | Rule::symbol_opencurly
                | Rule::symbol_closecurly
                | Rule::symbol_opensquare
                | Rule::symbol_closesquare
                | Rule::symbol_openround
                | Rule::symbol_closeround
        )
    }

    /// Remove lifetime restrictions from this token
    pub fn into_owned(self) -> Token<'static> {
        Token {
            line: self.line,
            rule: self.rule,
            input: Cow::Owned(self.input.into_owned()),
        }
    }
}

impl<'i> From<&Pair<'i, Rule>> for Token<'i> {
    fn from(pair: &Pair<'i, Rule>) -> Token<'i> {
        Token {
            line: pair.line_col().0,
            rule: pair.as_rule(),
            input: Cow::Borrowed(pair.as_str().trim()),
        }
    }
}

impl<'i> From<&Pairs<'i, Rule>> for Token<'i> {
    fn from(pairs: &Pairs<'i, Rule>) -> Token<'i> {
        if let Some(pair) = pairs.peek() {
            Token::from(&pair)
        } else {
            Token::dummy()
        }
    }
}

impl std::fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Line {}: {}", self.line, self.input)
    }
}
