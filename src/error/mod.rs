//! Error handling module.
//! Defines the Error/ErrorDetails types, and associated traits and macros.

pub mod macros;

mod error;
pub use error::Error;

pub use error_details::ErrorDetails;
mod error_details;

mod traits;
pub use traits::*;
