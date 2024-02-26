use crate::{error::WrapExternalError, oops, Error, Token};
use polyvalue::{
    types::{Array, Object},
    Value, ValueTrait, ValueType,
};

/// Macro to generate a static decorator
/// # Arguments
/// * `registry` - The registry to register the decorator to
/// * `name` - The name of the decorator
/// * `description` - The description of the decorator
/// * `expected_type` - The expected type of the decorator
/// * `handler` - The handler of the decorator
#[macro_export]
macro_rules! static_decorator {
    (registry = $map:expr, name = $name:literal, description = $description:literal, expected_type = $expected_type:expr, handler = $handler:expr) => {
        static_function!(
            registry = $map,
            name = concat!("@", $name),
            description = $description,
            category = "decorators",
            arguments = [required_argument!("input", ValueType::Any)],
            returns = ValueType::String,
            handler = |_state: &mut State, arguments, token, _| {
                let input = get_argument!("input", arguments);
                $crate::std_functions::decorator_function::recursively_apply_decorator(
                    input,
                    token,
                    $expected_type,
                    $name,
                    $handler,
                )
            }
        );
    };
}

pub fn recursively_apply_decorator(
    input: Value,
    token: &Token,
    required_type: ValueType,
    name: &str,
    handler: &dyn Fn(Value, &Token) -> Result<String, Error>,
) -> Result<Value, Error> {
    match input.own_type() {
        ValueType::Array => {
            let mut input = input.as_a::<Array>().with_context(token)?;
            for e in input.inner_mut() {
                *e = recursively_apply_decorator(e.clone(), token, required_type, name, handler)?;
            }
            Ok(Value::from(input.to_string()))
        }

        ValueType::Object => {
            let mut input = input.as_a::<Object>().with_context(token)?;
            for e in input.inner_mut().values_mut() {
                *e = recursively_apply_decorator(e.clone(), token, required_type, name, handler)?;
            }
            Ok(Value::from(input.to_string()))
        }

        _ => {
            if input.is_a(required_type) {
                Ok(Value::from(handler(input, token)?))
            } else {
                oops!(
                    FunctionArgumentType {
                        arg: 1,
                        expected_type: required_type,
                        signature: format!("<{required_type}> @{name}")
                    },
                    token.clone()
                )
            }
        }
    }
}
