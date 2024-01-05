use crate::syntax_tree::node::*;
use crate::{Error, Node, State, Token, Value};
use pest::{iterators::Pair, Parser};
use pest_derive::Parser;
use polyvalue::types::Array;
use polyvalue::ValueTrait;

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
        match $crate::parse_input($input, Rule::$rule) {
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

/// A macro to assert that the given input parses into an error
/// # Arguments
/// * `input` - The input to parse
/// * `err` - The expected error
#[macro_export]
macro_rules! assert_tree_error {
    ($input:literal, $err:ident) => {
        if let Err(err) = $crate::parse_input($input, Rule::SCRIPT) {
            assert!(matches!(err, Error::$err { .. }));
        } else {
            panic!("Expected error");
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
    fn get_value(&mut self, state: &mut State) -> Result<Value, Error>;
    fn token(&self) -> &Token;
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
        let mut target = self;

        // Bypass single-child nodes
        // They should not be parsed into a node directly
        // Except for the script and line nodes, which are allowed to be single-child
        while target.clone().into_inner().count() == 1
            && target.as_rule() != Rule::SCRIPT
            && target.as_rule() != Rule::LINE
        {
            target = target.into_inner().next().unwrap();
        }

        node_map()
            .get(&target.as_rule())
            .ok_or(Error::Internal(format!(
                "Grammar issue; rule {:?} is not mapped",
                target.as_rule()
            )))?(target)
    }
}

/// Main function to parse the input into a syntax tree
pub fn parse_input(input: &str, rule: Rule) -> Result<Node, Error> {
    let mut pairs = LavendeuxParser::parse(rule, input)?.flatten();
    if let Some(pair) = pairs.next() {
        pair.to_ast_node()
    } else {
        Err(Error::Internal(format!(
            "Grammar issue; empty input should be valid",
        )))
    }
}

pub struct Lavendeux(State);
impl Lavendeux {
    /// Create a new Lavendeux instance
    /// The instance will have a new state
    pub fn new() -> Self {
        Self::with_state(State::new())
    }

    /// Create a new Lavendeux instance with a given state
    pub fn with_state(state: State) -> Self {
        Self(state)
    }

    /// Evaluate the given input against a state
    pub fn eval(input: &str, state: &mut State) -> Result<Vec<Value>, Error> {
        let value = std::thread::scope(|s| -> Result<Value, Error> {
            let handle = std::thread::Builder::new()
                .stack_size(1024 * 1024 * 8)
                .spawn_scoped(s, || {
                    let mut script = parse_input(input, Rule::SCRIPT)?;
                    script.get_value(state)
                })?;
            handle.join().unwrap()
        })?;

        let lines = value.as_a::<Array>()?.inner().clone();
        Ok(lines)
    }

    /// Parse the given input
    /// # Arguments
    /// * `input` - The input to parse
    ///
    /// # Returns
    /// A vector of values, one for each line in the input
    /// Decorated lines will be a string value
    pub fn parse(&mut self, input: &str) -> Result<Vec<Value>, Error> {
        Self::eval(input, &mut self.0)
    }

    /// Load an extension from a file and register it
    /// # Arguments
    /// * `filename` - The filename of the extension to load
    ///
    /// # Returns
    /// An error if the extension could not be loaded
    #[cfg(feature = "extensions")]
    pub fn load_extension(
        &mut self,
        filename: &str,
    ) -> Result<crate::extensions::ExtensionDetails, Error> {
        crate::extensions::ExtensionController::with(|controller| controller.register(filename))
    }

    /// Unload an extension, stopping the thread and unregistering all functions
    /// # Arguments
    /// * `name` - The filename of the extension to unload
    #[cfg(feature = "extensions")]
    pub fn unload_extension(&mut self, filename: &str) {
        crate::extensions::ExtensionController::with(|controller| controller.unregister(filename));
    }
}
