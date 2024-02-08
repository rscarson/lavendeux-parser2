use crate::{error::WrapError, token::ToToken, Error, Token};
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
    pub consts: Vec<(String, String)>,
}

impl Preprocessor {
    pub fn process(input: &str) -> Result<PreprocessorDirectives, Error> {
        let mut directives = PreprocessorDirectives {
            script: String::new(),
            consts: Vec::new(),
        };

        let default_token = Token {
            line: 0,
            rule: crate::Rule::SCRIPT,
            input: input.to_string(),
            references: None,
        };

        let pairs = match Preprocessor::parse(Rule::file, input) {
            Ok(pairs) => pairs,
            Err(err) => {
                let span = match err.location {
                    pest::error::InputLocation::Pos(pos) => pos..=(input.len() - 1),
                    pest::error::InputLocation::Span(span) => span.0..=span.1,
                };
                let span = input[span].to_string();

                let line = match err.line_col {
                    pest::error::LineColLocation::Pos((line, _)) => line,
                    pest::error::LineColLocation::Span((line, _), _) => line,
                };

                return Err(Error::Syntax { line, span });
            }
        };

        let mut sq = 0;
        let mut pa = 0;
        let mut cu = 0;
        for pair in pairs {
            let mut text = pair.as_str().to_string();
            let token = pair.to_token();
            match pair.as_rule() {
                Rule::directive => {
                    let mut children = pair.into_inner();
                    let name = children.next().unwrap().as_str();
                    let value = children.next().unwrap().as_str().to_string();
                    let value = &value[1..value.len() - 1].to_string();

                    match name {
                        "include" => {
                            let include = std::fs::read_to_string(value).to_error(&token)?;
                            text = include;
                        }
                        "define" => {
                            let value = value.split("=").map(|s| s.trim()).collect::<Vec<&str>>();
                            if value.len() != 2 {
                                return Err(Error::InvalidDirective {
                                    token: token.clone(),
                                    directive: text,
                                });
                            }
                            directives
                                .consts
                                .push((value[0].to_string(), value[1].to_string()));
                        }
                        _ => (),
                    }
                }

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
