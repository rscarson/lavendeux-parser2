use super::pratt::PrattPair;
use crate::{document_operator, error::Error, AstNode, ToAstNode};

macro_rules! define_node {
    (
        $name:ident {$($param:ident : $param_t:ty),+},
        rules = [$($rule:ident),*],
        new = $new_hnd:expr,
        value = $get_hnd:expr,
        docs = {
            name: $docs_name:literal,
            symbols = [$($docs_symbols:literal),*],
            description: $docs_desc:literal,
            examples: $docs_examples:literal,
        }
    ) => {
        define_node!($name {$($param : $param_t),*}, rules = [$($rule),*], new = $new_hnd, value = $get_hnd);
        document_operator!(
            name = $docs_name,
            rules = [$($rule),*],
            symbols = [$($docs_symbols),*],
            description = $docs_desc,
            examples = $docs_examples,
        );
    };
    (
        $name:ident {$($param:ident : $param_t:ty),+},
        rules = [$($rule:ident),*],
        new = $new_hnd:expr,
        value = $get_hnd:expr
    ) => {
        #[derive(Debug)]
        pub struct $name {
            $(pub $param: $param_t),+, token: crate::Token,
        }
        impl $name {
            pub const RULES: &'static [crate::Rule] = &[$(crate::Rule::$rule),*];
        }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.token().input)
            }
        }
        #[allow(clippy::redundant_closure_call)]
        impl crate::AstNode for $name {
            fn from_pair(input: pest::iterators::Pair<crate::pest::Rule>) -> Result<crate::Node, crate::Error> {
                ($new_hnd)(input)
            }

            fn get_value(&self, state: &mut crate::State) -> Result<crate::Value, crate::Error> {
                state.check_timer()?;
                ($get_hnd)(self, state)
            }

            fn token(&self) -> &crate::Token {
                &self.token
            }

            fn token_offsetline(&mut self, offset: usize) {
                self.token.line += offset;
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

        inventory::submit! {
            crate::syntax_tree::resolver::CollectibleNode::Node($name::RULES, $name::from_pair)
        }
    };

    (
        $name:ident,
        rules = [$($rule:ident),*],
        new = $new_hnd:expr,
        value = $get_hnd:expr,
        docs = {
            name: $docs_name:literal,
            symbols = [$($docs_symbols:literal),*],
            description: $docs_desc:literal,
            examples: $docs_examples:literal,
        }
    ) => {
        define_node!($name, rules = [$($rule),*], new = $new_hnd, value = $get_hnd);
        document_operator!(
            name = $docs_name,
            rules = [$($rule),*],
            symbols = [$($docs_symbols),*],
            description = $docs_desc,
            examples = $docs_examples,
        );
    };

    ($name:ident, rules = [$($rule:ident),*], new = $new_hnd:expr, value = $get_hnd:expr) => {
        #[derive(Debug, Clone)]
        pub struct $name {
            token: crate::Token,
        }
        impl $name {
            pub const RULES: &'static [crate::Rule] = &[$(crate::Rule::$rule),*];
        }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.token().input)
            }
        }
        #[allow(clippy::redundant_closure_call)]
        impl crate::AstNode for $name {
            fn from_pair(input: pest::iterators::Pair<crate::pest::Rule>) -> Result<crate::Node, crate::Error> {
                ($new_hnd)(input)
            }

            fn get_value(&self, state: &mut crate::State) -> Result<crate::Value, crate::Error> {
                state.check_timer()?;
                ($get_hnd)(self, state)
            }

            fn token(&self) -> &crate::Token {
                &self.token
            }

            fn token_offsetline(&mut self, offset: usize) {
                self.token.line += offset;
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

        inventory::submit! {
            crate::syntax_tree::resolver::CollectibleNode::Node($name::RULES, $name::from_pair)
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

mod core;

// Pratt nodes
pub mod arithmetic;
pub mod assignment;
pub mod bitwise;
pub mod boolean;
pub mod matching;

// Mixed nodes
pub mod collections;
pub mod functions;
pub mod keyword;
pub mod literals;
