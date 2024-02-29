use lazy_static::lazy_static;
use pest::iterators::Pair;

use super::{pratt::PrattPair, Node};
use crate::{
    error::{ErrorDetails, WrapOption},
    pest::Rule,
    Error,
};
use std::collections::HashMap;

pub trait NodeResolver: Sync {
    fn handle<'i>(&self, pair: &Pair<'i, Rule>) -> Result<Node<'i>, Error<'i>>;
    fn handle_pratt<'i>(&self, pair: &PrattPair<'i>) -> Result<Node<'i>, Error<'i>>;
    fn rules(&self) -> &'static [Rule];
}
inventory::collect!(&'static dyn NodeResolver);
pub fn all() -> HashMap<Rule, &'static dyn NodeResolver> {
    let mut map = HashMap::new();
    for node in inventory::iter::<&'static dyn NodeResolver> {
        for rule in node.rules() {
            map.insert(*rule, *node);
        }
    }
    map
}
lazy_static! {
    pub static ref NODES: HashMap<Rule, &'static dyn NodeResolver> = all();
}

pub fn handle_pair<'i>(pair: &'i Pair<'i, Rule>) -> Result<Node<'i>, Error<'i>> {
    NODES
        .get(&pair.as_rule())
        .or_error(ErrorDetails::Internal {
            msg: format!("No handler for rule {:?}", pair.as_rule()),
        })?
        .handle(pair)
}

pub fn handle_pratt<'i>(pair: &'i PrattPair<'i>) -> Result<Node<'i>, Error<'i>> {
    NODES
        .get(&pair.as_rule())
        .or_error(ErrorDetails::Internal {
            msg: format!("No handler for rule {:?}", pair.as_rule()),
        })?
        .handle_pratt(pair)
}
