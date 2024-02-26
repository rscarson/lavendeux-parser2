use crate::error::WrapSyntaxError;
use crate::syntax_tree::resolver;
use crate::{oops, Error, Node, State, Token, Value};
use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

/// Lavendeux's parser
/// We will not directly expose this to the user, but instead use it to
/// parse the input into a syntax tree
#[derive(Parser)]
#[grammar = "grammar.pest"]
struct LavendeuxParser;

/// A macro to assert that the given input parses into the expected AST node
/// # Arguments
/// * `input` - The input to parse
/// * `rule` - The rule to parse the input with
/// * `expected` - The expected AST node
/// * `hnd` - A handler to run on the AST node
///
/// You can also pass in an existing tree instead of an input and rule
#[macro_export]
macro_rules! assert_tree {
    ($input:literal, $rule:ident, $expected:ty, $hnd:expr) => {
        match $crate::pest::parse_input($input, Rule::$rule) {
            Ok(mut tree) => {
                $crate::assert_tree!(&mut tree, $expected, $hnd);
            }
            Err(err) => panic!("Parsing error: {}", err),
        }
    };

    ($tree:expr, $expected:ty, $hnd:expr) => {
        $tree
            .as_any_mut()
            .downcast_mut::<$expected>()
            .ok_or(Error::Internal(format!(
                "Could not downcast to requested type"
            )))
            .and_then(|tree| {
                ($hnd)(tree);
                Ok(())
            })
            .expect(&format!(
                "Expected a {} but got a {:?}",
                stringify!($expected),
                $tree.token().rule,
            ))
    };
}

#[macro_export]
macro_rules! assert_tree_value {
    ($input:literal, $expected:expr) => {
        assert_eq!(
            $crate::Lavendeux::eval($input, &mut $crate::State::new(),).unwrap(),
            vec![$expected].into()
        );
    };
}

#[macro_export]
macro_rules! node_is_type {
    ($node:expr, $type:path) => {
        $node.as_any().downcast_ref::<$type>().is_some()
    };
}

/// A macro to assert that the given input parses into an error
/// # Arguments
/// * `input` - The input to parse
/// * `err` - The expected error
#[macro_export]
macro_rules! assert_tree_error {
    ($input:literal, $err:ident) => {
        match $crate::pest::parse_input($input, Rule::SCRIPT) {
            Ok(tree) => {
                // check if get_value errors instead
                match tree.get_value(&mut $crate::State::new()) {
                    Ok(_) => panic!("Expected error"),
                    Err(err) => {
                        if !matches!(err, Error::$err { .. }) {
                            panic!("Expected error {:?} but got {:?}", stringify!($err), err);
                        }
                    }
                }
            }
            Err(err) => {
                if !matches!(err, Error::$err { .. }) {
                    panic!("Expected error {:?} but got {:?}", stringify!($err), err);
                }
            }
        }
    };
}

/// A trait for all nodes in the syntax tree
/// The trait is used to build the AST, and to evaluate it by getting the
/// value of each node
pub trait AstNode: std::fmt::Display + std::fmt::Debug {
    fn from_pair(input: Pair<Rule>) -> Result<Node, Error>
    where
        Self: Sized;
    fn get_value(&self, state: &mut State) -> Result<Value, Error>;
    fn token(&self) -> &Token;
    fn token_offsetline(&mut self, offset: usize);
    fn boxed(self) -> Node
    where
        Self: Sized + 'static;

    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

/// A trait used to convert a pest pair into an AST node
/// This is used to build the AST
pub trait ToAstNode {
    fn to_ast_node(self) -> Result<Box<dyn AstNode>, Error>;
}
impl ToAstNode for Pair<'_, Rule> {
    /// Convert a pest pair into an AST node
    /// This maps all the rules to AST Node structures
    fn to_ast_node(self) -> Result<Box<dyn AstNode>, Error> {
        resolver::handle_pair(self)
    }
}

/// Main function to parse the input into a syntax tree
pub fn parse_input(input: &str, rule: Rule) -> Result<Node, Error> {
    let pairs = LavendeuxParser::parse(rule, input).wrap_syntax_error(input)?;
    if let Some(pair) = pairs.flatten().next() {
        pair.to_ast_node()
    } else {
        oops!(Internal {
            msg: format!("No instance of rule {:?} found in input", rule)
        })
    }
}
