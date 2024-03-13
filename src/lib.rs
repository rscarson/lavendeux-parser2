//!
#![warn(missing_docs)]

// Language documentation
// Regenerate this using:
// `cargo run --bin generate_docs`
pub mod language_docs;

// Utitlity functions for the network related functions in stdlib
#[cfg(feature = "network-functions")]
mod network;

// Docgen utilities
#[macro_use]
mod documentation;

// Errors and error-adjacent gubbins
#[macro_use]
pub mod error;
pub use error::Error;

// The core parser. Builds the AST and evaluates it.
pub mod pest;
pub use pest::Rule; // exported for Token
mod syntax_tree;
pub use syntax_tree::AssignmentTarget;

/// Function related definitions
/// Home of the stdlib, user-functions, and function docs
pub mod functions;

// The main parser state
mod state;
pub use state::State;

// A token parsed from the input
// Comes up in error handling
mod token;
pub use token::Token;

// Main entrypoint for the parser
mod lavendeux;
pub use lavendeux::{Lavendeux, ParserOptions};

// Experimental memory manager
//mod memory_manager;

// Public re-export of the polyvalue crate
pub use polyvalue;
pub use polyvalue::Value;

/// A few critical tests for common grammar issues post-update
#[cfg(test)]
mod test {
    use crate::Lavendeux;

    #[test]
    fn test_empty_input() {
        let mut lav = Lavendeux::new(Default::default());
        lav.parse("").expect("Failed to parse empty input");
    }

    #[test]
    fn test_stackoverflow() {
        let mut lav = Lavendeux::new(Default::default());
        let input = "[[[[[[[[[[99-7[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[99-7[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[99-7[[[[[[[[[[W[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[9[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[99-7[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[99-7[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[K[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[99-7[[[[[[[[[[[[[[[[[[[[[[[[[9-7[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[99-7[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[K[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[99-7[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[z-0&z&oo]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]";
        lav.parse(input).expect_err("this should fail");
    }
}
