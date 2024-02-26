#[macro_export]
macro_rules! define_stdfunction {
    (
        $name:ident { $($aname:ident : $meta:ident::$atype:ident),* },
        returns = $return:ident,
        docs = {
            category: $category:literal,
            description: $description:expr,
            ext_description: $ext_description:literal,
            examples: $examples:literal$(,)?
        },
        handler = $handler:expr$(,)?
    ) => {
        paste::paste! {
            #[allow(non_camel_case_types)]
            #[derive(Debug, Copy, Clone)]
            pub struct [<_stdlibfn_$name>];
            impl [<_stdlibfn_$name>] {
                const NAME: &'static str = stringify!($name);

                const DOCS: crate::functions::FunctionDocumentation = crate::functions::FunctionDocumentation {
                    category: $category,
                    description: Some($description),
                    ext_description: Some(indoc::indoc! { $ext_description }),
                    examples: Some(indoc::indoc! { $examples })
                };
                const ARGUMENTS: &'static [(&'static str, crate::functions::FunctionArgument)] = &[$(
                    (stringify!($aname), crate::functions::FunctionArgument {
                        expected_type: polyvalue::ValueType::$atype,
                        meta: crate::functions::FunctionArgumentType::$meta
                    })
                ),*];

                pub fn new() -> Self {
                    Self
                }
            }

            impl ParserFunction for [<_stdlibfn_$name>] {
                fn name(&self) -> &str {
                    Self::NAME
                }

                fn is_readonly(&self) -> bool {
                    true
                }

                fn documentation(&self) -> &crate::functions::FunctionDocumentation {
                    &Self::DOCS
                }

                fn return_type(&self) -> polyvalue::ValueType {
                    polyvalue::ValueType::$return
                }

                fn expected_arguments(&self) -> Vec<(&str, crate::functions::FunctionArgument)> {
                    Self::ARGUMENTS.to_vec()
                }

                fn clone_self(&self) -> Box<dyn ParserFunction> {
                    Box::new(Self::new())
                }

                fn call(&self, state: &mut State) -> Result<polyvalue::Value, crate::Error> {
                    $handler(state)
                }
            }

            inventory::submit! {
                &[<_stdlibfn_$name>] as &'static dyn ParserFunction
            }
        }
    };
}

#[macro_export]
macro_rules! define_stddecorator {
    (
        $name:ident { $aname:ident : $atype:ident },
        docs = {
            description: $description:expr,
            ext_description: $ext_description:literal,
            examples: $examples:literal$(,)?
        },
        handler = $handler:expr$(,)?
    ) => {
        paste::paste! {
            #[allow(non_camel_case_types)]
            #[derive(Debug, Copy, Clone)]
            pub struct [<_stdlibfn_dec_$name>];
            impl [<_stdlibfn_dec_$name>] {
                const NAME: &'static str = concat!("@", stringify!($name));

                const DOCS: crate::functions::FunctionDocumentation = crate::functions::FunctionDocumentation {
                    category: "Decorators",
                    description: Some($description),
                    ext_description: Some(indoc::indoc! { $ext_description }),
                    examples: Some(indoc::indoc! { $examples })
                };
                const ARGUMENTS: &'static [(&'static str, crate::functions::FunctionArgument)] = &[
                    (stringify!($aname), crate::functions::FunctionArgument {
                        expected_type: polyvalue::ValueType::$atype,
                        meta: crate::functions::FunctionArgumentType::Standard
                    })
                ];

                pub fn new() -> Self {
                    Self
                }
            }

            impl ParserFunction for [<_stdlibfn_dec_$name>] {
                fn name(&self) -> &str {
                    Self::NAME
                }

                fn is_readonly(&self) -> bool {
                    true
                }

                fn documentation(&self) -> &crate::functions::FunctionDocumentation {
                    &Self::DOCS
                }

                fn return_type(&self) -> polyvalue::ValueType {
                    polyvalue::ValueType::String
                }

                fn expected_arguments(&self) -> Vec<(&str, crate::functions::FunctionArgument)> {
                    Self::ARGUMENTS.to_vec()
                }

                fn clone_self(&self) -> Box<dyn ParserFunction> {
                    Box::new(Self::new())
                }

                fn call(&self, state: &mut State) -> Result<polyvalue::Value, crate::Error> {
                    let input = state.get_variable(stringify!($aname)).unwrap();
                    let value: String = $handler(input)?;
                    Ok(value.into())
                }
            }

            inventory::submit! {
                &[<_stdlibfn_dec_$name>] as &'static dyn ParserFunction
            }
        }
    };
}
