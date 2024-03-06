use polyvalue::{Value, ValueType};
use thiserror::Error;

use super::RuleCategory;

const BUG_REPORT_URL : &str = "https://github.com/rscarson/lavendeux-parser/issues/new?assignees=&labels=&template=bug_report.md&title=";

/// Inner error type for Lavendeux
/// Gives more detailed information about the error
/// And gets wrapped in the main Error type, along with metadata
#[derive(Error, Debug)]
#[rustfmt::skip]
pub enum ErrorDetails {
    //
    // Core
    //

    /// An error caused by a problem with the parser itself
    #[error(
        "Internal parser issue: {msg}\nPlease report this problem at {}",
        BUG_REPORT_URL
    )]
    Internal {
        /// Message describing the error
        msg: String,
    },

    /// An error caused by leaving a block empty
    #[error("Block cannot be empty")]
    EmptyBlock,

    /// An error caused by a problem with the syntax of the script
    #[error("Syntax error{}", if expected.len() == 1 {
        format!("; Expected {}", expected[0])
    } else if !expected.is_empty() {
        format!("; Expected one of: {}", RuleCategory::fmt(expected))
    } else {
        "".to_string()
    }
    )]
    Syntax {
        /// List of expected rule categories
        expected: Vec<RuleCategory>
    },

    /// Error causing the parser thread to panic
    #[error("Fatal error: {msg}")]
    Fatal {
        /// Message describing the error
        msg: String
    },

    /// A timeout error caused by a script taking too long to execute
    #[error("Script execution timed out")]
    Timeout,

    /// An error caused by a custom error message
    #[error("{msg}")]
    Custom {
        /// Message describing the error
        msg: String,
    },

    /// An error used to return a value from a function early
    #[error("Returned from the root scope")]
    Return {
        /// Value being returned
        value: Value,
    },

    /// An error used to skip a value from a loop
    #[error("Skipped from outside a loop")]
    Skip,

    /// An error used to skip a value from a loop
    #[error("Break called from outside a loop")]
    Break,

    ///////////////////////////////////////////////////////////////////////////
    // Syntax Errors
    // Deals with issues during Pest tree parsing
    ///////////////////////////////////////////////////////////////////////////

    /// An error caused by attempting to modify a read-only stdlib function
    #[error("Could not alter system function {name}")]
    ReadOnlyFunction {
        /// Name of the function being referred to
        name: String,
    },

    /// An error caused by a problem with the syntax of the script
    #[error("If statements are required return a value - use 'else' to select a default value")]
    NoElseBlock,

    /// An error caused by a problem with the syntax of the script
    #[error("Operator assignment is not allowed in destructuring assignment")]
    DestructuringAssignmentWithOperator,

    /// An error caused by a problem with the syntax of the script
    #[error("Did not specify a value for return")]
    UnterminatedReturn,

    /// An error caused by using a decorator in the wrong place
    #[error("@decorators must be at the end of a statement")]
    UnexpectedDecorator,

    /// An error caused by using a postfix operator without an operand
    #[error("Unterminated block comment: Expected '*/'")]
    UnterminatedComment,

    /// An error caused by a missing bracket
    #[error("Unclosed bracket: Expected ']'")]
    UnterminatedArray,

    /// An error caused by a missing brace
    #[error("Unclosed brace: Expected '}}'")]
    UnterminatedObject,

    /// An error caused by a missing brace
    #[error("Unclosed parentheses: Expected '('")]
    UnterminatedParen,

    /// An error caused by ending a script on a backslash
    #[error("Missing linebreak after '\\'")]
    UnterminatedLinebreak,

    /// An error caused by a missing quote
    #[error("Expected ' or \"")]
    UnterminatedLiteral,

    /// Cause by a missing default case in a switch statement
    #[error("Match expression is not exhaustive. Add a default case '_' to match all values")]
    NonExhaustiveSwitch,

    /// Caused by a default case eclipsing other cases in a switch statement
    #[error("All cases after the default case '_' are unreachable")]
    UnreachableSwitchCase,

    /// Caused by a type mismatch in a switch statement
    #[error("{case} is not valid for this switch statement. Expected a {expected_type}")]
    SwitchCaseTypeMismatch {
        /// Case that caused the issue
        case: Value,

        /// Type that was expected
        expected_type: ValueType,
    },

    ///////////////////////////////////////////////////////////////////////////
    // Value Errors
    // Mostly deals with variables, and value objects
    ///////////////////////////////////////////////////////////////////////////
    
    /// Caused by assignments to constants
    #[error("Cannot assign to a constant value")]
    ConstantValue,

    /// An error caused by a mismatch in types for a range
    #[error("Invalid combination of types for range. Use a pair of either integers, or characters")]
    RangeTypeMismatch,

    /// An error caused by invalid range values
    #[error("{start}..{end} is not a valid range: use integers or single-byte strings")]
    InvalidRange {
        /// Start value
        start: String,

        /// End value
        end: String,
    },

    /// An error caused by invalid range values
    #[error("{start}..{end} is not a valid range: start > end")]
    RangeStartGT {
        /// Start value
        start: String,

        /// End value
        end: String,
    },

    /// An error caused by a value being out of range
    #[error("Arithmetic overflow")]
    Overflow,

    /// Caused by a mismatch in the number of values in a destructuring assignment
    #[error("Expected {expected_length} values, found {actual_length}")]
    DestructuringAssignment {
        /// Number of values expected
        expected_length: usize,

        /// Number of values found
        actual_length: usize,
    },

    /// An error caused by a value not being able to be parsed
    #[error("Input could not be parsed as {expected_format}")]
    ValueFormat {
        /// Format that was expected
        expected_format: String,
    },

    /// An error caused by a value being out of range
    #[error("{input} was out of range")]
    Range {
        /// Input that was out of range
        input: String,
    },

    /// An error caused by a missing variable
    #[error("Undefined variable {name}. You can assign a value with {name} = ...")]
    VariableName {
        /// Name of the variable being referred to
        name: String,
    },

    /// An error caused by an attempt to access an element of an empty array
    #[error("Array empty")]
    ArrayEmpty,

    ///////////////////////////////////////////////////////////////////////////
    // Function Errors
    // Deals with issues during builtin, user, or extension function calls
    ///////////////////////////////////////////////////////////////////////////

    /// An error caused by a decorator specifying the wrong number of arguments
    #[error("Decorator @{name} must accept a single argument")]
    DecoratorSignatureArgs {
        /// Name of the decorator being referred to
        name: String,
    },

    /// An error caused by a decorator specifying a return type
    #[error("@{name} does not need to specify a return type; decorators always return a string")]
    DecoratorSignatureReturn {
        /// Name of the decorator being referred to
        name: String,
    },

    /// An error caused by a function call
    #[error("Error in `{name}()`")]
    FunctionCall {
        /// Name of the source function
        name: String
    },

    /// An error caused by a function calling itself too many times
    #[error("Recursive function went too deep")]
    StackOverflow,
    
    /// An error caused by calling a function with the wrong type of argument
    #[error("Expected {expected_type} value for argument {arg} of `{signature}`")]
    FunctionArgumentType {
        /// Argument number causing the issue (1-based)
        arg: usize,

        /// Type that was requested
        expected_type: ValueType,
        
        /// Signature of the function called
        signature: String,

    },

    /// An error caused by calling a function that does not exist
    #[error("Undefined function {name}. You can define a function with {name}(a, b, c) = ...")]
    FunctionName {
        /// Name of the function being referred to
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
    #[error("No decorator named {name}")]
    DecoratorName {
        /// Name of the decorator being referred to
        name: String,
    },
    
    /// An error caused by attempting to use an API without registering it
    #[error("API {name} was not found. Add it with api_register(\"{name}\", base_url, [optional api key])")]
    UnknownApi {
        /// Name of the API being referred to
        name: String,
    },

    //
    // 3rd Party
    //
    
    /// Error dealing with polyvalue issues
    #[error("{0}")]
    Value(#[from] polyvalue::Error),

    /// Error dealing with filesystem issues
    #[error("{0}")]
    Io(#[from] std::io::Error),

    /// Error dealing with network issues from the reqwest crate
    #[error("{0}")]
    Network(#[from] reqwest::Error),

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
