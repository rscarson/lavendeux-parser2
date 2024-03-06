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
