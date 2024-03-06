use crate::{Error, State, Token};
use enum_dispatch::enum_dispatch;
use polyvalue::Value;

use crate::Rule;

use super::{nodes::Node, pair::InnerPestIterator};

pub trait IntoOwned {
    type Owned;
    fn into_owned(self) -> Self::Owned;
}

/// Tree evaluation trait
#[enum_dispatch(CoreSyntaxNode)]
pub trait NodeExt<'i>
where
    Self: IntoOwned,
{
    fn evaluate(&self, state: &mut State) -> Result<Value, Error>;
    fn token(&self) -> &Token<'i>;
}

/// Tree construction trait
pub trait SyntaxNodeBuilderExt<'i> {
    fn build(
        pairs: InnerPestIterator<'i>,
        token: Token<'i>,
        state: &mut State,
    ) -> Result<Node<'i>, Error>;
}

pub trait IntoNode<'i> {
    fn into_node(self, state: &mut State) -> Result<Node<'i>, Error>;
}
impl<'i> IntoNode<'i> for pest::iterators::Pair<'i, Rule> {
    fn into_node(self, state: &mut State) -> Result<Node<'i>, Error> {
        Node::from_pair(self, state)
    }
}
