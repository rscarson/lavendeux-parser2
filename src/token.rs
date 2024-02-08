use crate::Rule;
use pest::iterators::Pair;

/// A struct representing a token in the syntax tree
/// It stores metadata about the token, such as the rule it was parsed from,
/// the input it was parsed from, and the references to variables it contains
#[derive(Debug, Clone)]
pub struct Token {
    pub line: usize,
    pub rule: Rule,
    pub input: String,

    /// An optional variable reference, for pass-by-ref
    pub references: Option<String>,
}

impl Token {
    pub fn dummy() -> Self {
        Self {
            line: 0,
            rule: Rule::SCRIPT,
            input: "".to_string(),
            references: None,
        }
    }
}

/// A trait used to convert a pest pair into a token
pub trait ToToken {
    fn to_token(&self) -> Token;
}
impl ToToken for Pair<'_, Rule> {
    fn to_token(&self) -> Token {
        Token {
            line: self.as_span().start_pos().line_col().0,
            rule: self.as_rule(),
            input: self.as_str().to_string(),
            references: None,
        }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Line {}: {}", self.line + 1, self.input)
    }
}
