#[macro_export]
macro_rules! define_prattnode {
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
        #[derive(Debug)]
        pub struct $name {
            $(pub $param: $param_t),+, token: crate::Token,
        }
        impl $name {
            pub const RULES: &'static [crate::Rule] = &[$(crate::Rule::$rule),*];
            pub fn new(input: crate::syntax_tree::pratt::PrattPair) -> Result<Node, Error> {
                ($new_hnd)(input)
            }
        }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.token().input)
            }
        }
        #[allow(clippy::redundant_closure_call)]
        impl crate::AstNode for $name {
            fn from_pair(input: ::pest::iterators::Pair<crate::pest::Rule>) -> Result<crate::Node, crate::Error> {
                crate::syntax_tree::pratt::Parser::parse(input)
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
            crate::syntax_tree::resolver::CollectibleNode::Pratt($name::RULES, $name::new)
        }

        document_operator!(
            name = $docs_name,
            rules = [$($rule),*],
            symbols = [$($docs_symbols),*],
            description = $docs_desc,
            examples = $docs_examples,
        );
    };
}
