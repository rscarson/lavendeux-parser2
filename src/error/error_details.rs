use crate::Token;
use polyvalue::{Value, ValueType};
use thiserror::Error;

const BUG_REPORT_URL : &str = "https://github.com/rscarson/lavendeux-parser/issues/new?assignees=&labels=&template=bug_report.md&title=";

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
        msg: String,
    },

    /// An error caused by leaving a block empty
    #[error("Block cannot be empty")]
    EmptyBlock,

    /// An error caused by a problem with the syntax of the script
    #[error("Syntax error; unexpected token")]
    Syntax,

    /// Error causing the parser thread to panic
    #[error("Fatal error: {msg}")]
    Fatal {
        msg: String
    },

    /// A timeout error caused by a script taking too long to execute
    #[error("Script execution timed out")]
    Timeout,

    /// An error caused by a custom error message
    #[error("{message}")]
    Custom{
        message: String,
    },

    /// An error caused by a problem with the syntax of the script
    #[error("Invalid preprocessor directive: {directive}")]
    InvalidDirective {
        directive: String,
    },

    /// An error used to return a value from a function early
    #[error("Returned from the root scope")]
    Return {
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

    #[error("Expected a pattern to match against (an array, value, or regex literal)")]
    IncompleteMatchingExpression,

    #[error("Expected 2 bounds for range expression, for example: 1..2 or 'a'..'z'")]
    IncompleteRangeExpression,

    #[error("Expected a key-value pair, for example: {{0: 'test'}}")]
    IncompleteObject,

    #[error("Match expression is not exhaustive. Add a default case '_' to match all values")]
    NonExhaustiveSwitch,

    #[error("All cases after the default case '_' are unreachable")]
    UnreachableSwitchCase,

    #[error("{case} is not valid for this switch statement. Expected a {expected_type}")]
    SwitchCaseTypeMismatch {
        case: Value,
        expected_type: ValueType,
    },

    ///////////////////////////////////////////////////////////////////////////
    // Value Errors
    // Mostly deals with variables, and value objects
    ///////////////////////////////////////////////////////////////////////////
    
    #[error("Cannot assign to a constant value")]
    ConstantValue,

    #[error("Implicit multiplication is not allowed between {left} and {right}")]
    IllegalImplicitMultiplication {
        left: String, right: String, token: Token
    },

    #[error("Invalid combination of types for range. Use a pair of either integers, or characters")]
    RangeTypeMismatch,

    #[error("{start}..{end} is not a valid range: use integers or single-byte strings")]
    InvalidRange {
        start: String,
        end: String,
    },

    #[error("{start}..{end} is not a valid range: start > end")]
    RangeStartGT {
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

    #[error("{input} was out of range")]
    Range {
        input: String,
    },

    #[error("Undefined variable {name}. You can assign a value with {name} = ...")]
    VariableName {
        name: String,
    },

    #[error("Array empty")]
    ArrayEmpty,
    
    #[error("An index is required here")]
    EmptyIndex,

    ///////////////////////////////////////////////////////////////////////////
    // Function Errors
    // Deals with issues during builtin, user, or extension function calls
    ///////////////////////////////////////////////////////////////////////////

    #[error("Decorator @{name} must accept a single argument")]
    DecoratorSignatureArgs {
        name: String,
    },

    #[error("@{name} does not need to specify a return type; decorators always return a string")]
    DecoratorSignatureReturn {
        name: String,
    },

    #[error("Error in `{name}()`")]
    FunctionCall {
        name: String
    },

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
        name: String,
    },
    
    /// An error caused by attempting to use an API without registering it
    #[error("API {name} was not found. Add it with api_register(\"{name}\", base_url, [optional api key])")]
    UnknownApi {
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
