use crate::{error::WrapSyntaxError, token::ToToken, Error, Token};
use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "pre_grammar.pest"]
pub struct Preprocessor;

impl ToToken for Pair<'_, Rule> {
    fn to_token(&self) -> Token {
        Token {
            line: self.as_span().start_pos().line_col().0,
            rule: crate::Rule::SCRIPT,
            input: self.as_str().to_string(),
            references: None,
        }
    }
}

pub struct PreprocessorDirectives {
    pub script: String,
}

impl Preprocessor {
    pub fn process(input: &str) -> Result<PreprocessorDirectives, Error> {
        let mut directives = PreprocessorDirectives {
            script: String::new(),
        };

        let default_token = Token {
            line: 0,
            rule: crate::Rule::SCRIPT,
            input: input.to_string(),
            references: None,
        };

        let pairs = Preprocessor::parse(Rule::SCRIPT, input).wrap_syntax_error(input)?;

        let mut sq = 0;
        let mut pa = 0;
        let mut cu = 0;
        for pair in pairs {
            let text = pair.as_str().to_string();
            match pair.as_rule() {
                Rule::brack_open => pa += 1,
                Rule::brack_close => pa -= 1,

                Rule::brace_open => cu += 1,
                Rule::brace_close => cu -= 1,

                Rule::paren_open => sq += 1,
                Rule::paren_close => sq -= 1,

                _ => (),
            }

            directives.script.push_str(&text);
        }

        if pa > 0 {
            return Err(Error::UnterminatedParen {
                token: default_token,
            });
        } else if sq > 0 {
            return Err(Error::UnterminatedArray {
                token: default_token,
            });
        } else if cu > 0 {
            return Err(Error::UnterminatedObject {
                token: default_token,
            });
        }

        Ok(directives)
    }
}
