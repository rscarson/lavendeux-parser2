#![allow(unused_imports)]
use crate::{error::WrapExternalError, token, Error, Rule, State, Token};
use pest::iterators::Pair;
use polyvalue::Value;

use super::{
    pair::PestIterator,
    traits::{IntoOwned, NodeExt, SyntaxNodeBuilderExt},
};

mod assignment;
use assignment::Assignment;

mod arithmetic;
use arithmetic::Arithmetic;

mod bitwise;
use bitwise::Bitwise;

mod boolean;
use boolean::Boolean;

mod collections;
use collections::Collections;

mod core;
use core::Core;

mod functions;
use functions::Functions;

mod iterators;

mod conditionals;
use conditionals::Conditionals;

mod values;
pub use values::Reference;
use values::Values;

mod literals;

/// Root type for AST nodes, split by class of node
#[derive(Debug, Clone)]
pub enum Node<'i> {
    /// Core syntax elements (script and block)
    Core(core::Core<'i>),

    /// Variable storate (identifiers, assignment and deletion)
    Assignment(assignment::Assignment<'i>),

    /// Value manipulation (references, decorators, match, cast, etc.)
    Values(values::Values<'i>),

    /// Collections (arrays, objects, ranges, etc.)
    Collections(collections::Collections<'i>),

    /// Conditional expressions (if, ternary, switch)
    Conditionals(conditionals::Conditionals<'i>),

    /// Iterators (for loops)
    Iterators(iterators::Iterators<'i>),

    /// Function assignment and calling
    Functions(functions::Functions<'i>),

    /// Arithmetic expressions (add, sub, mul, div, etc.)
    Arithmetic(arithmetic::Arithmetic<'i>),

    /// Bitwise expressions
    Bitwise(bitwise::Bitwise<'i>),

    /// Boolean expressions
    Boolean(boolean::Boolean<'i>),

    /// Literal constants
    Literal(Value, Token<'i>),
}
impl Node<'_> {
    /// This is where rules are matched to node-builder types
    pub fn from_pair<'i>(pair: Pair<'i, Rule>, state: &mut State) -> Result<Node<'i>, Error> {
        let pairs = PestIterator::from(pair);
        Self::from_iterator(pairs, state)
    }

    pub(crate) fn from_iterator<'i>(
        pairs: PestIterator<'i>,
        state: &mut State,
    ) -> Result<Node<'i>, Error> {
        let (token, pairs) = pairs.decompose();

        // println!("{:#?}", pairs);

        match token.rule {
            //
            // Core nodes
            Rule::SCRIPT => core::Script::build(pairs, token, state),
            Rule::BLOCK => core::Block::build(pairs, token, state),

            //
            // Value Literals
            Rule::int_literal => literals::IntLiteral::build(pairs, token, state),
            Rule::float_literal | Rule::sci_literal => {
                literals::FloatLiteral::build(pairs, token, state)
            }
            Rule::string_literal => literals::StringLiteral::build(pairs, token, state),
            Rule::bool_literal => literals::BoolLiteral::build(pairs, token, state),
            Rule::regex_literal => literals::RegexLiteral::build(pairs, token, state),
            Rule::fixed_literal => literals::FixedLiteral::build(pairs, token, state),
            Rule::currency_literal => literals::CurrencyLiteral::build(pairs, token, state),
            Rule::const_literal => literals::ConstLiteral::build(pairs, token, state),

            //
            // Value expressions
            Rule::identifier => values::Identifier::build(pairs, token, state),
            Rule::OP_CAST => values::CastExpression::build(pairs, token, state),
            Rule::POSTFIX_DECORATE => values::DecoratorExpression::build(pairs, token, state),

            //
            // Matching expressions
            Rule::OP_MATCH_CONTAINS
            | Rule::OP_MATCH_MATCHES
            | Rule::OP_MATCH_IS
            | Rule::OP_MATCH_STARTSWITH
            | Rule::OP_MATCH_ENDSWITH => values::MatchingExpression::build(pairs, token, state),

            //
            // Collection nodes
            Rule::ARRAY_TERM => collections::Array::build(pairs, token, state),
            Rule::OBJECT_TERM => collections::Object::build(pairs, token, state),
            Rule::OP_RANGE => collections::Range::build(pairs, token, state),
            Rule::POSTFIX_INDEX => collections::IndexingExpression::build(pairs, token, state),

            //
            // Iterator nodes
            Rule::BREAK_KEYWORD => iterators::KeywordBreak::build(pairs, token, state),
            Rule::SKIP_KEYWORD => iterators::KeywordContinue::build(pairs, token, state),
            Rule::FOR_LOOP_EXPRESSION => iterators::ForLoopExpression::build(pairs, token, state),

            //
            // Conditional nodes
            Rule::IF_EXPRESSION => conditionals::IfExpression::build(pairs, token, state),
            Rule::OP_TERNARY => conditionals::TernaryExpression::build(pairs, token, state),
            Rule::SWITCH_EXPRESSION => conditionals::SwitchExpression::build(pairs, token, state),

            //
            // Arithmetic
            Rule::PREFIX_NEG => arithmetic::ArithmeticNeg::build(pairs, token, state),
            Rule::OP_ADD
            | Rule::OP_SUB
            | Rule::OP_POW
            | Rule::OP_DIV
            | Rule::OP_MOD
            | Rule::OP_MUL => arithmetic::ArithmeticExpr::build(pairs, token, state),

            //
            // Bitwise
            Rule::PREFIX_BIT_NOT => bitwise::BitwiseNot::build(pairs, token, state),
            Rule::OP_BIT_OR
            | Rule::OP_BIT_XOR
            | Rule::OP_BIT_AND
            | Rule::OP_BIT_SL
            | Rule::OP_BIT_SR => bitwise::BitwiseExpr::build(pairs, token, state),

            //
            // Boolean
            Rule::PREFIX_BOOL_NOT => boolean::BooleanNot::build(pairs, token, state),
            Rule::OP_BOOL_OR
            | Rule::OP_BOOL_AND
            | Rule::OP_BOOL_EQ
            | Rule::OP_BOOL_NE
            | Rule::OP_BOOL_LE
            | Rule::OP_BOOL_GE
            | Rule::OP_BOOL_LT
            | Rule::OP_BOOL_GT => boolean::BooleanExpr::build(pairs, token, state),

            //
            // Functions
            Rule::FUNCTION_ASSIGNMENT_STATEMENT => {
                functions::FunctionDefinition::build(pairs, token, state)
            }
            Rule::POSTFIX_CALL => functions::FunctionCall::build(pairs, token, state),
            Rule::RETURN_EXPRESSION => functions::KeywordReturn::build(pairs, token, state),

            //
            // Assignment
            Rule::OP_ASSIGN_ADD
            | Rule::OP_ASSIGN_SUB
            | Rule::OP_ASSIGN_POW
            | Rule::OP_ASSIGN_MUL
            | Rule::OP_ASSIGN_DIV
            | Rule::OP_ASSIGN_MOD
            | Rule::OP_ASSIGN_AND
            | Rule::OP_ASSIGN_XOR
            | Rule::OP_ASSIGN_OR
            | Rule::OP_ASSIGN_SL
            | Rule::OP_ASSIGN_SR
            | Rule::OP_BASSIGN_AND
            | Rule::OP_BASSIGN_OR
            | Rule::OP_ASSIGN => assignment::AssignmentExpression::build(pairs, token, state),
            Rule::PREFIX_DEL => assignment::DeleteExpression::build(pairs, token, state),

            //
            // Errors
            Rule::UNTERMINATED_BLOCK_COMMENT => oops!(UnterminatedComment),
            Rule::UNTERMINATED_STRING_LITERAL => oops!(UnterminatedLiteral),
            Rule::UNCLOSED_BRACKET => oops!(UnterminatedArray),
            Rule::UNCLOSED_BRACE => oops!(UnterminatedObject),
            Rule::UNCLOSED_PAREN => oops!(UnterminatedParen),
            Rule::MISSING_LINEBREAK => oops!(UnterminatedLinebreak),

            _ => panic!("No node builder for rule {:?}", token.rule),
        }
    }
}
impl IntoOwned for Node<'_> {
    type Owned = Node<'static>;
    fn into_owned(self) -> Self::Owned {
        match self {
            Self::Core(node) => Self::Owned::Core(node.into_owned()),
            Self::Assignment(node) => Self::Owned::Assignment(node.into_owned()),
            Self::Collections(node) => Self::Owned::Collections(node.into_owned()),
            Self::Values(node) => Self::Owned::Values(node.into_owned()),
            Self::Arithmetic(node) => Self::Owned::Arithmetic(node.into_owned()),
            Self::Functions(node) => Self::Owned::Functions(node.into_owned()),
            Self::Iterators(node) => Self::Owned::Iterators(node.into_owned()),
            Self::Conditionals(node) => Self::Owned::Conditionals(node.into_owned()),
            Self::Bitwise(node) => Self::Owned::Bitwise(node.into_owned()),
            Self::Boolean(node) => Self::Owned::Boolean(node.into_owned()),
            Self::Literal(value, token) => Self::Owned::Literal(value.clone(), token.into_owned()),
        }
    }
}
impl<'i> NodeExt<'i> for Node<'i> {
    fn evaluate(&self, state: &mut State) -> Result<Value, Error> {
        match self {
            Self::Core(node) => node.evaluate(state),
            Self::Assignment(node) => node.evaluate(state),
            Self::Collections(node) => node.evaluate(state),
            Self::Values(node) => node.evaluate(state),
            Self::Arithmetic(node) => node.evaluate(state),
            Self::Functions(node) => node.evaluate(state),
            Self::Iterators(node) => node.evaluate(state),
            Self::Conditionals(node) => node.evaluate(state),
            Self::Bitwise(node) => node.evaluate(state),
            Self::Boolean(node) => node.evaluate(state),
            Self::Literal(value, ..) => Ok(value.clone()),
        }
    }

    fn token(&self) -> &Token<'i> {
        match self {
            Self::Core(node) => node.token(),
            Self::Assignment(node) => node.token(),
            Self::Collections(node) => node.token(),
            Self::Values(node) => node.token(),
            Self::Arithmetic(node) => node.token(),
            Self::Functions(node) => node.token(),
            Self::Iterators(node) => node.token(),
            Self::Conditionals(node) => node.token(),
            Self::Bitwise(node) => node.token(),
            Self::Boolean(node) => node.token(),
            Self::Literal(.., token) => token,
        }
    }
}
