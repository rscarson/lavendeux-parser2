use crate::Token;
use polyvalue::{Value, ValueType};
use thiserror::Error;

/// A stub of rustyscript::Error for use when the extensions feature is disabled
#[cfg(not(feature = "extensions"))]
pub mod rustyscript {
    use std::fmt;

    #[derive(Debug)]
    pub struct Error;

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Extensions are not enabled")
        }
    }

    impl std::error::Error for Error {}
}

const BUG_REPORT_URL : &str = "https://github.com/rscarson/lavendeux-parser/issues/new?assignees=&labels=&template=bug_report.md&title=";

/// Represents the errors that can occur during parsing
#[derive(Error, Debug)]
#[rustfmt::skip]
pub enum Error {
    /// An error caused by a problem with the parser itself
    #[error(
        "Internal parser issue: {0}\nPlease report this problem at {}",
        BUG_REPORT_URL
    )]
    Internal(String),

    /// Error causing the parser thread to panic
    #[error("Fatal error: {0}")]
    Fatal(String),

    /// A timeout error caused by a script taking too long to execute
    #[error("Script execution timed out")]
    Timeout,

    /// An error caused by a custom error message
    #[error("\n| {token}\n= {message}")]
    Custom{
        message: String,
        token: Token,
    },

    /// An error caused by a problem with the syntax of the script
    #[error("\n| {token}\n= Invalid preprocessor directive: {directive}")]
    InvalidDirective {
        token: Token,
        directive: String,
    },

    /// An error used to return a value from a function early
    #[error("\n| {token}\n= Returned from the root scope")]
    Return {
        value: Value,
        token: Token,
    },

    /// An error used to skip a value from a loop
    #[error("\n| {token}\n= Skipped from outside a loop")]
    Skip {
        token: Token,
    },

    /// An error used to skip a value from a loop
    #[error("\n| {token}\n= Break called from outside a loop")]
    Break {
        token: Token,
    },

    ///////////////////////////////////////////////////////////////////////////
    // Syntax Errors
    // Deals with issues during Pest tree parsing
    ///////////////////////////////////////////////////////////////////////////

    /// An error caused by a problem with the syntax of the script
    #[error("\n| {token}\n= Did not specify a value for return")]
    UnterminatedReturn {
        token: Token,
    },

    /// An error caused by using a decorator in the wrong place
    #[error("\n| {token}\n= @decorators must be at the end of a statement")]
    UnexpectedDecorator {
        token: Token,
    },

    /// An error caused by using a postfix operator without an operand
    #[error("\n| {token}\n= Expected '*/'")]
    UnterminatedComment{
        token: Token,
    },

    /// An error caused by a missing bracket
    #[error("\n| {token}\n= Expected ']'")]
    UnterminatedArray{
        token: Token,
    },

    /// An error caused by a missing brace
    #[error("\n| {token}\n= Expected '}}'")]
    UnterminatedObject{
        token: Token,
    },

    /// An error caused by ending a script on a backslash
    #[error("\n| {token}\n= Missing linebreak after '\\'")]
    UnterminatedLinebreak{
        token: Token,
    },

    /// An error caused by a missing quote
    #[error("\n| {token}\n= Expected ' or \"")]
    UnterminatedLiteral{
        token: Token,
    },

    /// An error caused by a missing parentheses
    #[error("\n| {token}\n= Expected ')'")]
    UnterminatedParen{
        token: Token,
    },

    #[error("\n| {token}\n= Expected a pattern to match against (an array, value, or regex literal)")]
    IncompleteMatchingExpression{
        token: Token,
    },

    #[error("\n| {token}\n= Expected 2 bounds for range expression, for example: 1..2 or 'a'..'z'")]
    IncompleteRangeExpression{
        token: Token,
    },

    #[error("\n| {token}\n= Expected a key-value pair, for example: {{0: 'test'}}")]
    IncompleteObject {
        token: Token,
    },

    #[error("\n| {token}\n= Match expression is not exhaustive. Add a default case '_' to match all values")]
    NonExhaustiveSwitch {
        token: Token,
    },

    #[error("\n| {token}\n= All cases after the default case '_' are unreachable")]
    UnreachableSwitchCase {
        token: Token,
    },

    #[error("\n| {token}\n= {case} is not valid for this switch statement. Expected a {expected_type}")]
    SwitchCaseTypeMismatch {
        case: Value,
        expected_type: ValueType,
        token: Token,
    },

    ///////////////////////////////////////////////////////////////////////////
    // Value Errors
    // Mostly deals with variables, and value objects
    ///////////////////////////////////////////////////////////////////////////
    
    #[error("\n| {token}\n= Implicit multiplication is not allowed between {left} and {right}")]
    IllegalImplicitMultiplication {
        left: String, right: String, token: Token
    },

    #[error("\n| {token}\n= Invalid combination of types for range. Use a pair of either integers, or characters")]
    RangeTypeMismatch {
        token: Token,
    },

    #[error("\n| {token}\n= Invalid values for range: {start} > {end}")]
    InvalidRange {
        start: String,
        end: String,
        token: Token,
    },

    #[error("\n| {token}\n= Arithmetic overflow")]
    Overflow {
        token: Token,
    },

    #[error("\n| {token}\n= Expected {expected_length} values, found {actual_length}")]
    DestructuringAssignment {
        expected_length: usize,
        actual_length: usize,
        token: Token,
    },

    #[error("\n| {token}\n= Input could not be parsed as {expected_format}")]
    ValueFormat {
        expected_format: String,
        token: Token,
    },

    #[error("\n| {token}\n= {input} was out of range")]
    Range {
        input: String,
        token: Token,
    },

    #[error("\n| {token}\n= Undefined variable {name}. You can assign a value with {name} = ...")]
    VariableName {
        name: String,
        token: Token,
    },

    #[error("\n| {token}\n= Array empty")]
    ArrayEmpty {
        token: Token,
    },

    ///////////////////////////////////////////////////////////////////////////
    // Function Errors
    // Deals with issues during builtin, user, or extension function calls
    ///////////////////////////////////////////////////////////////////////////

    #[error("\n| {token}\n= Error in `{name}()`: {source}")]
    FunctionCall {
        name: String,
        source: Box<Error>,
        token: Token,
    },

    #[error("\n| {token}\n= Recursive function went too deep")]
    StackOverflow {
        token: Token,
    },
    
    #[error("\n| {token}\n= Expected {expected_type} value for argument {arg} of `{signature}`")]
    FunctionArgumentType {
        /// Argument number causing the issue (1-based)
        arg: usize,

        /// Type that was requested
        expected_type: ValueType,
        
        /// Signature of the function called
        signature: String,

        token: Token,
    },

    #[error("\n| {token}\n= Undefined function {name}. You can define a function with {name}(a, b, c) = ...")]
    FunctionName {
        name: String,
        token: Token,
    },

    /// An error caused by calling a function using the wrong number of arguments
    #[error(
        "Expected {} arguments for `{signature}`",
        if min == max {format!("{}", min)} else {format!("{}-{}", min, max)}
    )]
    FunctionArguments {
        /// Smallest number of arguments accepted by the function
        min: usize,
        
        /// Largest number of arguments accepted by the function
        max: usize, 
        
        
        /// Signature of the function called
        signature: String,
        
        token: Token,
    },

    /// An error caused by calling a decorator that does not exist
    #[error("\n| {token}\n= No decorator named @{name}")]
    DecoratorName {
        name: String,
        token: Token,
    },
    
    /// An error caused by attempting to use an API without registering it
    #[error("\n| {token}\n= API {name} was not found. Add it with api_register(\"{name}\", base_url, [optional api key])")]
    UnknownApi {
        name: String,
        token: Token,
    },

    ///////////////////////////////////////////////////////////////////////////
    // External Errors
    // Deals with issues inside dependencies
    ///////////////////////////////////////////////////////////////////////////
    
    /// An error caused by a problem with the syntax of the script
    #[error("\n| Line {line}: {}\n= Syntax error; unexpected token {}", 
    span.split('\n').next().unwrap_or(""),
    if span.is_empty() {
        "end-of-input".to_string()
    } else {
        span.chars().next().unwrap().to_string()
    } )]
    Syntax {
        line: usize,
        span: String 
    },

    /// Error dealing with 3rd party lib issues
    #[error("\n| {token}\n= {error}")]
    External {
        error: ExternalError,
        token: Token,
    },

    /// Error dealing with 3rd party lib issues
    #[error("{0}")]
    ExternalNoToken(#[from] ExternalError),
}

#[derive(Error, Debug)]
#[rustfmt::skip]
pub enum ExternalError {
    #[error("{0}")]
    Internal(#[from] Box<Error>),

    /// Error dealing with polyvalue issues
    #[error("{0}")]
    Value(#[from] polyvalue::Error),

    /// Error dealing with filesystem issues
    #[error("{0}")]
    Io(#[from] std::io::Error),

    /// Error dealing with network issues from the reqwest crate
    #[error("{0}")]
    Network(#[from] reqwest::Error),

    /// Error dealing with JS execution issues
    #[error("{0}")]
    Javascript(#[from] rustyscript::Error),

    /// Error dealing with int parsing issues
    #[error("{0}")]
    ParseIntError(#[from] std::num::ParseIntError),

    /// Error dealing with utf8 issues
    #[error("{0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),

    /// Error dealing with json issues
    #[error("{0}")]
    SerdeJsonError(#[from] serde_json::Error),
}

impl ExternalError {
    #[allow(clippy::wrong_self_convention)]
    pub fn to_error(self, token: &Token) -> Error {
        Error::External {
            error: self,
            token: token.clone(),
        }
    }
}

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

                Err(Error::Syntax { line, span })
            }
        }
    }
}

pub trait WrapError<T> {
    fn to_error(self, token: &Token) -> Result<T, Error>;
}

impl<T> WrapError<T> for Result<T, ExternalError> {
    fn to_error(self, token: &Token) -> Result<T, Error> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(Error::External {
                error: e,
                token: token.clone(),
            }),
        }
    }
}

macro_rules! wrap_dep_error {
    ($e:ty) => {
        impl<T> WrapError<T> for Result<T, $e> {
            fn to_error(self, token: &Token) -> Result<T, Error> {
                self.or_else(|e| {
                    Err(Error::External {
                        error: e.into(),
                        token: token.clone(),
                    })
                })
            }
        }
    };
}

wrap_dep_error!(polyvalue::Error);
wrap_dep_error!(std::io::Error);
wrap_dep_error!(reqwest::Error);
wrap_dep_error!(rustyscript::Error);
wrap_dep_error!(std::num::ParseIntError);
wrap_dep_error!(std::string::FromUtf8Error);
wrap_dep_error!(serde_json::Error);
