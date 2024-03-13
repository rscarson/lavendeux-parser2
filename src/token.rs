use crate::Rule;
use pest::iterators::Pair;
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
                | Rule::EOL
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

impl std::fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let lines = self
            .input
            .lines()
            .take(3)
            .map(|l| format!("| {l}"))
            .collect::<Vec<_>>();
        if lines.len() == 1 {
            write!(f, "Line {}: {}", self.line, self.input)
        } else {
            write!(f, "Line {}: \n{}", self.line, lines.join("\n"))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::pest::LavendeuxParser;
    use pest::Parser;

    fn get_pair() -> Pair<'static, Rule> {
        let mut pairs = LavendeuxParser::parse(Rule::SCRIPT, "1").unwrap();
        pairs.next().unwrap()
    }

    #[test]
    fn test_token_from_pair() {
        let token = Token::from(&get_pair());
        assert_eq!(token.line, 1);
        assert_eq!(token.rule, Rule::SCRIPT);
        assert_eq!(token.input, "1");
    }

    #[test]
    fn test_token_display() {
        let token = Token {
            line: 1,
            rule: Rule::symbol_arrow,
            input: Cow::Borrowed("->"),
        };
        assert_eq!(format!("{}", token), "Line 1: ->");

        let token = Token {
            line: 1,
            rule: Rule::symbol_arrow,
            input: Cow::Borrowed("->\n->"),
        };
        assert_eq!(format!("{}", token), "Line 1: \n| ->\n| ->");
    }

    #[test]
    fn test_token_into_owned() {
        let token = Token {
            line: 1,
            rule: Rule::symbol_arrow,
            input: Cow::Borrowed("->"),
        };
        let token = token.into_owned();
        assert_eq!(token.line, 1);
        assert_eq!(token.rule, Rule::symbol_arrow);
        assert_eq!(token.input, "->");
    }

    #[test]
    fn test_token_is_symbol() {
        assert!(Token::is_symbol(Rule::symbol_arrow));
        assert!(!Token::is_symbol(Rule::OP_ADD));
    }

    #[test]
    fn test_token_dummy() {
        let token = Token::dummy();
        assert_eq!(token.line, 0);
        assert_eq!(token.rule, Rule::SCRIPT);
        assert_eq!(token.input, "");
    }
}
