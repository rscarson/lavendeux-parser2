#![allow(missing_docs)]
use crate::{error::WrapSyntaxError, syntax_tree::Node, Error, State};
use pest::Parser;
use pest_derive::Parser;

/// Lavendeux's parser
/// We will not directly expose this to the user, but instead use it to
/// parse the input into a syntax tree
#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct LavendeuxParser;
impl LavendeuxParser {
    pub fn build_ast<'i>(input: &'i str, rule: Rule, state: &mut State) -> Result<Node<'i>, Error> {
        let pairs = LavendeuxParser::parse(rule, input).wrap_syntax_error(input)?;
        if let Some(pair) = pairs.flatten().next() {
            Node::from_pair(pair, state)
        } else {
            oops!(Internal {
                msg: format!("No instance of rule {:?} found in input", rule)
            })
        }
    }
}
/*

/// A macro to assert that the given input parses into the expected AST node
/// # Arguments
/// * `input` - The input to parse
/// * `rule` - The rule to parse the input with
/// * `expected` - The expected AST node
/// * `hnd` - A handler to run on the AST node
///
/// You can also pass in an existing tree instead of an input and rule
#[cfg(test)]
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

/// Asserts that the input given parses into the expected value
#[cfg(test)]
#[macro_export]
macro_rules! assert_tree_value {
    ($input:literal, $expected:expr) => {
        assert_eq!(
            $crate::Lavendeux::eval($input, &mut $crate::State::new(),).unwrap(),
            vec![$expected].into()
        );
    };
}

/// A macro to test the type of a node
#[cfg(test)]
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
#[cfg(test)]
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
pub trait AstNode<'i>: std::fmt::Display + std::fmt::Debug + Send + Sync {
    fn from_pair(input: Pair<'i, Rule>) -> Result<Node<'i>, Error>
    where
        Self: Sized,
    {
        crate::syntax_tree::pratt::Parser::parse(input.into_inner())
    }

    fn from_pratt(
        input: crate::syntax_tree::pratt::PrattPair<'i>,
    ) -> Result<crate::Node<'i>, crate::Error>
    where
        Self: Sized,
    {
        let first_pair = input.first_pair();
        Self::from_pair(first_pair)
    }

    fn get_value(&self, state: &mut State) -> Result<Value, Error>;
    fn token(&self) -> &Token<'i>;
    fn token_offsetline(&mut self, offset: usize);

    fn boxed(self) -> Node<'i>
    where
        Self: Sized + 'i,
    {
        Box::new(self)
    }

    fn clone_self(&self) -> Node<'i>;
    fn into_owned(&self) -> Self
    where
        Self: Sized;
}

/// A trait used to convert a pest pair into an AST node
/// This is used to build the AST
pub trait ToAstNode<'i> {
    fn to_ast_node(self) -> Result<Node<'i>, Error>;
}
impl<'i> ToAstNode<'i> for Pair<'i, Rule> {
    /// Convert a pest pair into an AST node
    /// This maps all the rules to AST Node structures
    fn to_ast_node(self) -> Result<Node<'i>, Error> {
        resolver::handle_pair(self)
    }
}

/// Main function to parse the input into a syntax tree
pub fn parse_input<'i>(input: &'i str, rule: Rule) -> Result<Node<'i>, Error> {
    let pairs = LavendeuxParser::parse(rule, input).wrap_syntax_error(input)?;
    if let Some(pair) = pairs.flatten().next() {
        pair.to_ast_node()
    } else {
        oops!(Internal {
            msg: format!("No instance of rule {:?} found in input", rule)
        })
    }
}
*/
