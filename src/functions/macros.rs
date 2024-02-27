/// This macro defines a standard function and registers it with the standard library of functions.
/// The standard function is a function that takes multiple arguments and returns a value.
///
/// # Usage
/// ```rust
/// define_stdfunction!(
///     add { a: Number, b: Number },
///     returns = Number,
///     docs = {
///         category: "Math",
///         description: "Addition",
///         ext_description: "Adds two numbers together.",
///         examples: "
///             assert_eq(
///                 add(2, 3),
///                 5
///             )
///         "
///     },
///     handler = |state: &mut State| -> Result<polyvalue::Value, crate::Error> {
///         let a = state.get_variable("a").unwrap().as_a::<Number>()?;
///         let b = state.get_variable("b").unwrap().as_a::<Number>()?;
///         Ok((a + b).into())
///     }
/// );
/// ```
///
/// # Arguments
/// - `$name:ident`: The name of the function.
/// - `$aname:ident : $meta:ident::$atype:ident`: The arguments of the function, where `$aname` is the argument name, `$meta` is how to process the argument (Standard, Optional or Plural), and `$atype` is the argument value type (See [polyvalue::ValueType]).
/// - `returns = $return:ident`: The return value type of the function. See [polyvalue::ValueType].
/// - `docs = { ... }`: The documentation for the function, including the category name, description, extended description, and examples.
/// - `handler = $handler:expr`: The handler function that implements the logic of the function.
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

/// Defines a decorator function and registers it with the standard library of functions.
/// The decorator function is a function that takes a single argument and returns a string.
///
/// # Usage
/// ```rust
/// define_stddecorator!(
///     upper { input: String },
///     docs = {
///         description: "Uppercase",
///         ext_description: "Converts the input string to uppercase.",
///         examples: "
///             assert_eq(
///                 'hello' @upper,
///                 'HELLO'
///             )
///         "
///     },
///     handler = |input: polyvalue::Value| -> Result<String, crate::Error> {
///         Ok(input.as_a::<String>()?.to_uppercase())
///     }
/// );
/// ```
///
/// # Arguments
/// - `$name:ident`: The name of the function.
/// - `$aname:ident : $atype:ident`: The argument of the function, where `$aname` is the argument name and `$atype` is the argument value type (See [polyvalue::ValueType]).
/// - `docs = { ... }`: The documentation for the function, including the category name, description, extended description, and examples.
/// - `handler = $handler:expr`: The handler function that implements the logic of the function.
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
