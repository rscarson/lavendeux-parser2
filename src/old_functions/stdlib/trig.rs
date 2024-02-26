use crate::{
    error::WrapExternalError, get_argument, required_argument, static_function,
    std_functions::Function, State,
};
use polyvalue::{types::Float, Value, ValueTrait, ValueType};
use std::collections::HashMap;

macro_rules! trig_function {
    ($registry:ident, $name:ident) => {
        static_function!(
            registry = $registry,
            name = stringify!($name),
            description = concat!("Calculate the ", stringify!($name), " of n"),
            category = "trig",
            arguments = [required_argument!("n", ValueType::Numeric)],
            returns = ValueType::Float,
            handler = |_: &mut State, arguments, token, _| {
                let n = *get_argument!("n", arguments)
                    .as_a::<Float>()
                    .with_context(token)?
                    .inner();
                Ok(Value::from(n.$name()))
            }
        );
    };
}

pub fn register_all(map: &mut HashMap<String, Function>) {
    // to_degrees converts radians to degrees
    static_function!(
        registry = map,
        name = "to_degrees",
        description = "Converts radians to degrees",
        category = "trig",
        arguments = [required_argument!("radians", ValueType::Numeric)],
        returns = ValueType::Float,
        handler = |_: &mut State, arguments, token, _| {
            let radians = *get_argument!("radians", arguments)
                .as_a::<Float>()
                .with_context(token)?
                .inner();
            Ok(Value::from(radians * 180.0 / std::f64::consts::PI))
        }
    );

    // to_radians converts degrees to radians
    static_function!(
        registry = map,
        name = "to_radians",
        description = "Converts degrees to radians",
        category = "trig",
        arguments = [required_argument!("degrees", ValueType::Numeric)],
        returns = ValueType::Float,
        handler = |_: &mut State, arguments, token, _| {
            let degrees = *get_argument!("degrees", arguments)
                .as_a::<Float>()
                .with_context(token)?
                .inner();
            Ok(Value::from(degrees * std::f64::consts::PI / 180.0))
        }
    );

    trig_function!(map, sin);
    trig_function!(map, asin);
    trig_function!(map, sinh);
    trig_function!(map, asinh);

    trig_function!(map, cos);
    trig_function!(map, acos);
    trig_function!(map, cosh);
    trig_function!(map, acosh);

    trig_function!(map, tan);
    trig_function!(map, atan);
    trig_function!(map, tanh);
    trig_function!(map, atanh);
}
