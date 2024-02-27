//! Error handling module.
//! Defines the Error/ErrorDetails types, and associated traits and macros.

#[macro_use]
mod macros;

mod error;
pub use error::Error;

mod error_details;
pub use error_details::ErrorDetails;

mod traits;
pub use traits::*;
