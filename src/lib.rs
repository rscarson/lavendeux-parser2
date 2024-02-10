mod error;
mod lavendeux;
mod pest;
mod preprocessor;
mod state;
mod std_functions;
mod syntax_tree;
mod token;
mod user_function;

#[cfg(feature = "network-functions")]
mod network_utils;

#[cfg(feature = "extensions")]
pub use rustyscript;

#[cfg(feature = "extensions")]
mod extensions;

use pest::{AstNode, Rule, ToAstNode};
use syntax_tree::Node;
use token::ToToken;

#[cfg(feature = "extensions")]
pub use extensions::ExtensionDetails;

pub use error::Error;
pub use lavendeux::{Lavendeux, ParserOptions};
pub use polyvalue::Value;
pub use state::State;
pub use token::Token;

#[cfg(test)]
mod test {
    use crate::Lavendeux;

    #[test]
    fn test_load_stdlib() {
        if let Err(e) =
            Lavendeux::new(Default::default()).parse("'examples/stdlib_example.lav'.include()")
        {
            panic!("Failed to load stdlib: {}", e);
        }
    }

    #[test]
    fn test_empty_input() {
        let mut lav = Lavendeux::new(Default::default());
        lav.parse("").expect("Failed to parse empty input");
    }
}
