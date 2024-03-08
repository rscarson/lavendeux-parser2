use crate::{Error, Rule, State, Token};
use pest::iterators::Pair;
use std::collections::VecDeque;

use super::{pratt, traits::IntoNode, Node};

#[derive(Clone, Debug)]
pub struct PestIterator<'i> {
    token: Token<'i>,
    inner: InnerPestIterator<'i>,
}

#[allow(dead_code)]
impl<'i> PestIterator<'i> {
    pub fn from_pair(pair: Pair<'i, Rule>) -> Self {
        match pair.as_rule() {
            Rule::EXPR => {
                let mut inner = pair.into_inner();
                if inner.len() == 1 {
                    Self::from_pair(inner.next().unwrap())
                } else {
                    pratt::Parser::parse(inner)
                }
            }
            _ => {
                let token = Token::from(&pair);
                Self {
                    inner: InnerPestIterator::from_pair(pair),
                    token,
                }
            }
        }
    }

    pub fn from_infix(left: PestIterator<'i>, op: Pair<'i, Rule>, right: PestIterator<'i>) -> Self {
        let mut token = Token::from(&op);
        token.input = format!("{} {} {}", left.as_str(), token.input, right.as_str()).into();
        let inner = InnerPestIterator::from_vec(vec![left, Self::from_pair(op), right]);
        Self { token, inner }
    }

    pub fn from_prefix(op: Pair<'i, Rule>, right: PestIterator<'i>) -> Self {
        let mut token = Token::from(&op);
        token.input = format!("{} {}", token.input, right.as_str()).into();
        let inner = InnerPestIterator::from_vec(vec![Self::from_pair(op), right]);
        Self { token, inner }
    }

    pub fn from_postfix(left: PestIterator<'i>, op: Pair<'i, Rule>) -> Self {
        let mut token = Token::from(&op);
        token.input = format!("{} {}", left.as_str(), token.input).into();
        let inner = InnerPestIterator::from_vec(vec![left, Self::from_pair(op)]);
        Self { token, inner }
    }

    pub fn token(&self) -> &Token<'i> {
        &self.token
    }

    pub fn next_child(&mut self) -> Option<Self> {
        self.inner.next_child()
    }

    pub fn last_child(&mut self) -> Option<Self> {
        self.inner.last_child()
    }

    pub fn peek(&self) -> Option<&Self> {
        self.inner.peek()
    }

    pub fn peek_last(&self) -> Option<&Self> {
        self.inner.peek_last()
    }

    pub fn as_str(&self) -> &str {
        &self.token.input
    }

    pub fn as_rule(&self) -> Rule {
        self.token.rule
    }

    pub fn into_inner(self) -> InnerPestIterator<'i> {
        self.inner
    }

    pub fn into_token(self) -> Token<'i> {
        self.token
    }

    pub fn decompose(self) -> (Token<'i>, InnerPestIterator<'i>) {
        (self.token, self.inner)
    }
}

impl<'i> IntoNode<'i> for PestIterator<'i> {
    fn into_node(self, state: &mut State) -> Result<Node<'i>, Error> {
        Node::from_iterator(self, state)
    }
}

impl<'i> From<Pair<'i, Rule>> for PestIterator<'i> {
    fn from(p: Pair<'i, Rule>) -> Self {
        Self::from_pair(p)
    }
}

impl<'i> Iterator for PestIterator<'i> {
    type Item = Self;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_child()
    }
}

impl<'i> DoubleEndedIterator for PestIterator<'i> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.last_child()
    }
}

impl<'i> ExactSizeIterator for PestIterator<'i> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

#[derive(Clone, Debug)]
pub struct InnerPestIterator<'i>(VecDeque<PestIterator<'i>>);
impl<'i> InnerPestIterator<'i> {
    pub fn from_pair(pair: Pair<'i, Rule>) -> Self {
        let inner = pair
            .into_inner()
            .filter(|p| !Token::is_symbol(p.as_rule()))
            .map(PestIterator::from)
            .collect();
        Self(inner)
    }

    pub fn from_vec(pairs: Vec<PestIterator<'i>>) -> Self {
        Self(pairs.into())
    }

    pub fn next_child(&mut self) -> Option<PestIterator<'i>> {
        self.0.pop_front()
    }

    pub fn last_child(&mut self) -> Option<PestIterator<'i>> {
        self.0.pop_back()
    }

    pub fn peek(&self) -> Option<&PestIterator<'i>> {
        self.0.front()
    }

    pub fn peek_last(&self) -> Option<&PestIterator<'i>> {
        self.0.back()
    }
}

impl<'i> Iterator for InnerPestIterator<'i> {
    type Item = PestIterator<'i>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_child()
    }
}

impl<'i> DoubleEndedIterator for InnerPestIterator<'i> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.last_child()
    }
}

impl<'i> ExactSizeIterator for InnerPestIterator<'i> {
    fn len(&self) -> usize {
        self.0.len()
    }
}
