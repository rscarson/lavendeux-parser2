use crate::{AstNode, Error, ToAstNode};

macro_rules! define_node {
    ($name:ident {$($param:ident : $param_t:ty),+}, rules = [$($rule:ident),+], new = $new_hnd:expr, value = $get_hnd:expr) => {
        #[derive(Debug)]
        pub struct $name {
            $(pub $param: $param_t),+, token: crate::Token,
        }
        impl $name {
            pub const RULES: &'static [crate::Rule] = &[$(crate::Rule::$rule),+];
        }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.token().input)
            }
        }
        impl crate::AstNode for $name {
            fn from_pair(input: pest::iterators::Pair<crate::pest::Rule>) -> Result<crate::Node, crate::Error> {
                ($new_hnd)(input)
            }

            fn get_value(&mut self, state: &mut crate::State) -> Result<crate::Value, crate::Error> {
                state.check_timer()?;
                ($get_hnd)(self, state)
            }

            fn token(&self) -> &crate::Token {
                &self.token
            }

            fn boxed(self) -> crate::Node
            where
                Self: Sized + 'static,
            {
                Box::new(self)
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
        }
    };
    ($name:ident, rules = [$($rule:ident),+], new = $new_hnd:expr, value = $get_hnd:expr) => {
        #[derive(Debug, Clone)]
        pub struct $name {
            token: crate::Token,
        }
        impl $name {
            pub const RULES: &'static [crate::Rule] = &[$(crate::Rule::$rule),+];
        }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.token().input)
            }
        }
        impl crate::AstNode for $name {
            fn from_pair(input: pest::iterators::Pair<crate::pest::Rule>) -> Result<crate::Node, crate::Error> {
                ($new_hnd)(input)
            }

            fn get_value(&mut self, state: &mut crate::State) -> Result<crate::Value, crate::Error> {
                state.check_timer()?;
                ($get_hnd)(self, state)
            }

            fn token(&self) -> &crate::Token {
                &self.token
            }

            fn boxed(self) -> crate::Node
            where
                Self: Sized + 'static,
            {
                Box::new(self)
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
        }
    };
}

pub type Node = Box<dyn AstNode>;

impl TryFrom<&str> for Node {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        crate::pest::parse_input(value, crate::pest::Rule::SCRIPT)
    }
}

mod boolean;
pub use boolean::*;

mod bitwise;
pub use bitwise::*;

mod arithmetic;
pub use arithmetic::*;

mod unary;
use lazy_static::lazy_static;
pub use unary::*;

mod assignment;
pub use assignment::*;

mod values;
pub use values::*;

mod core;
pub use core::*;

mod error;
pub use error::*;

mod function;
pub use function::*;

macro_rules! include_node {
    ($map:expr, $node:ident) => {
        $map.extend($node::RULES.iter().map(|r| (*r, $node::from_pair as _)));
    };
}

use std::collections::HashMap;
lazy_static! {
    static ref NODES: HashMap<crate::Rule, fn(pest::iterators::Pair<crate::Rule>) -> Result<Node, Error>> = {
        let mut map = HashMap::new();

        //
        // Values
        include_node!(map, ValueLiteral);
        include_node!(map, ConstantValue);
        include_node!(map, Identifier);
        include_node!(map, ArrayValue);
        include_node!(map, ObjectValue);
        include_node!(map, RangeValue);
        include_node!(map, CastingExpression);
        include_node!(map, DeleteExpression);

        //
        // Unary
        include_node!(map, IndexingExpression);

        //
        // Functions
        include_node!(map, FunctionCall);

        //
        // Errors
        include_node!(map, UnterminatedLinebreak);
        include_node!(map, UnterminatedLiteral);
        include_node!(map, UnterminatedComment);
        include_node!(map, UnterminatedArray);
        include_node!(map, UnterminatedObject);
        include_node!(map, UnterminatedParen);
        include_node!(map, UnexpectedDecorator);

        //
        // Core
        include_node!(map, Script);
        include_node!(map, Line);
        include_node!(map, TernaryExpression);
        include_node!(map, ForLoopExpression);

        //
        // Boolean
        include_node!(map, BooleanExpression);
        include_node!(map, BooleanNotExpression);
        include_node!(map, MatchingExpression);

        //
        // Bitwise
        include_node!(map, BitwiseExpression);
        include_node!(map, BitwiseNotExpression);

        //
        // Arithmetic
        include_node!(map, ArithmeticExpression);
        include_node!(map, ArithmeticNegExpression);

        //
        // Assignments
        include_node!(map, FunctionAssignment);
        include_node!(map, VariableAssignment);
        include_node!(map, DestructuringAssignment);
        include_node!(map, IndexAssignment);

        map
    };
}

pub fn node_map(
) -> &'static HashMap<crate::Rule, fn(pest::iterators::Pair<crate::Rule>) -> Result<Node, Error>> {
    &NODES
}
