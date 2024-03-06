macro_rules! define_prattnode {
    (
        $name:ident {$($param:ident : $param_t:ty),+},
        rules = [$($rule:ident),*],
        new = ($new_hndvar:ident) $new_hnd:block,
        value = ($get_hndself:ident, $get_hndstate:ident) $get_hnd:block,
        into_owned = ($to_owned_hndself:ident) $to_owned_hnd:block,
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
            fn from_pratt($new_hndvar: crate::syntax_tree::pratt::PrattPair<'i>) -> Result<crate::Node<'i>, crate::Error> {
                $new_hnd
            }

            fn get_value(&self, $get_hndstate: &mut crate::State) -> Result<crate::Value, crate::Error> {
                let $get_hndself = self;
                $get_hndstate.check_timer()?;
                $get_hnd
            }

            fn token(&self) -> &$crate::Token<'i> {
                &self.token
            }

            fn token_offsetline(&mut self, offset: usize) {
                self.token.line += offset;
            }

            fn into_owned(self) -> Self {
                let $to_owned_hndself = self;
                $to_owned_hnd
            }
        }

        define_resolver!($name);

        document_operator! {
            name = $docs_name,
            rules = [$($rule),*],
            symbols = [$($docs_symbols),*],
            description = $docs_desc,
            examples = $docs_examples,
        }
    };
}
