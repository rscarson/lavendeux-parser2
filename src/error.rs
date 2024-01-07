use crate::Rule;
use polyvalue::{operations::ArithmeticOperation, ValueType};
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
        "internal parser issue: {0}\nPlease report this problem at {}",
        BUG_REPORT_URL
    )]
    Internal(String),

    /// Error causing the parser thread to panic
    #[error("fatal error: {0}")]
    Fatal(String),

    /// A timeout error caused by a script taking too long to execute
    #[error("script execution timed out")]
    Timeout,

    ///////////////////////////////////////////////////////////////////////////
    // Value Errors
    // Mostly deals with variables, and value objects
    ///////////////////////////////////////////////////////////////////////////
    
    /// An error caused by an invalid range expression
    #[error("invalid range expression {start}..{end}")]
    InvalidRange {
        /// Start of the range
        start: String,
        
        /// End of the range
        end: String,
    },

    /// An error caused by attempting to overwrite a constant
    #[error("{src_type} cannot be converted to {dst_type}")]
    ValueConversion {
        src_type: ValueType,
        dst_type: ValueType,
    },

    /// An error caused by attempting to overwrite a constant
    #[error("could not overwrite constant value {name}")]
    ConstantValue {
        /// Name of the constant
        name: String
    },

    /// An error caused by a calculation that resulted in an overflow
    #[error("arithmetic overflow")]
    Overflow,

    /// An error caused by a calculation that resulted in an underflow
    #[error("arithmetic underflow")]
    Underflow,

    /// An error caused by destructuring an array or object with the wrong number of values
    #[error("expected {expected_length} values, found {actual_length}")]
    DestructuringAssignment {
        expected_length: usize,
        actual_length: usize,
    },

    /// An error caused by attempting to parse an value
    #[error("{input} could not be parsed as {expected_type}")]
    ValueParsing {
        /// Value causing the error
        input: String,
        
        /// Type that was requested
        expected_type: ValueType,
    },

    /// An error caused by attempting to parse an invalid string into a given format
    #[error("string could not be parsed as {expected_format}")]
    StringFormat {
        /// Expected format of the string
        expected_format: String,
    },

    /// An error caused by attempting use an out of range value
    #[error("value {0} was out of range")]
    Range(String),

    /// An error caused by attempting to use a value of the wrong type in a calculation
    #[error("expected {expected_type}, found {actual_type}")]
    ValueType {
        /// Value causing the error
        actual_type: ValueType,
        
        /// Type that was requested
        expected_type: ValueType,
    },

    /// An error caused by attempting to use an unassigned variable
    #[error("undefined variable {name}")]
    VariableName {
        /// Name of the variable
        name: String,
    },

    /// An error caused by attempting to use an operator on
    /// the wrong type
    #[error("could not perform arithmetic {operation} on {actual_type}")]
    UnsupportedOperation {
        operation: ArithmeticOperation,
        actual_type: ValueType,
    },

    ///////////////////////////////////////////////////////////////////////////
    // Syntax Errors
    // Deals with issues during Pest tree parsing
    ///////////////////////////////////////////////////////////////////////////

    /// An error caused by using a decorator in the wrong place
    #[error("@decorators must be at the end of a statement")]
    UnexpectedDecorator,

    /// An error caused by using a postfix operator without an operand
    #[error("expected '*/'")]
    UnterminatedComment,

    /// An error caused by a missing bracket
    #[error("expected ']'")]
    UnterminatedArray,

    /// An error caused by a missing brace
    #[error("expected '}}'")]
    UnterminatedObject,

    /// An error caused by ending a script on a backslash
    #[error("missing linebreak after '\\'")]
    UnterminatedLinebreak,

    /// An error caused by a missing quote
    #[error("expected ' or \"")]
    UnterminatedLiteral,

    /// An error caused by a missing parentheses
    #[error("expected ')'")]
    UnterminatedParen,

    ///////////////////////////////////////////////////////////////////////////
    // Function Errors
    // Deals with issues during builtin, user, or extension function calls
    ///////////////////////////////////////////////////////////////////////////

    /// An error caused by a recursive function going too deep
    #[error("user-defined function exceeded maximum recursion depth")]
    StackOverflow,

    /// An error caused by attempting to use a function with ambiguous arguments
    #[error("function parameters for {signature} are ambiguous")]
    AmbiguousFunctionDefinition {
        /// Signature of the function called
        signature: String,
    },

    /// An error caused by calling a function with an argument of the wrong type
    #[error("argument {arg} of {signature}, expected {expected_type}")]
    FunctionArgumentType {
        /// Argument number causing the issue (1-based)
        arg: usize,

        /// Type that was requested
        expected_type: ValueType,
        
        /// Signature of the function called
        signature: String,
    },

    /// An error caused by calling a function that does not exist
    #[error("no such function {name}")]
    FunctionName {
        /// Name of the function
        name: String,
    },

    /// An error caused by calling a function using the wrong number of arguments
    #[error(
        "{signature} expected {} arguments",
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

    /// An error caused by a function argument overflowing a pre-determined limit
    #[error("argument {arg} of {signature}")]
    FunctionArgumentOverflow {
        /// Argument number causing the issue (1-based)
        arg: usize,
        
        /// Signature of the function called
        signature: String,
    },

    /// An error caused by calling a decorator with an argument of the wrong type
    #[error("@{name} expected type {expected_type}")]
    DecoratorArgumentType {
        /// Type that was requested
        expected_type: ValueType,

        /// Name of the decorator
        name: String,
    },

    /// An error caused by calling a decorator that does not exist
    #[error("no such decorator {name}")]
    DecoratorName {
        /// Name of the decorator
        name: String,
    },
    
    /// An error caused by attempting to use an API without registering it
    #[error("API {name} was not found. Add it with api_register(\"{name}\", base_url, [optional api key])")]
    UnknownApi {
        /// Name of the API
        name: String,
    },

    ///////////////////////////////////////////////////////////////////////////
    // Array Errors
    // Deals with issues indexing of arrays and objects
    ///////////////////////////////////////////////////////////////////////////

    /// An error caused by attempting to use an invalid object or array key
    #[error("undefined index {key}")]
    Index {
        /// Index that caused the error
        key: String,
    },

    /// An error caused by attempting to index on an empty array
    #[error("could not index empty array")]
    ArrayEmpty,

    /// An error caused by attempting to operate on a pair of arrays of incompatible lengths
    #[error("array lengths were incompatible")]
    ArrayLengths,

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
