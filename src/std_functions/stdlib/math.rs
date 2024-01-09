use crate::{
    get_argument, required_argument, static_function, std_functions::Function, Error, State,
};
use polyvalue::{
    fpdec::Round,
    types::{Array, CurrencyInner, Float, Int},
    Value, ValueTrait, ValueType,
};
use std::collections::HashMap;

pub fn register_all(map: &mut HashMap<String, Function>) {
    static_function!(
        registry = map,
        name = "min",
        description = "Returns the smallest value in a set",
        category = "math",
        arguments = [required_argument!("input", ValueType::Compound)],
        returns = ValueType::Float,
        handler = |_: &mut State, arguments, token, _| {
            let input = get_argument!("input", arguments)
                .as_a::<Array>()?
                .inner()
                .clone();
            let min = input
                .iter()
                .min()
                .ok_or(Error::ArrayEmpty {
                    token: token.clone(),
                })?
                .clone();
            Ok(min)
        }
    );

    static_function!(
        registry = map,
        name = "max",
        description = "Returns the largest value in a set",
        category = "math",
        arguments = [required_argument!("input", ValueType::Compound)],
        returns = ValueType::Float,
        handler = |_: &mut State, arguments, token, _| {
            let input = get_argument!("input", arguments)
                .as_a::<Array>()?
                .inner()
                .clone();
            let max = input
                .iter()
                .max()
                .ok_or(Error::ArrayEmpty {
                    token: token.clone(),
                })?
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
        handler = |_: &mut State, arguments, _token, _| {
            let n = *get_argument!("n", arguments).as_a::<Float>()?.inner();
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
        handler = |_: &mut State, arguments, _token, _| {
            let n = *get_argument!("n", arguments).as_a::<Float>()?.inner();
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
        handler = |_: &mut State, arguments, _token, _| {
            match &get_argument!("n", arguments) {
                Value::Float(n) => Ok(Value::from(n.inner().abs())),
                Value::Int(n) => Ok(Value::from(n.inner().abs())),
                Value::Fixed(n) => Ok(Value::from(n.inner().abs())),
                Value::Currency(n) => {
                    let symbol = n.symbol().clone();
                    let precision = n.precision();
                    let value = n.inner().value().inner().abs();
                    Ok(CurrencyInner::new(symbol, precision, value.into()).into())
                }
                _ => Err(Error::Internal("Invalid argument type".to_string())),
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
        handler = |_: &mut State, arguments, _token, _| {
            let precision = *get_argument!("precision", arguments).as_a::<Int>()?.inner();
            match &get_argument!("n", arguments) {
                Value::Int(n) => Ok(Value::from(n)),

                // Round floats to n decimal places
                Value::Float(n) => {
                    let n = n.inner();
                    let n = n * 10.0_f64.powi(precision as i32);
                    let n = n.round();
                    let n = n / 10.0_f64.powi(precision as i32);
                    Ok(Value::from(n))
                }

                Value::Fixed(n) => Ok(Value::from(n.inner().round(precision as i8))),
                Value::Currency(n) => {
                    let symbol = n.symbol().clone();
                    let precision = n.precision();
                    let value = n.inner().value().inner().round(precision as i8);
                    Ok(CurrencyInner::new(symbol, precision, value.into()).into())
                }
                _ => Err(Error::Internal("Invalid argument type".to_string())),
            }
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
        handler = |_: &mut State, arguments, _token, _| {
            let n = *get_argument!("n", arguments).as_a::<Float>()?.inner();
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
        handler = |_: &mut State, arguments, _token, _| {
            let n = *get_argument!("n", arguments).as_a::<Float>()?.inner();
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
        handler = |_: &mut State, arguments, _token, _| {
            let n = *get_argument!("n", arguments).as_a::<Float>()?.inner();
            let base = *get_argument!("base", arguments).as_a::<Float>()?.inner();
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
        handler = |_: &mut State, arguments, _token, _| {
            let n = *get_argument!("n", arguments).as_a::<Float>()?.inner();
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
        handler = |_: &mut State, arguments, _token, _| {
            let n = *get_argument!("n", arguments).as_a::<Float>()?.inner();
            let root = *get_argument!("root", arguments).as_a::<Float>()?.inner();
            Ok(Value::from(n.powf(1.0 / root)))
        }
    );
}
