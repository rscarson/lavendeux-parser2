macro_rules! define_astnode {
    (
        outer = $oname:ident,
        $name:ident $({$($param:ident : $param_t:ty),*})?,
        build = ($pairsvar:ident, $btokvar:ident, $bstatevar:ident) $build_hnd:block,
        eval  = ($selfvar:ident, $estatevar:ident) $eval_hnd:block,
        owned = ($oselfvar:ident) $owned_hnd:block
        $(
                ,docs  = {
                name: $docs_name:literal,
                symbols = [$($docs_symbols:literal),*],
                description: $docs_desc:literal,
                examples: $docs_examples:literal,
            }
        )?
    ) => {
        $(
            document_operator! {
                name = $docs_name,
                rules = [],
                symbols = [$($docs_symbols),*],
                description = $docs_desc,
                examples = $docs_examples,
            }
        )?

        #[allow(missing_docs)]
        #[derive(Debug, Clone)]
        pub struct $name<'i> {
            $($(pub $param: $param_t,)*)?
            token: crate::Token<'i>,
        }
        impl crate::syntax_tree::traits::IntoOwned for $name<'_> {
            type Owned = $name<'static>;
            fn into_owned(self) -> Self::Owned {
                let $oselfvar = self;
                $owned_hnd
            }
        }
        impl<'i> crate::syntax_tree::traits::NodeExt<'i> for $name<'i> {
            fn evaluate(&self, $estatevar: &mut crate::State) -> Result<crate::Value, crate::Error> {
                let $selfvar = self;
                $eval_hnd
            }

            fn token(&self) -> &crate::Token<'i> {
                &self.token
            }
        }
        #[allow(unused_mut)]
        impl<'i> crate::syntax_tree::traits::SyntaxNodeBuilderExt<'i> for $name<'i> {
            fn build(mut $pairsvar: crate::syntax_tree::pair::InnerPestIterator<'i>, $btokvar: crate::Token<'i>, $bstatevar: &mut crate::State) -> Result<crate::syntax_tree::Node<'i>, crate::Error>
            $build_hnd
        }
        /// Simplify conversion from node to AST node
        impl<'i> From<$name<'i>> for crate::syntax_tree::Node<'i> {
            fn from(node: $name<'i>) -> Self {
                Self::$oname($oname::$name(node))
            }
        }
    };
}

macro_rules! define_ast {
    (
        $name:ident {
            $($iname:ident $(($($param:ident : $param_t:ty),*))? {
                build = ($pairsvar:ident, $btokvar:ident, $bstatevar:ident) $build_hnd:block,
                eval  = ($selfvar:ident, $estatevar:ident) $eval_hnd:block,
                owned = ($oselfvar:ident) $owned_hnd:block
                $(
                        ,docs  = {
                        name: $docs_name:literal,
                        symbols = [$($docs_symbols:literal),*],
                        description: $docs_desc:literal,
                        examples: $docs_examples:literal,
                    }
                )?
            }),+
        }
    ) => {
        #[enum_dispatch::enum_dispatch]
        #[derive(Debug, Clone)]
        pub enum $name<'i> {
            $( $iname($iname<'i>), )+
        }
        impl crate::syntax_tree::traits::IntoOwned for $name<'_> {
            type Owned = $name<'static>;
            fn into_owned(self) -> Self::Owned {
                match self {
                    $(
                        $name::$iname(node) => $name::$iname(node.into_owned()),
                    )+
                }
            }
        }
        impl<'i> crate::syntax_tree::traits::NodeExt<'i> for $name<'i> {
            fn evaluate(&self, state: &mut crate::State) -> Result<polyvalue::Value, crate::Error> {
                match self {
                    $(
                        $name::$iname(node) => node.evaluate(state),
                    )+
                }
            }
            fn token(&self) -> &crate::Token<'i> {
                match self {
                    $(
                        $name::$iname(node) => node.token(),
                    )+
                }
            }
        }

        $(
            define_astnode! {
                outer = $name,
                $iname $({$($param: $param_t),*})?,
                build = ($pairsvar, $btokvar, $bstatevar) $build_hnd,
                eval  = ($selfvar, $estatevar) $eval_hnd,
                owned = ($oselfvar) $owned_hnd
                $(,docs  = {
                    name: $docs_name,
                    symbols = [$($docs_symbols),*],
                    description: $docs_desc,
                    examples: $docs_examples,
                })?
            }
        )+
    };
}

macro_rules! define_handler {
    ($name:ident ($pairsvar:ident, $btokvar:ident, $bstatevar:ident) $build_hnd:block) => {
        pub struct $name;
        #[allow(unused_mut)]
        impl<'i> crate::syntax_tree::traits::SyntaxNodeBuilderExt<'i> for $name {
            fn build(mut $pairsvar: crate::syntax_tree::pair::InnerPestIterator<'i>, $btokvar: crate::Token<'i>, $bstatevar: &mut crate::State) -> Result<crate::syntax_tree::Node<'i>, crate::Error>
            $build_hnd
        }
    };
}

macro_rules! node_type {
    ($base:ident :: $type:ident($ref:ident)) => {
        Node::$base(crate::syntax_tree::nodes::$base::$type($ref))
    };
}

macro_rules! unwrap_next {
    ($pairs:expr, $context:expr) => {
        $pairs.next().unwrap_or_else(|| {
            panic!(
                "Rule {:?} expected a token; Grammar bug - please report this.",
                $context.rule,
            )
        })
    };
}

macro_rules! unwrap_last {
    ($pairs:expr, $context:expr) => {
        $pairs.last_child().unwrap_or_else(|| {
            panic!(
                "Rule {:?} expected a token; Grammar bug - please report this.",
                $context.rule,
            )
        })
    };
}

macro_rules! unwrap_node {
    ($pairs:expr, $state:expr, $context:expr) => {
        unwrap_next!($pairs, $context)
            .into_node($state)
            .with_context(&$context)
    };
}
