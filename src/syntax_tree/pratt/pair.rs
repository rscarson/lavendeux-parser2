use crate::{
    pest::{AstNode, Rule, ToAstNode},
    syntax_tree::resolver,
    token::ToToken,
    Error, Token,
};
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub enum PrattPair<'i> {
    Primary(Pair<'i, Rule>),
    Infix(Box<PrattPair<'i>>, Pair<'i, Rule>, Box<PrattPair<'i>>),
    Prefix(Pair<'i, Rule>, Box<PrattPair<'i>>),
    Postfix(Box<PrattPair<'i>>, Pair<'i, Rule>),
}
impl<'a> PrattPair<'a> {
    pub fn into_inner(self) -> std::vec::IntoIter<PrattPair<'a>> {
        match self {
            PrattPair::Primary(p) => vec![PrattPair::Primary(p)],
            PrattPair::Infix(l, o, r) => {
                vec![*l, PrattPair::Primary(o), *r]
            }
            PrattPair::Prefix(o, r) => vec![PrattPair::Primary(o), *r],
            PrattPair::Postfix(l, o) => vec![*l, PrattPair::Primary(o)],
        }
        .into_iter()
    }

    pub fn first_pair(&self) -> &Pair<'_, Rule> {
        match self {
            PrattPair::Primary(p) => p,
            PrattPair::Infix(l, _, _) => l.first_pair(),
            PrattPair::Prefix(o, _) => o,
            PrattPair::Postfix(l, _) => l.first_pair(),
        }
    }

    pub fn as_rule(&self) -> Rule {
        match self {
            PrattPair::Primary(p) => p.as_rule(),
            PrattPair::Infix(_, o, _) => o.as_rule(),
            PrattPair::Prefix(o, _) => o.as_rule(),
            PrattPair::Postfix(_, o) => o.as_rule(),
        }
    }

    pub fn as_token(&self) -> Token {
        match self {
            PrattPair::Primary(p) => p.to_token(),
            PrattPair::Infix(l, o, r) => {
                let mut token = l.as_token();
                let op_token = o.to_token();
                token.rule = op_token.rule;
                token.input = op_token.input; /*format!(
                                                  "{} {} {}",
                                                  l.as_token().input,
                                                  op_token.input,
                                                  r.as_token().input
                                              );*/
                token
            }
            PrattPair::Prefix(o, r) => {
                let mut token = o.to_token();
                //  token.input = format!("{} {}", token.input, r.as_token().input);
                token
            }
            PrattPair::Postfix(l, o) => {
                //   let mut token = l.as_token();
                let op_token = o.to_token();
                //     token.rule = op_token.rule;
                //      token.input = format!("{}{}", l.as_token().input, op_token.input);
                op_token
            }
        }
    }
}

impl<'i> ToAstNode<'i> for PrattPair<'i> {
    /// Convert a pest pair into an AST node
    /// This maps all the rules to AST Node structures
    fn to_ast_node(&self) -> Result<Box<dyn AstNode<'i>>, Error<'i>> {
        resolver::handle_pratt(self)
    }
}
