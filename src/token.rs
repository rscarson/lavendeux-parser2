use crate::Rule;
use pest::iterators::Pair;

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
    pub input: &'i str,

    /// An optional variable reference, for pass-by-ref
    /// Used by a handful of stdlib functions, like push, insert, etc
    pub references: Option<String>,
}

impl Token<'_> {
    #[cfg(test)]
    /// Create a dummy token
    pub fn dummy() -> Self {
        Self {
            line: 0,
            rule: Rule::SCRIPT,
            input: "",
            references: None,
        }
    }
}

/// A trait used to convert a pest pair into a token
pub trait ToToken<'i> {
    fn to_token(&self) -> Token<'i>;
}
impl<'i> ToToken<'i> for Pair<'i, Rule> {
    fn to_token(&self) -> Token<'i> {
        Token {
            line: self.as_span().start_pos().line_col().0,
            rule: self.as_rule(),
            input: self.as_str(),
            references: None,
        }
    }
}

impl std::fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Line {}: {}", self.line, self.input)
    }
}
