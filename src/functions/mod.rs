mod std_function;
mod user_function;

#[macro_use]
mod macros;

mod documentation;
pub use documentation::*;

pub use std_function::{FunctionArgument, FunctionArgumentType, ParserFunction};
pub use user_function::UserDefinedFunction;

/// The standard library of functions
/// Loaded by the state by default
pub mod stdlib;
