use crate::{Rule, Token};
use polyvalue::ValueType;
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

/// A wrapper for an error that also contains the token that caused it
#[derive(Debug)]
pub struct ErrorWithToken {
    /// Token that caused the error
    pub token: Token,

    /// Error that occurred
    pub source: Error,
}
impl std::error::Error for ErrorWithToken {}
impl std::fmt::Display for ErrorWithToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\n| {}\n= {}", self.token, self.source)
    }
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

    ///////////////////////////////////////////////////////////////////////////
    // Syntax Errors
    // Deals with issues during Pest tree parsing
    ///////////////////////////////////////////////////////////////////////////

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

    ///////////////////////////////////////////////////////////////////////////
    // Value Errors
    // Mostly deals with variables, and value objects
    ///////////////////////////////////////////////////////////////////////////

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
 
    /// Error dealing with filesystem issues
    #[error("{0}")]
    Io(#[from] std::io::Error),

    /// Error dealing with network issues from the reqwest crate
    #[error("{0}")]
    Network(#[from] reqwest::Error),

    /// Error dealing with pest parsing problems
    #[error("{0}")]
    Pest(#[from] pest::error::Error<Rule>),

    /// Error dealing with JS execution issues
    #[error("{0}")]
    Javascript(#[from] rustyscript::Error),
    
    /// Error dealing with arithmetic issues
    #[error("{0}")]
    ValueError(#[from] polyvalue::Error),

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
