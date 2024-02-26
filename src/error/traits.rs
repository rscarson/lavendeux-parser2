use crate::{error::ErrorDetails, Error, Token};
pub trait WrapSyntaxError<T, R> {
    fn wrap_syntax_error(self, input: &str) -> Result<T, Error>;
}
impl<T, R> WrapSyntaxError<T, R> for Result<T, pest::error::Error<R>> {
    fn wrap_syntax_error(self, input: &str) -> Result<T, Error> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => {
                let span = match e.location {
                    pest::error::InputLocation::Pos(pos) => pos..(input.len()),
                    pest::error::InputLocation::Span(span) => span.0..span.1,
                };
                let span = input[span].to_string();

                let line = match e.line_col {
                    pest::error::LineColLocation::Pos((line, _)) => line,
                    pest::error::LineColLocation::Span((line, _), _) => line,
                };

                let token = crate::Token {
                    line,
                    rule: crate::Rule::SCRIPT,
                    input: span.split('\n').next().unwrap_or_default().to_string(),
                    references: None,
                };

                oops!(Syntax, token)
            }
        }
    }
}

pub trait WrapExternalError<T> {
    fn with_context(self, context: &Token) -> Result<T, Error>;
    fn with_source(self, source: Error) -> Result<T, Error>;
    fn without_context(self) -> Result<T, Error>;
}

impl<T, E> WrapExternalError<T> for Result<T, E>
where
    E: Into<Error>,
{
    fn with_context(self, context: &Token) -> Result<T, Error> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(e.into().with_context(context.clone())),
        }
    }

    fn with_source(self, source: Error) -> Result<T, Error> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(e.into().with_source(source)),
        }
    }

    fn without_context(self) -> Result<T, Error> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(e.into().without_context()),
        }
    }
}

pub trait WrapOption<T> {
    fn or_error(self, error: ErrorDetails) -> Result<T, Error>;
}
impl<T> WrapOption<T> for Option<T> {
    fn or_error(self, error: ErrorDetails) -> Result<T, Error> {
        match self {
            Some(v) => Ok(v),
            None => Err(Error {
                details: error,
                context: None,
                source: None,
            }),
        }
    }
}
