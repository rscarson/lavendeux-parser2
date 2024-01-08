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

    /// An error from this list, but tagged with a token
    #[error("\n| {token}\n= {source}")]
    Parsing {
        /// Token that caused the error
        token: Token,

        /// Error that occurred
        #[source]
        source: Box<Error>,
    },

    ///////////////////////////////////////////////////////////////////////////
    // Syntax Errors
    // Deals with issues during Pest tree parsing
    ///////////////////////////////////////////////////////////////////////////

    /// An error caused by using a decorator in the wrong place
    #[error("@decorators must be at the end of a statement")]
    UnexpectedDecorator,

    /// An error caused by using a postfix operator without an operand
    #[error("Expected '*/'")]
    UnterminatedComment,

    /// An error caused by a missing bracket
    #[error("Expected ']'")]
    UnterminatedArray,

    /// An error caused by a missing brace
    #[error("Expected '}}'")]
    UnterminatedObject,

    /// An error caused by ending a script on a backslash
    #[error("Missing linebreak after '\\'")]
    UnterminatedLinebreak,

    /// An error caused by a missing quote
    #[error("Expected ' or \"")]
    UnterminatedLiteral,

    /// An error caused by a missing parentheses
    #[error("Expected ')'")]
    UnterminatedParen,

    ///////////////////////////////////////////////////////////////////////////
    // Value Errors
    // Mostly deals with variables, and value objects
    ///////////////////////////////////////////////////////////////////////////

    #[error("Invalid combination of types for range. Use a pair of either integers, or characters")]
    RangeTypeMismatch,

    #[error("Invalid values for range: {start} > {end}")]
    InvalidRange {
        start: String,
        end: String,
    },

    #[error("Arithmetic overflow")]
    Overflow,

    #[error("Expected {expected_length} values, found {actual_length}")]
    DestructuringAssignment {
        expected_length: usize,
        actual_length: usize,
    },

    #[error("Input could not be parsed as {expected_format}")]
    ValueFormat {
        expected_format: String,
    },

    #[error("{0} was out of range")]
    Range(String),

    #[error("Undefined variable {name}. You can assign a value with {name} = ...")]
    VariableName {
        name: String,
    },

    #[error("Array empty")]
    ArrayEmpty,

    ///////////////////////////////////////////////////////////////////////////
    // Function Errors
    // Deals with issues during builtin, user, or extension function calls
    ///////////////////////////////////////////////////////////////////////////

    #[error("Recursive function went too deep")]
    StackOverflow,
    
    #[error("Expected {expected_type} value for argument {arg} of `{signature}`")]
    FunctionArgumentType {
        /// Argument number causing the issue (1-based)
        arg: usize,

        /// Type that was requested
        expected_type: ValueType,
        
        /// Signature of the function called
        signature: String,
    },

    #[error("Undefined function {name}. You can define a function with {name}(a, b, c) = ...")]
    FunctionName {
        name: String,
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
    },

    /// An error caused by calling a decorator that does not exist
    #[error("No decorator named @{name}")]
    DecoratorName {
        /// Name of the decorator
        name: String,
    },
    
    /// An error caused by attempting to use an API without registering it
    #[error("API {name} was not found. Add it with api_register(\"{name}\", base_url, [optional api key])")]
    UnknownApi {
        name: String,
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
