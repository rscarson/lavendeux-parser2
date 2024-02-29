use super::pratt::PrattPair;
use crate::{error::Error, AstNode, ToAstNode};

macro_rules! define_node {
    (
        $name:ident {$($param:ident : $param_t:ty),+},
        rules = [$($rule:ident),*],
        new = ($new_hndvar:ident) $new_hnd:block,
        value = ($get_hndself:ident, $get_hndstate:ident) $get_hnd:block,
        docs = {
            name: $docs_name:literal,
            symbols = [$($docs_symbols:literal),*],
            description: $docs_desc:literal,
            examples: $docs_examples:literal,
        }
    ) => {
        define_node!($name {$($param : $param_t),*}, rules = [$($rule),*], new = ($new_hndvar) $new_hnd, value = ($get_hndself, $get_hndstate) $get_hnd);
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
        new = ($new_hndvar:ident) $new_hnd:block,
        value = ($get_hndself:ident, $get_hndstate:ident) $get_hnd:block
    ) => {
        #[derive(Debug)]
        pub struct $name<'i> {
            $(pub $param: $param_t),+, token: crate::Token<'i>,
        }
        impl $name<'_> {
            pub const RULES: &'static [crate::Rule] = &[$(crate::Rule::$rule),*];
        }
        impl std::fmt::Display for $name<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.token().input)
            }
        }

        impl<'i> crate::AstNode<'i> for $name<'i> {
            fn from_pair($new_hndvar: pest::iterators::Pair<'i, crate::pest::Rule>) -> Result<crate::Node<'i>, crate::Error<'i>> $new_hnd
            fn from_pratt(input: crate::syntax_tree::pratt::PrattPair<'i>) -> Result<crate::Node<'i>, crate::Error<'i>> {
                let $new_hndvar = input.first_pair();
                Self::from_pair($new_hndvar)
            }

            fn get_value<'state>(&self, $get_hndstate: &'state mut crate::State) -> Result<crate::Value, crate::Error<'state>> {
                let $get_hndself = self;
                $get_hndstate.check_timer()?;
                $get_hnd
            }

            fn token(&self) -> &crate::Token<'i> {
                &self.token
            }

            fn token_offsetline(&mut self, offset: usize) {
                self.token.line += offset;
            }

            fn boxed(self) -> crate::Node<'i>
            where
                Self: Sized + 'static,
            {
                Box::new(self)
            }
        }

        paste::paste! {
            #[allow(non_camel_case_types)]
            pub struct [<_noderesolver_$name>];
            impl crate::syntax_tree::resolver::NodeResolver for [<_noderesolver_$name>] {
                fn handle<'i>(&self, pair: pest::iterators::Pair<'i, crate::pest::Rule>) -> Result<crate::Node<'i>, crate::Error<'i>> {
                    $name::from_pair(pair)
                }
                fn handle_pratt<'i>(&self, pair: crate::syntax_tree::pratt::PrattPair<'i>) -> Result<crate::Node<'i>, crate::Error<'i>> {
                    $name::from_pratt(pair)
                }
                fn rules(&self) -> &'static [crate::Rule] {
                    $name::RULES
                }
            }

            inventory::submit! {
                &[<_noderesolver_$name>] as &'static dyn $crate::syntax_tree::resolver::NodeResolver
            }
        }
    };

    (
        $name:ident,
        rules = [$($rule:ident),*],
        new = ($new_hndvar:ident) $new_hnd:block,
        value = ($get_hndself:ident, $get_hndstate:ident) $get_hnd:block,
        docs = {
            name: $docs_name:literal,
            symbols = [$($docs_symbols:literal),*],
            description: $docs_desc:literal,
            examples: $docs_examples:literal,
        }
    ) => {
        define_node!($name, rules = [$($rule),*], new = ($new_hndvar) $new_hnd, value = ($get_hndself, $get_hndstate) $get_hnd);
        document_operator!(
            name = $docs_name,
            rules = [$($rule),*],
            symbols = [$($docs_symbols),*],
            description = $docs_desc,
            examples = $docs_examples,
        );
    };

    ($name:ident, rules = [$($rule:ident),*], new = ($new_hndvar:ident) $new_hnd:block, value = ($get_hndself:ident, $get_hndstate:ident) $get_hnd:block) => {
        #[derive(Debug, Clone)]
        pub struct $name<'i> {
            token: crate::Token<'i>,
        }
        impl $name<'_> {
            pub const RULES: &'static [crate::Rule] = &[$(crate::Rule::$rule),*];
        }
        impl std::fmt::Display for $name<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.token().input)
            }
        }

        impl<'i> crate::AstNode<'i> for $name<'i> {
            fn from_pair($new_hndvar: pest::iterators::Pair<'i, crate::pest::Rule>) -> Result<crate::Node<'i>, crate::Error<'i>> $new_hnd
            fn from_pratt(input: crate::syntax_tree::pratt::PrattPair<'i>) -> Result<crate::Node<'i>, crate::Error<'i>> {
                let $new_hndvar = input.first_pair();
                Self::from_pair($new_hndvar)
            }

            fn get_value<'state>(&self, $get_hndstate: &'state mut crate::State) -> Result<crate::Value, crate::Error<'state>> {
                let $get_hndself = self;
                $get_hndstate.check_timer()?;
                $get_hnd
            }

            fn token(&self) -> &crate::Token<'i> {
                &self.token
            }

            fn token_offsetline(&mut self, offset: usize) {
                self.token.line += offset;
            }

            fn boxed(self) -> crate::Node<'i>
            where
                Self: Sized + 'static,
            {
                Box::new(self)
            }
        }

        paste::paste! {
            #[allow(non_camel_case_types)]
            pub struct [<_noderesolver_$name>];
            impl crate::syntax_tree::resolver::NodeResolver for [<_noderesolver_$name>] {
                fn handle<'i>(&self, pair: pest::iterators::Pair<'i, crate::pest::Rule>) -> Result<crate::Node<'i>, crate::Error<'i>> {
                    $name::from_pair(pair)
                }
                fn handle_pratt<'i>(&self, pair: crate::syntax_tree::pratt::PrattPair<'i>) -> Result<crate::Node<'i>, crate::Error<'i>> {
                    $name::from_pratt(pair)
                }
                fn rules(&self) -> &'static [crate::Rule] {
                    $name::RULES
                }
            }

            inventory::submit! {
                &[<_noderesolver_$name>] as &'static dyn $crate::syntax_tree::resolver::NodeResolver
            }
        }
    };
}

pub type Node<'i> = Box<dyn AstNode<'i>>;

impl<'i> TryFrom<&'i str> for Node<'i> {
    type Error = Error<'i>;

    fn try_from(value: &'i str) -> Result<Self, Error<'i>> {
        crate::pest::parse_input(value, crate::pest::Rule::SCRIPT)
    }
}

mod core;
mod errors;

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
