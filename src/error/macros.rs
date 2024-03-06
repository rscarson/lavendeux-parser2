/// Matches an error to the type of ErrorDetails it contains
/// # Arguments
/// * `err` - The error to match
/// * `pat` - The pattern to match against
///
/// # Example
/// ```rust
/// # use lavendeux_parser::error::Error;
/// # use lavendeux_parser::error::ErrorDetails;
/// # use lavendeux_parser::error_matches;
///
/// let err = Error {
///     details: ErrorDetails::ArrayEmpty,
///     context: None,
///     source: None,
/// };
/// assert!(error_matches!(err, ArrayEmpty));
/// ```
#[macro_export]
macro_rules! error_matches {
    ($err:expr, $pat:ident) => {
        matches!(($err).details, $crate::error::ErrorDetails::$pat { .. })
    };
}

/// Returns an Err(Error), optionally with a context and/or source
/// Example:
/// ```rust
/// # use lavendeux_parser::error::Error;
/// # use lavendeux_parser::oops;
/// # use lavendeux_parser::Token;
/// # fn example() -> Result<(), Error> {
/// # let token = Token::dummy();
/// # let parent_error = Error::from(lavendeux_parser::error::ErrorDetails::ArrayEmpty);
/// return oops!(FunctionName { name: "foo".to_string() }, token);
/// return oops!(FunctionName { name: "foo".to_string() }, token = token, src = parent_error);
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! oops {
    ($variant:ident $({ $($n:ident$(:$v:expr)?),+ })?, token = $context:expr, src = $src:expr) => {
        Err($crate::error::Error {
            details: $crate::error::ErrorDetails::$variant $({ $($n $(: $v)?),+ })?,
            context: Some($context.into_owned()),
            source: Some(Box::new($src)),
        })
    };
    ($variant:ident $({ $($n:ident$(:$v:expr)?),+ })?, $context:expr) => {
        Err($crate::error::Error {
            details: $crate::error::ErrorDetails::$variant $({ $($n $(: $v)?),+ })?,
            context: Some($context.into_owned()),
            source: None,
        })
    };
    ($variant:ident $({ $($n:ident$(:$v:expr)?),+ })?, src = $src:expr) => {
        Err($crate::error::Error {
            details: $crate::error::ErrorDetails::$variant $({ $($n $(: $v)?),+ })?,
            context: None,
            source: Some(Box::new($src)),
        })
    };
    ($variant:ident $({ $($n:ident$(:$v:expr)?),+ })?) => {
        Err($crate::error::Error {
            details: $crate::error::ErrorDetails::$variant $({ $($n $(: $v)?),+ })?,
            context: None,
            source: None,
        })
    };
}
