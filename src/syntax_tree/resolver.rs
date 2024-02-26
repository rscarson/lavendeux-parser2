use lazy_static::lazy_static;
use pest::iterators::Pair;

use super::{pratt::PrattPair, Node};
use crate::{
    error::{ErrorDetails, WrapOption},
    pest::Rule,
    Error,
};
use std::collections::HashMap;

pub type NodeHandlerFn = fn(pest::iterators::Pair<crate::Rule>) -> Result<Node, Error>;
pub type PrattHandlerFn = fn(PrattPair) -> Result<Node, Error>;

#[derive(Debug, Clone)]
pub enum CollectibleNode {
    Node(&'static [Rule], NodeHandlerFn),
    Pratt(&'static [Rule], PrattHandlerFn),
}

impl CollectibleNode {
    pub fn rules(&self) -> &'static [Rule] {
        match self {
            CollectibleNode::Node(rules, _) => rules,
            CollectibleNode::Pratt(rules, _) => rules,
        }
    }
}

inventory::collect!(CollectibleNode);
pub fn all() -> HashMap<Rule, CollectibleNode> {
    let nodes = inventory::iter::<CollectibleNode>
        .into_iter()
        .collect::<Vec<_>>();
    let mut map = HashMap::new();
    for node in nodes {
        for rule in node.rules() {
            map.insert(*rule, node.clone());
        }
    }
    map
}

lazy_static! {
    pub static ref NODES: HashMap<Rule, CollectibleNode> = all();
}

pub fn handle_pair(pair: Pair<Rule>) -> Result<Node, Error> {
    let node = NODES
        .get(&pair.as_rule())
        .or_error(ErrorDetails::Internal {
            msg: format!("No handler for rule {:?}", pair.as_rule()),
        })?;
    match node {
        CollectibleNode::Node(_, handler) => handler(pair),
        CollectibleNode::Pratt(_, handler) => handler(PrattPair::Primary(pair)),
    }
}

pub fn handle_pratt(pair: PrattPair) -> Result<Node, Error> {
    let node = NODES
        .get(&pair.as_rule())
        .or_error(ErrorDetails::Internal {
            msg: format!("No handler for rule {:?}", pair.as_rule()),
        })?;
    match node {
        CollectibleNode::Node(_, handler) => handler(pair.first_pair()),
        CollectibleNode::Pratt(_, handler) => handler(pair),
    }
}
