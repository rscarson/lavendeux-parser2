use crate::pest::Rule;
use pest::{
    iterators::Pair,
    pratt_parser::{Assoc, Op, PrattParser},
};

use super::pair::PestIterator;

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

    pub fn parse<'i, I>(input: I) -> PestIterator<'i>
    where
        I: Iterator<Item = Pair<'i, Rule>> + ExactSizeIterator,
    {
        let pratt = Self::get_pratt_parser();
        let mut pratt = pratt
            .map_primary(PestIterator::from)
            .map_infix(PestIterator::from_infix)
            .map_prefix(PestIterator::from_prefix)
            .map_postfix(PestIterator::from_postfix);
        pratt.parse(input)
    }
}

#[derive(Clone, Debug)]
pub enum PrattOperatorType {
    Prefix,
    Infix,
    Postfix,
}
pub struct PrattOperator {
    ty: PrattOperatorType,
    rule: Rule,
    asoc: Assoc,
}

impl PrattOperator {
    pub fn to_pratt(&self) -> Op<Rule> {
        match self.ty {
            PrattOperatorType::Prefix => Op::prefix(self.rule),
            PrattOperatorType::Infix => Op::infix(self.rule, self.asoc),
            PrattOperatorType::Postfix => Op::postfix(self.rule),
        }
    }
}

macro_rules! prefix {
    ($rule:ident) => {
        PrattOperator {
            ty: PrattOperatorType::Prefix,
            rule: Rule::$rule,
            asoc: Assoc::Left,
        }
    };
}
macro_rules! infix {
    ($rule:ident, $asso:ident) => {
        PrattOperator {
            ty: PrattOperatorType::Infix,
            rule: Rule::$rule,
            asoc: Assoc::$asso,
        }
    };
}
macro_rules! postfix {
    ($rule:ident) => {
        PrattOperator {
            ty: PrattOperatorType::Postfix,
            rule: Rule::$rule,
            asoc: Assoc::Left,
        }
    };
}

/// The precedence map for the Pratt parser
/// This is a list of lists of all EXPR operators, where each list is a level of precedence
/// from lowest to highest precedence
pub const PRECEDENCE_MAP: &[&[PrattOperator]] = &[
    // Assignment
    &[
        infix!(OP_ASSIGN_ADD, Right),
        infix!(OP_ASSIGN_SUB, Right),
        infix!(OP_ASSIGN_POW, Right),
        infix!(OP_ASSIGN_MUL, Right),
        infix!(OP_ASSIGN_DIV, Right),
        infix!(OP_ASSIGN_MOD, Right),
        infix!(OP_BASSIGN_AND, Right),
        infix!(OP_BASSIGN_OR, Right),
        infix!(OP_ASSIGN_OR, Right),
        infix!(OP_ASSIGN_AND, Right),
        infix!(OP_ASSIGN_XOR, Right),
        infix!(OP_ASSIGN_SL, Right),
        infix!(OP_ASSIGN_SR, Right),
        infix!(OP_ASSIGN, Right),
    ],
    // Delete
    &[prefix!(PREFIX_DEL)],
    // Range, Ternary
    &[infix!(OP_RANGE, Left)],
    &[infix!(OP_TERNARY, Right)],
    // Decorator
    &[postfix!(POSTFIX_DECORATE)],
    //
    // Logical OR, Logical AND
    &[infix!(OP_BOOL_OR, Left)],
    &[infix!(OP_BOOL_AND, Left)],
    //
    // Pattern Matching
    &[
        infix!(OP_MATCH_MATCHES, Left),
        infix!(OP_MATCH_CONTAINS, Left),
        infix!(OP_MATCH_IS, Left),
        infix!(OP_MATCH_STARTSWITH, Left),
        infix!(OP_MATCH_ENDSWITH, Left),
    ],
    //
    // Bitwise OR, Bitwise XOR, Bitwise AND
    &[infix!(OP_BIT_OR, Left)],
    &[infix!(OP_BIT_XOR, Left)],
    &[infix!(OP_BIT_AND, Left)],
    //
    // == and !=, followed by <, <=, >, >=
    &[infix!(OP_BOOL_EQ, Left), infix!(OP_BOOL_NE, Left)],
    &[
        infix!(OP_BOOL_LT, Left),
        infix!(OP_BOOL_LE, Left),
        infix!(OP_BOOL_GT, Left),
        infix!(OP_BOOL_GE, Left),
    ],
    //
    // << and >>
    &[infix!(OP_BIT_SL, Left), infix!(OP_BIT_SR, Left)],
    //
    // Add and subtract, followed by multiply divide and mod, then pow
    &[infix!(OP_ADD, Left), infix!(OP_SUB, Left)],
    &[
        infix!(OP_MUL, Left),
        infix!(OP_DIV, Left),
        infix!(OP_MOD, Left),
    ],
    &[infix!(OP_POW, Right)],
    //
    // Cast, and prefix operators
    &[
        infix!(OP_CAST, Right),
        prefix!(PREFIX_NEG),
        prefix!(PREFIX_INC),
        prefix!(PREFIX_DEC),
        prefix!(PREFIX_BOOL_NOT),
        prefix!(PREFIX_BIT_NOT),
    ],
    //
    // Postfix operators
    &[postfix!(POSTFIX_CALL), postfix!(POSTFIX_INDEX)],
];
