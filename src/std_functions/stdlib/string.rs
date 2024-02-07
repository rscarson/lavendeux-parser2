use crate::{
    error::WrapError, get_argument, required_argument, static_function, std_functions::Function,
    Error, State,
};
use polyvalue::{
    types::{Array, Str, I64},
    Value, ValueTrait, ValueType,
};
use std::collections::HashMap;

pub fn register_all(map: &mut HashMap<String, Function>) {
    static_function!(
        registry = map,
        name = "ord",
        description = "Returns the Unicode code point of the first character in a string",
        category = "string",
        arguments = [required_argument!("input", ValueType::String)],
        returns = ValueType::Int,
        handler = |_: &mut State, arguments, token, _| {
            let input = get_argument!("input", arguments)
                .as_a::<Str>()
                .to_error(token)?;
            if input.inner().is_empty() {
                return Err(Error::ValueFormat {
                    expected_format: "non-empty string".to_string(),
                    token: token.clone(),
                });
            }
            Ok(Value::from(input.inner().chars().next().unwrap() as u32))
        }
    );

    static_function!(
        registry = map,
        name = "chr",
        description =
            "Returns a string containing the character represented by a Unicode code point",
        category = "string",
        arguments = [required_argument!("input", ValueType::Int)],
        returns = ValueType::String,
        handler = |_: &mut State, arguments, token, _| {
            let input = *get_argument!("input", arguments)
                .as_a::<I64>()
                .to_error(token)?
                .inner();
            if let Some(c) = std::char::from_u32(input as u32) {
                return Ok(Value::from(c.to_string()));
            } else {
                return Err(Error::ValueFormat {
                    expected_format: "valid Unicode code point".to_string(),
                    token: token.clone(),
                });
            }
        }
    );

    static_function!(
        registry = map,
        name = "uppercase",
        description = "Returns an uppercase version of a string",
        category = "string",
        arguments = [required_argument!("input", ValueType::String)],
        returns = ValueType::String,
        handler = |_: &mut State, arguments, token, _| {
            let input = get_argument!("input", arguments)
                .as_a::<Str>()
                .to_error(token)?
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
        handler = |_: &mut State, arguments, token, _| {
            let input = get_argument!("input", arguments)
                .as_a::<Str>()
                .to_error(token)?
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
        handler = |_: &mut State, arguments, token, _| {
            let input = get_argument!("input", arguments)
                .as_a::<Str>()
                .to_error(token)?
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
        handler = |_: &mut State, arguments, token, _| {
            let input = get_argument!("input", arguments)
                .as_a::<Str>()
                .to_error(token)?
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
        handler = |_: &mut State, arguments, token, _| {
            let input = get_argument!("input", arguments)
                .as_a::<Str>()
                .to_error(token)?
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
        handler = |_: &mut State, arguments, token, _| {
            let input = get_argument!("input", arguments)
                .as_a::<Str>()
                .to_error(token)?
                .inner()
                .clone();
            let old = get_argument!("old", arguments)
                .as_a::<Str>()
                .to_error(token)?
                .inner()
                .clone();
            let new = get_argument!("new", arguments)
                .as_a::<Str>()
                .to_error(token)?
                .inner()
                .clone();
            Ok(Value::from(input.replace(&old, &new)))
        }
    );

    static_function!(
        registry = map,
        name = "repeat",
        description = "Repeats a string a specified number of times",
        category = "string",
        arguments = [
            required_argument!("input", ValueType::String),
            required_argument!("times", ValueType::Int)
        ],
        returns = ValueType::String,
        handler = |_: &mut State, arguments, token, _| {
            let input = get_argument!("input", arguments)
                .as_a::<Str>()
                .to_error(token)?
                .inner()
                .clone();
            let times = *get_argument!("times", arguments)
                .as_a::<I64>()
                .to_error(token)?
                .inner();
            if times < 0 {
                return Err(Error::ValueFormat {
                    expected_format: "non-negative integer".to_string(),
                    token: token.clone(),
                });
            }
            Ok(Value::from(input.repeat(times as usize)))
        }
    );

    static_function!(
        registry = map,
        name = "concat",
        description = "Treats the argument as an array of strings and concatenates them",
        category = "string",
        arguments = [required_argument!("input", ValueType::Array)],
        returns = ValueType::String,
        handler = |_state: &mut State, arguments, token, _| {
            let input = get_argument!("input", arguments)
                .as_a::<Array>()
                .to_error(token)?;
            let mut result = String::new();
            for value in input.inner() {
                let s = value.as_a::<Str>().to_error(token)?.inner().to_string();
                result.push_str(&s);
            }
            Ok(Value::from(result))
        }
    );

    static_function!(
        registry = map,
        name = "prettyjson",
        description = "Returns a pretty-printed JSON string",
        category = "string",
        arguments = [required_argument!("input", ValueType::Any)],
        returns = ValueType::String,
        handler = |_: &mut State, arguments, token, _| {
            let input = get_argument!("input", arguments)
                .as_a::<Str>()
                .to_error(token)?
                .inner()
                .clone();
            Ok(Value::from(
                serde_json::to_string_pretty(&input).to_error(token)?,
            ))
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
        handler = |_: &mut State, arguments, token, _| {
            let input = get_argument!("input", arguments)
                .as_a::<Str>()
                .to_error(token)?
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
        handler = |_: &mut State, arguments, token, _| {
            let input = get_argument!("input", arguments)
                .as_a::<Str>()
                .to_error(token)?
                .inner()
                .clone();
            Ok(Value::from(
                urlencoding::decode(&input).to_error(token)?.into_owned(),
            ))
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
        handler = |_: &mut State, arguments, token, _| {
            use base64::{engine::general_purpose, Engine as _};
            let input = get_argument!("input", arguments)
                .as_a::<Str>()
                .to_error(token)?
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
                .as_a::<Str>()
                .to_error(token)?
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
