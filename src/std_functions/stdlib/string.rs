use crate::{
    get_argument, required_argument, static_function, std_functions::Function, Error, State,
};
use polyvalue::{types::Str, Value, ValueTrait, ValueType};
use std::collections::HashMap;

pub fn register_all(map: &mut HashMap<String, Function>) {
    static_function!(
        registry = map,
        name = "uppercase",
        description = "Returns an uppercase version of a string",
        category = "string",
        arguments = [required_argument!("input", ValueType::String)],
        returns = ValueType::String,
        handler = |_: &mut State, arguments, _token, _| {
            let input = get_argument!("input", arguments)
                .as_a::<Str>()?
                .inner()
                .clone();
            Ok(Value::from(input.to_uppercase()))
        }
    );

    static_function!(
        registry = map,
        name = "lowercase",
        description = "Returns a lowercase version of a string",
        category = "string",
        arguments = [required_argument!("input", ValueType::String)],
        returns = ValueType::String,
        handler = |_: &mut State, arguments, _token, _| {
            let input = get_argument!("input", arguments)
                .as_a::<Str>()?
                .inner()
                .clone();
            Ok(Value::from(input.to_lowercase()))
        }
    );

    static_function!(
        registry = map,
        name = "trim",
        description = "Removes whitespace from the beginning and end of a string",
        category = "string",
        arguments = [required_argument!("input", ValueType::String)],
        returns = ValueType::String,
        handler = |_: &mut State, arguments, _token, _| {
            let input = get_argument!("input", arguments)
                .as_a::<Str>()?
                .inner()
                .clone();
            Ok(Value::from(input.trim()))
        }
    );

    static_function!(
        registry = map,
        name = "trim_start",
        description = "Removes whitespace from the beginning of a string",
        category = "string",
        arguments = [required_argument!("input", ValueType::String)],
        returns = ValueType::String,
        handler = |_: &mut State, arguments, _token, _| {
            let input = get_argument!("input", arguments)
                .as_a::<Str>()?
                .inner()
                .clone();
            Ok(Value::from(input.trim_start()))
        }
    );

    static_function!(
        registry = map,
        name = "trim_end",
        description = "Removes whitespace from the end of a string",
        category = "string",
        arguments = [required_argument!("input", ValueType::String)],
        returns = ValueType::String,
        handler = |_: &mut State, arguments, _token, _| {
            let input = get_argument!("input", arguments)
                .as_a::<Str>()?
                .inner()
                .clone();
            Ok(Value::from(input.trim_end()))
        }
    );

    static_function!(
        registry = map,
        name = "replace",
        description = "Replaces all instances of a substring with another substring",
        category = "string",
        arguments = [
            required_argument!("input", ValueType::String),
            required_argument!("old", ValueType::String),
            required_argument!("new", ValueType::String)
        ],
        returns = ValueType::String,
        handler = |_: &mut State, arguments, _token, _| {
            let input = get_argument!("input", arguments)
                .as_a::<Str>()?
                .inner()
                .clone();
            let old = get_argument!("old", arguments)
                .as_a::<Str>()?
                .inner()
                .clone();
            let new = get_argument!("new", arguments)
                .as_a::<Str>()?
                .inner()
                .clone();
            Ok(Value::from(input.replace(&old, &new)))
        }
    );

    static_function!(
        registry = map,
        name = "prettyjson",
        description = "Returns a pretty-printed JSON string",
        category = "string",
        arguments = [required_argument!("input", ValueType::Any)],
        returns = ValueType::String,
        handler = |_: &mut State, arguments, _token, _| {
            let input = get_argument!("input", arguments)
                .as_a::<Str>()?
                .inner()
                .clone();
            Ok(Value::from(serde_json::to_string_pretty(&input)?))
        }
    );

    #[cfg(feature = "encoding-functions")]
    static_function!(
        registry = map,
        name = "urlencode",
        description = "Returns a URL-encoded string",
        category = "string",
        arguments = [required_argument!("input", ValueType::String)],
        returns = ValueType::String,
        handler = |_: &mut State, arguments, _token, _| {
            let input = get_argument!("input", arguments)
                .as_a::<Str>()?
                .inner()
                .clone();
            Ok(Value::from(urlencoding::encode(&input).into_owned()))
        }
    );

    #[cfg(feature = "encoding-functions")]
    static_function!(
        registry = map,
        name = "urldecode",
        description = "Returns a URL-decoded string",
        category = "string",
        arguments = [required_argument!("input", ValueType::String)],
        returns = ValueType::String,
        handler = |_: &mut State, arguments, _token, _| {
            let input = get_argument!("input", arguments)
                .as_a::<Str>()?
                .inner()
                .clone();
            Ok(Value::from(urlencoding::decode(&input)?.into_owned()))
        }
    );

    #[cfg(feature = "encoding-functions")]
    static_function!(
        registry = map,
        name = "atob",
        description = "Returns a base64-encoded string",
        category = "string",
        arguments = [required_argument!("input", ValueType::String)],
        returns = ValueType::String,
        handler = |_: &mut State, arguments, _token, _| {
            use base64::{engine::general_purpose, Engine as _};
            let input = get_argument!("input", arguments)
                .as_a::<Str>()?
                .inner()
                .clone();
            let mut buf = String::new();
            general_purpose::STANDARD.encode_string(&input, &mut buf);
            Ok(Value::from(buf))
        }
    );

    #[cfg(feature = "encoding-functions")]
    static_function!(
        registry = map,
        name = "btoa",
        description = "Returns a base64-decoded string",
        category = "string",
        arguments = [required_argument!("input", ValueType::String)],
        returns = ValueType::String,
        handler = |_: &mut State, arguments, token, _| {
            use base64::{engine::general_purpose, Engine as _};
            let input = get_argument!("input", arguments)
                .as_a::<Str>()?
                .inner()
                .clone();
            if let Ok(bytes) = general_purpose::STANDARD.decode(input) {
                if let Ok(s) = std::str::from_utf8(&bytes) {
                    return Ok(Value::from(s));
                }
            }

            Err(Error::ValueFormat {
                expected_format: "base64".to_string(),
                token: token.clone(),
            })
        }
    );
}
