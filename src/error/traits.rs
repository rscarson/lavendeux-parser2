use crate::{error::ErrorDetails, pest::Rule, Error, Token};

/// Wraps a syntax error into an Error<'i>.
pub trait WrapSyntaxError<'i, T, R> {
    /// Turns a pest error into an Error<'i>.
    fn wrap_syntax_error(self, input: &'i str) -> Result<T, Error<'i>>;
}
impl<'i, T> WrapSyntaxError<'i, T, Rule> for Result<T, pest::error::Error<Rule>> {
    fn wrap_syntax_error(self, input: &'i str) -> Result<T, Error<'i>> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => {
                let span = match e.location {
                    pest::error::InputLocation::Pos(pos) => pos..(input.len()),
                    pest::error::InputLocation::Span(span) => span.0..span.1,
                };
                let span = &input[span];

                let line = match e.line_col {
                    pest::error::LineColLocation::Pos((line, _)) => line,
                    pest::error::LineColLocation::Span((line, _), _) => line,
                };

                let token = crate::Token {
                    line,
                    rule: crate::Rule::SCRIPT,
                    input: span.split('\n').next().unwrap_or_default(),
                    references: None,
                };

                oops!(Syntax, token)
            }
        }
    }
}

/// Wrap a 3rd party error into an Error<'i>.
pub trait WrapExternalError<'i, T> {
    /// Adds a context [Token]
    fn with_context(self, context: &Token<'i>) -> Result<T, Error<'i>>;

    /// Adds a source [Error<'i>]
    fn with_source(self, source: Error<'i>) -> Result<T, Error<'i>>;

    /// Wraps the error without context or a source
    fn without_context(self) -> Result<T, Error<'i>>;
}

impl<'i, T, E> WrapExternalError<'i, T> for Result<T, E>
where
    E: Into<Error<'i>>,
{
    fn with_context(self, context: &Token<'i>) -> Result<T, Error<'i>> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(e.into().with_context(context.clone())),
        }
    }

    fn with_source(self, source: Error<'i>) -> Result<T, Error<'i>> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(e.into().with_source(source)),
        }
    }

    fn without_context(self) -> Result<T, Error<'i>> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(e.into().without_context()),
        }
    }
}

/// Wrap an `Option<T>` into a `Result<T, Error<'i>>`
pub trait WrapOption<'i, T> {
    /// Turns an `Option<T>` into a `Result<T, Error<'i>>`
    fn or_error(self, error: ErrorDetails) -> Result<T, Error<'i>>;
}
impl<'i, T> WrapOption<'i, T> for Option<T> {
    fn or_error(self, error: ErrorDetails) -> Result<T, Error<'i>> {
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
