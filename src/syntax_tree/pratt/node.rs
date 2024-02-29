macro_rules! define_prattnode {
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
            fn from_pair(input: &::pest::iterators::Pair<crate::pest::Rule>) -> Result<crate::Node<'i>, crate::Error<'i>> {
                crate::syntax_tree::pratt::Parser::parse(input.into_inner())
            }
            fn from_pratt($new_hndvar: &crate::syntax_tree::pratt::PrattPair) -> Result<crate::Node<'i>, crate::Error<'i>> $new_hnd

            fn get_value(&self, $get_hndstate: &mut crate::State) -> Result<crate::Value, crate::Error<'i>> {
                let $get_hndself = self;
                $get_hndstate.check_timer()?;
                $get_hnd
            }

            fn token(&self) -> &crate::Token {
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

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
        }

        paste::paste! {
            #[allow(non_camel_case_types)]
            pub struct [<_noderesolver_$name>];
            impl crate::syntax_tree::resolver::NodeResolver for [<_noderesolver_$name>] {
                fn handle<'i>(&self, pair: &pest::iterators::Pair<'i, crate::pest::Rule>) -> Result<crate::Node<'i>, crate::Error<'i>> {
                    $name::from_pair(pair)
                }
                fn handle_pratt<'i>(&self, pair: &crate::syntax_tree::pratt::PrattPair<'i>) -> Result<crate::Node<'i>, crate::Error<'i>> {
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

        document_operator!(
            name = $docs_name,
            rules = [$($rule),*],
            symbols = [$($docs_symbols),*],
            description = $docs_desc,
            examples = $docs_examples,
        );
    };
}
