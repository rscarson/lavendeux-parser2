use crate::{define_stdfunction, functions::std_function::ParserFunction, State};
use polyvalue::Value;

macro_rules! define_trigfunction {
    ($name:ident, examples = $examples:literal) => {
        define_stdfunction!(
            $name {
                n: Standard::Numeric
            },
            returns = Float,
            docs = {
                category: "Trigonometry",
                description: concat!("Calculate the ", stringify!($name), " of n"),
                ext_description: "
                    Returns a result for the angle n (in radians).
                    You can use the `to_degrees` and `to_radians` functions to convert between degrees and radians.
                ",
                examples: $examples,
            },
            handler = |state: &mut State| {
                let n = state.get_variable("n").unwrap().as_a::<f64>()?;
                Ok(Value::from(n.$name()))
            }
        );
    };
}

define_trigfunction!(sin, examples = "assert_eq( 0.0, sin(0) )");
define_trigfunction!(asin, examples = "assert_eq( 0.0, asin(0) )");
define_trigfunction!(sinh, examples = "assert_eq( 0.0, sinh(0) )");
define_trigfunction!(asinh, examples = "assert_eq( 0.0, asinh(0) )");

define_trigfunction!(cos, examples = "assert_eq( 1.0, cos(0) )");
define_trigfunction!(acos, examples = "assert_eq( 0.0, acos(1) )");
define_trigfunction!(cosh, examples = "assert_eq( 1.0, cosh(0) )");
define_trigfunction!(acosh, examples = "assert_eq( 0.0, acosh(1) )");

define_trigfunction!(tan, examples = "assert_eq( 0.0, tan(0) )");
define_trigfunction!(atan, examples = "assert_eq( 0.0, atan(0) )");
define_trigfunction!(tanh, examples = "assert_eq( 0.0, tanh(0) )");
define_trigfunction!(atanh, examples = "assert_eq( 0.0, atanh(0) )");
