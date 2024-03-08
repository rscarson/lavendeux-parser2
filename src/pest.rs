//! This module contains the underlying PEST parser for Lavendeux
//! It is not intended to be used directly, but instead is used to parse the input into a syntax tree
//! Use [Lavendeux] to parse input instead
#![allow(missing_docs)]
use crate::{error::WrapSyntaxError, Error, State};
use pest::Parser;
use pest_derive::Parser;

/// Re-export for use with the internal Lavendeux::eval compiler function
pub use crate::syntax_tree::Node;

/// Re-export for use with the internal Lavendeux::eval compiler function
pub use crate::syntax_tree::traits::NodeExt;

/// Lavendeux's parser
/// We will not directly expose this to the user, but instead use it to
/// parse the input into a syntax tree
#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct LavendeuxParser;
impl LavendeuxParser {
    pub fn compile_ast<'i>(
        root_pair: pest::iterators::Pair<'i, Rule>,
        state: &mut State,
    ) -> Result<Node<'i>, Error> {
        Node::from_pair(root_pair, state)
    }

    pub fn parse2<'i>(
        input: &'i str,
        rule: Rule,
    ) -> Result<pest::iterators::Pair<'i, Rule>, Error> {
        let pairs = Self::parse(rule, input).wrap_syntax_error(input)?;
        if let Some(pair) = pairs.flatten().next() {
            Ok(pair)
        } else {
            oops!(Internal {
                msg: format!("No instance of rule {:?} found in input", rule)
            })
        }
    }
}

/// Generates a test case sent to the parser
/// Examples:
///  ```rust
/// use lavendeux_parser::lav;
/// use lavendeux_parser::{error::ErrorDetails, Error};
///
/// lav!(test_isok r#"
/// 1 + 1
/// "#);
///
/// lav!(test_isvar(a = 1, b = 2) r#"
/// a=1, b=2
/// "#);
///
/// lav!(test_iserr(Error) r#"
/// asparagus
/// "#);
///
/// lav!(test_whaterr(Error = |e: &Error| matches!(e.details, ErrorDetails::VariableName {..})) r#"
/// asparagus
/// "#);
/// ```
#[macro_export]
macro_rules! lav {
    ($test_name:ident $body:literal) => {
        #[test]
        fn $test_name() {
            $crate::Lavendeux::new(Default::default()).parse($body).expect("Error parsing expression");
        }
    };
    ($test_name:ident(Error) $body:literal) => {
        #[test]
        fn $test_name() {
            $crate::Lavendeux::new(Default::default()).parse($body).expect_err("Expected expression to fail");
        }
    };

    ($test_name:ident(Error = $pattern:expr) $body:literal) => {
        #[test]
        fn $test_name() {
            let mut lav = $crate::Lavendeux::new(Default::default());
            let e = lav.parse($body).expect_err("Expected expression to fail");
            if !( $pattern(&e) ) {
                panic!("Error did not match pattern: {:#?}", e)
            }
        }
    };

    ($test_name:ident($($n:ident = $v:expr),+$(,)?) $body:literal) => {
        #[test]
        fn $test_name() {
            let mut lav = $crate::Lavendeux::new(Default::default());
            lav.parse($body).expect("Error parsing expression");
            $( assert_eq!(lav.state().get_variable(stringify!($n)).expect(&format!("`{}` was not set", stringify!($n))), &$crate::Value::from($v)); )+
        }
    };
}

#[cfg(test)]
mod test {
    use crate::{error::ErrorDetails, Error};

    lav!(test_isok r#"
        1 + 1
    "#);

    lav!(test_isvar(a = 1i64, b = 2i64) r#"
        a=1; b=2
    "#);

    lav!(test_iserr(Error) r#"
        asparagus
    "#);

    lav!(test_whaterr(Error = |e: &Error| matches!(e.details, ErrorDetails::VariableName {..})) r#"
        asparagus
    "#);
}
