use super::{pair::PrattPair, precedence_map::PRECEDENCE_MAP};
use crate::{
    pest::{Rule, ToAstNode},
    syntax_tree::Node,
    Error,
};
use pest::{iterators::Pair, pratt_parser::PrattParser};

pub struct Parser;
impl Parser {
    fn get_pratt_parser() -> PrattParser<Rule> {
        let mut pratt = PrattParser::new();
        for op_level in PRECEDENCE_MAP {
            let mut r = op_level[0].to_pratt();
            for op in *op_level {
                r = r | op.to_pratt();
            }
            pratt = pratt.op(r);
        }
        pratt
    }

    pub fn parse(input: Pair<Rule>) -> Result<Node, Error> {
        let pratt = Self::get_pratt_parser();
        let mut pratt = pratt
            .map_primary(|primary| PrattPair::Primary(primary))
            .map_infix(|l, o, r| PrattPair::Infix(Box::new(l), o, Box::new(r)))
            .map_prefix(|o, r| PrattPair::Prefix(o, Box::new(r)))
            .map_postfix(|l, o| PrattPair::Postfix(Box::new(l), o));
        pratt.parse(input.into_inner()).to_ast_node()
    }
}
