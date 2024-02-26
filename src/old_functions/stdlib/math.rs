use crate::{
    error::{ErrorDetails, WrapExternalError},
    get_argument, oops, required_argument, static_function,
    std_functions::Function,
    State,
};
use polyvalue::{
    fpdec::Round,
    types::{Array, CurrencyInner, Float, I64},
    InnerValue, Value, ValueTrait, ValueType,
};
use std::collections::HashMap;

pub fn register_all(map: &mut HashMap<String, Function>) {
    static_function!(
        registry = map,
        name = "min",
        description = "Returns the smallest value in a set",
        category = "math",
        arguments = [required_argument!("input", ValueType::Collection)],
        returns = ValueType::Float,
        handler = |_: &mut State, arguments, token, _| {
            let input = get_argument!("input", arguments)
                .as_a::<Array>()
                .with_context(token)?
                .inner()
                .clone();
            let min = input
                .iter()
                .min()
                .ok_or(ErrorDetails::ArrayEmpty)
                .with_context(token)?
                .clone();
            Ok(min)
        }
    );

    static_function!(
        registry = map,
        name = "max",
        description = "Returns the largest value in a set",
        category = "math",
        arguments = [required_argument!("input", ValueType::Collection)],
        returns = ValueType::Float,
        handler = |_: &mut State, arguments, token, _| {
            let input = get_argument!("input", arguments)
                .as_a::<Array>()
                .with_context(token)?
                .inner()
                .clone();
            let max = input
                .iter()
                .max()
                .ok_or(ErrorDetails::ArrayEmpty)
                .with_context(token)?
                .clone();
            Ok(max)
        }
    );

    static_function!(
        registry = map,
        name = "ceil",
        description = "Rounds a number up to the nearest integer",
        category = "math",
        arguments = [required_argument!("n", ValueType::Numeric)],
        returns = ValueType::Float,
        handler = |_: &mut State, arguments, token, _| {
            let n = *get_argument!("n", arguments)
                .as_a::<Float>()
                .with_context(token)?
                .inner();
            Ok(Value::from(n.ceil()))
        }
    );

    static_function!(
        registry = map,
        name = "floor",
        description = "Rounds a number down to the nearest integer",
        category = "math",
        arguments = [required_argument!("n", ValueType::Numeric)],
        returns = ValueType::Float,
        handler = |_: &mut State, arguments, token, _| {
            let n = *get_argument!("n", arguments)
                .as_a::<Float>()
                .with_context(token)?
                .inner();
            Ok(Value::from(n.floor()))
        }
    );

    static_function!(
        registry = map,
        name = "abs",
        description = "Returns the absolute value of a number",
        category = "math",
        arguments = [required_argument!("n", ValueType::Numeric)],
        returns = ValueType::Float,
        handler = |_: &mut State, arguments, token, _| {
            match &get_argument!("n", arguments).inner() {
                InnerValue::Float(n) => Ok(Value::from(n.inner().abs())),
                InnerValue::I64(n) => Ok(Value::from(n.inner().abs())),
                InnerValue::Fixed(n) => Ok(Value::fixed(n.inner().abs())),
                InnerValue::Currency(n) => {
                    let symbol = n.symbol().clone();
                    let precision = n.precision();
                    let value = n.inner().value().inner().abs();
                    Ok(CurrencyInner::new(symbol, precision, value.into()).into())
                }
                _ => oops!(
                    Internal {
                        msg: "Invalid argument type".to_string()
                    },
                    token.clone()
                ),
            }
        }
    );

    static_function!(
        registry = map,
        name = "round",
        description = "Rounds a number to a given precision",
        category = "math",
        arguments = [
            required_argument!("n", ValueType::Numeric),
            required_argument!("precision", ValueType::Int)
        ],
        returns = ValueType::Float,
        handler = |_: &mut State, arguments, token, _| {
            let n = get_argument!("n", arguments);
            if n.is_a(ValueType::Int) {
                return Ok(n);
            }

            let precision = *get_argument!("precision", arguments)
                .as_a::<I64>()
                .with_context(token)?
                .inner();
            match &get_argument!("n", arguments).inner() {
                // Round floats to n decimal places
                InnerValue::Float(n) => {
                    let n = n.inner();
                    let n = n * 10.0_f64.powi(precision as i32);
                    let n = n.round();
                    let n = n / 10.0_f64.powi(precision as i32);
                    Ok(Value::from(n))
                }

                InnerValue::Fixed(n) => Ok(Value::from(n.inner().clone().round(precision as i8))),
                InnerValue::Currency(n) => {
                    let symbol = n.symbol().clone();
                    let precision = n.precision();
                    let value = n.inner().value().inner().clone().round(precision);
                    Ok(CurrencyInner::new(symbol, precision, value.into()).into())
                }
                _ => oops!(
                    Internal {
                        msg: "Invalid argument type".to_string()
                    },
                    token.clone()
                ),
            }
        }
    );

    // LOG2
    static_function!(
        registry = map,
        name = "log2",
        description = "Returns the base 2 logarithm of a number",
        category = "math",
        arguments = [required_argument!("n", ValueType::Numeric)],
        returns = ValueType::Float,
        handler = |_: &mut State, arguments, token, _| {
            let n = *get_argument!("n", arguments)
                .as_a::<Float>()
                .with_context(token)?
                .inner();
            Ok(Value::from(n.log2()))
        }
    );

    // iLOG2
    static_function!(
        registry = map,
        name = "ilog2",
        description = "Returns the base 2 logarithm of an integer",
        category = "math",
        arguments = [required_argument!("n", ValueType::Int)],
        returns = ValueType::Float,
        handler = |_: &mut State, arguments, token, _| {
            let n = *get_argument!("n", arguments)
                .as_a::<I64>()
                .with_context(token)?
                .inner();
            Ok(Value::from(
                n.ilog2() as <polyvalue::types::I64 as ValueTrait>::Inner
            ))
        }
    );

    // LOG10
    static_function!(
        registry = map,
        name = "log10",
        description = "Returns the base 10 logarithm of a number",
        category = "math",
        arguments = [required_argument!("n", ValueType::Numeric)],
        returns = ValueType::Float,
        handler = |_: &mut State, arguments, token, _| {
            let n = *get_argument!("n", arguments)
                .as_a::<Float>()
                .with_context(token)?
                .inner();
            Ok(Value::from(n.log10()))
        }
    );

    // LN
    static_function!(
        registry = map,
        name = "ln",
        description = "Returns the natural logarithm of a number",
        category = "math",
        arguments = [required_argument!("n", ValueType::Numeric)],
        returns = ValueType::Float,
        handler = |_: &mut State, arguments, token, _| {
            let n = *get_argument!("n", arguments)
                .as_a::<Float>()
                .with_context(token)?
                .inner();
            Ok(Value::from(n.ln()))
        }
    );

    // LOG
    static_function!(
        registry = map,
        name = "log",
        description = "Returns the logarithm of a number with a given base",
        category = "math",
        arguments = [
            required_argument!("n", ValueType::Numeric),
            required_argument!("base", ValueType::Numeric)
        ],
        returns = ValueType::Float,
        handler = |_: &mut State, arguments, token, _| {
            let n = *get_argument!("n", arguments)
                .as_a::<Float>()
                .with_context(token)?
                .inner();
            let base = *get_argument!("base", arguments)
                .as_a::<Float>()
                .with_context(token)?
                .inner();
            Ok(Value::from(n.log(base)))
        }
    );

    // SQRT
    static_function!(
        registry = map,
        name = "sqrt",
        description = "Returns the square root of a number",
        category = "math",
        arguments = [required_argument!("n", ValueType::Numeric)],
        returns = ValueType::Float,
        handler = |_: &mut State, arguments, token, _| {
            let n = *get_argument!("n", arguments)
                .as_a::<Float>()
                .with_context(token)?
                .inner();
            Ok(Value::from(n.sqrt()))
        }
    );

    // ROOT
    static_function!(
        registry = map,
        name = "root",
        description = "Returns the nth root of a number",
        category = "math",
        arguments = [
            required_argument!("n", ValueType::Numeric),
            required_argument!("root", ValueType::Numeric)
        ],
        returns = ValueType::Float,
        handler = |_: &mut State, arguments, token, _| {
            let n = *get_argument!("n", arguments)
                .as_a::<Float>()
                .with_context(token)?
                .inner();
            let root = *get_argument!("root", arguments)
                .as_a::<Float>()
                .with_context(token)?
                .inner();
            Ok(Value::from(n.powf(1.0 / root)))
        }
    );
}
