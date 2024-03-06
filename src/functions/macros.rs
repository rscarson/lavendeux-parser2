/// This macro defines a standard function and registers it with the standard library of functions.
/// The standard function is a function that takes multiple arguments and returns a value.
///
/// # Usage
/// ```rust
/// use lavendeux_parser::{define_stdfunction, polyvalue, Error, functions::ParserFunction, State};
/// define_stdfunction!(
///     add { a: Standard::Numeric, b: Standard::Numeric },
///     returns = Numeric,
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
///     handler = (state, reference) {
///         let a = state.get_variable("a").unwrap().as_a::<i64>()?;
///         let b = state.get_variable("b").unwrap().as_a::<i64>()?;
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
        handler = ($hndstate:ident, $hndref:ident) $handler:block$(,)?
    ) => {
        paste::paste! {
            #[allow(non_camel_case_types)]
            #[derive(Debug, Copy, Clone)]
            pub struct [<_stdlibfn_$name>];
            impl [<_stdlibfn_$name>] {
                const NAME: &'static str = stringify!($name);

                const DOCS: $crate::functions::StaticFunctionDocumentation = $crate::functions::StaticFunctionDocumentation {
                    category: $category,
                    description: Some($description),
                    ext_description: Some(indoc::indoc! { $ext_description }),
                    examples: Some(indoc::indoc! { $examples })
                };
                const ARGUMENTS: &'static [(&'static str, $crate::functions::FunctionArgument)] = &[$(
                    (stringify!($aname), $crate::functions::FunctionArgument {
                        expected_type: polyvalue::ValueType::$atype,
                        meta: $crate::functions::FunctionArgumentType::$meta
                    })
                ),*];

                pub fn new() -> Self {
                    Self
                }
            }

            impl $crate::functions::std_function::ParserFunction for [<_stdlibfn_$name>] {
                fn name(&self) -> &str {
                    Self::NAME
                }

                fn is_readonly(&self) -> bool {
                    true
                }

                fn documentation(&self) -> &dyn $crate::functions::FunctionDocumentation {
                    &Self::DOCS
                }

                fn documentation_mut(&mut self) -> &mut dyn $crate::functions::FunctionDocumentation {
                    unimplemented!()
                }

                fn return_type(&self) -> polyvalue::ValueType {
                    polyvalue::ValueType::$return
                }

                fn expected_arguments(&self) -> Vec<(std::borrow::Cow<'static, str>, $crate::functions::FunctionArgument)> {
                    Self::ARGUMENTS
                        .iter()
                        .copied()
                        .map(|(name, arg)| (std::borrow::Cow::Borrowed(name), arg))
                        .collect()
                }

                fn clone_self(&self) -> Box<dyn $crate::functions::std_function::ParserFunction> {
                    Box::new(Self::new())
                }

                fn call(&self, $hndstate: &mut $crate::State, $hndref: Option<&$crate::Reference>) -> Result<polyvalue::Value, $crate::Error> $handler
            }

            inventory::submit! {
                &[<_stdlibfn_$name>] as &'static dyn $crate::functions::std_function::ParserFunction
            }
        }
    };
}

/// Defines a decorator function and registers it with the standard library of functions.
/// The decorator function is a function that takes a single argument and returns a string.
///
/// # Usage
/// ```rust
/// use lavendeux_parser::{define_stddecorator, polyvalue, Error, functions::ParserFunction};
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
///     handler = (input) {
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
        handler = ($hndval:ident) $handler:block$(,)?
    ) => {
        paste::paste! {
            #[allow(non_camel_case_types)]
            #[derive(Debug, Copy, Clone)]
            pub struct [<_stdlibfn_dec_$name>];
            impl [<_stdlibfn_dec_$name>] {
                const NAME: &'static str = concat!("@", stringify!($name));

                const DOCS: $crate::functions::StaticFunctionDocumentation = $crate::functions::StaticFunctionDocumentation {
                    category: "Decorators",
                    description: Some($description),
                    ext_description: Some(indoc::indoc! { $ext_description }),
                    examples: Some(indoc::indoc! { $examples })
                };
                const ARGUMENTS: &'static [(&'static str, $crate::functions::FunctionArgument)] = &[
                    (stringify!($aname), $crate::functions::FunctionArgument {
                        expected_type: polyvalue::ValueType::$atype,
                        meta: $crate::functions::FunctionArgumentType::Standard
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

                fn documentation(&self) -> & dyn $crate::functions::FunctionDocumentation {
                    &Self::DOCS
                }

                fn documentation_mut(&mut self) -> &mut dyn $crate::functions::FunctionDocumentation {
                    unimplemented!()
                }

                fn return_type(&self) -> $crate::polyvalue::ValueType {
                    polyvalue::ValueType::String
                }

                fn expected_arguments(&self) -> Vec<(std::borrow::Cow<'static, str>, $crate::functions::FunctionArgument)> {
                    Self::ARGUMENTS
                        .iter()
                        .copied()
                        .map(|(name, arg)| (std::borrow::Cow::Borrowed(name), arg))
                        .collect()
                }

                fn clone_self(&self) -> Box<dyn $crate::functions::ParserFunction> {
                    Box::new(Self::new())
                }

                fn call(&self, state: &mut $crate::State, _: Option<&crate::syntax_tree::Reference>) -> Result<polyvalue::Value, $crate::Error> {
                    let $hndval = required_arg!(state::$aname);
                    let value: Result<String, Error> = $handler;
                    Ok(value?.into())
                }
            }

            inventory::submit! {
                &[<_stdlibfn_dec_$name>] as &'static dyn $crate::functions::ParserFunction
            }
        }
    };
}

macro_rules! required_arg {
    ($state:ident :: $name:ident) => {
        match optional_arg!($state::$name) {
            Some(v) => v,
            None => {
                return oops!(Internal {
                    msg: format!("Missing required argument: {}", stringify!($name))
                })
            }
        }
    };
}

macro_rules! optional_arg {
    ($state:ident :: $name:ident) => {
        $state.get_variable(stringify!($name)).cloned()
    };
}
