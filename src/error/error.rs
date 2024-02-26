use crate::{error::ErrorDetails, Token};

/// Error type for the Lavendeux parser
/// Can have optional context [Token], and parent error
#[derive(Debug)]
pub struct Error {
    /// details: The specific error that occurred - see [ErrorDetails]
    pub details: ErrorDetails,

    /// context: The [Token] that caused the error, or was being parsed when the error occurred
    pub context: Option<Token>,

    /// source: A parent error, if one exists - errors during a function call, for example
    pub source: Option<Box<Error>>,
}

impl Error {
    /// Add context to this error, in the form a [Token]
    pub fn with_context(self, context: Token) -> Self {
        Error {
            context: Some(context),
            ..self
        }
    }

    /// Link the parent error to this error
    pub fn with_source(self, source: Error) -> Self {
        Error {
            source: Some(Box::new(source)),
            ..self
        }
    }

    /// Remove context from this error
    pub fn without_context(self) -> Self {
        Error {
            context: None,
            ..self
        }
    }

    /// Offset the line-numbers in this and all parent errors
    /// Useful for when a script is included in another script
    /// Or for function calls
    pub fn offset_linecount(mut self, offset: usize) -> Self {
        let mut new_context = self.context.clone();
        if let Some(context) = &mut new_context {
            context.line += offset;
        }

        let mut new_source = std::mem::take(&mut self.source);
        if let Some(source) = new_source {
            new_source = Some(Box::new(source.offset_linecount(offset)));
        }

        Error {
            context: new_context,
            source: new_source,
            ..self
        }
    }
}

impl<T> From<T> for Error
where
    T: std::convert::Into<ErrorDetails>,
{
    fn from(details: T) -> Self {
        Error {
            details: details.into(),
            context: None,
            source: None,
        }
    }
}

impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let token_part = if let Some(context) = &self.context {
            format!("| {}\n= ", context)
        } else {
            "".to_string()
        };

        let source_part = if let Some(source) = &self.source {
            format!("\n{}", source)
        } else {
            "".to_string()
        };

        write!(f, "{}{}{}", token_part, self.details, source_part)
    }
}
