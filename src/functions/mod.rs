mod compiler_cache;
mod std_function;
mod user_function;

#[macro_use]
mod macros;

pub use std_function::{
    FunctionArgument, FunctionArgumentType, FunctionDocumentation, ParserFunction,
};
pub use user_function::UserDefinedFunction;

pub mod stdlib;
