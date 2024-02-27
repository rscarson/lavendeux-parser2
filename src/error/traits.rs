use crate::{error::ErrorDetails, pest::Rule, Error, Token};

/// Wraps a syntax error into an Error.
pub trait WrapSyntaxError<T, R> {
    /// Turns a pest error into an Error.
    fn wrap_syntax_error(self, input: &str) -> Result<T, Error>;
}
impl<T> WrapSyntaxError<T, Rule> for Result<T, pest::error::Error<Rule>> {
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

/// Wrap a 3rd party error into an Error.
pub trait WrapExternalError<T> {
    /// Adds a context [Token]
    fn with_context(self, context: &Token) -> Result<T, Error>;

    /// Adds a source [Error]
    fn with_source(self, source: Error) -> Result<T, Error>;

    /// Wraps the error without context or a source
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

/// Wrap an `Option<T>` into a `Result<T, Error>`
pub trait WrapOption<T> {
    /// Turns an `Option<T>` into a `Result<T, Error>`
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
