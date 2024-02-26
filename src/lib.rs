#![warn(missing_docs)]

pub mod language_docs;

pub mod error;
pub use error::Error;

mod functions;
mod lavendeux;
mod pest;
mod state;
mod syntax_tree;
mod token;

#[cfg(feature = "network-functions")]
mod network;

#[cfg(feature = "extensions")]
pub use rustyscript;

#[cfg(feature = "extensions")]
mod extensions;

use pest::{AstNode, Rule, ToAstNode};
use syntax_tree::Node;
use token::ToToken;

#[cfg(feature = "extensions")]
pub use extensions::ExtensionDetails;

pub use lavendeux::{Lavendeux, ParserOptions};
pub use polyvalue::Value;
pub use state::State;
pub use token::Token;

mod documentation;

#[cfg(test)]
mod test {
    use crate::Lavendeux;

    #[test]
    fn test_load_stdlib() {
        if let Err(e) =
            Lavendeux::new(Default::default()).parse("'examples/stdlib_example.lav'.include()")
        {
            panic!("Failed to load stdlib:\n{}", e);
        }
    }

    #[test]
    fn test_empty_input() {
        let mut lav = Lavendeux::new(Default::default());
        lav.parse("").expect("Failed to parse empty input");
    }

    #[test]
    fn test_stackoverflow() {
        let mut lav = Lavendeux::new(Default::default());
        let input = "[[[[[[[[[[99-7[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[99-7[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[99-7[[[[[[[[[[W[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[9[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[99-7[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[99-7[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[K[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[99-7[[[[[[[[[[[[[[[[[[[[[[[[[9-7[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[99-7[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[K[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[99-7[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[z-0&z&oo]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]";
        let _ = lav.parse(&input);
    }
}
