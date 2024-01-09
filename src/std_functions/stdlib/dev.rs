use crate::{
    get_argument, get_optional_argument, optional_argument, required_argument, static_function,
    std_functions::Function, Error, State,
};
use polyvalue::{
    types::{Array, Int, Str},
    Value, ValueTrait, ValueType,
};
use std::{collections::HashMap, fs::File, io::BufRead};

pub fn register_all(map: &mut HashMap<String, Function>) {
    static_function!(
        registry = map,
        name = "time",
        description = "Returns a unix timestamp for the current system time",
        category = "dev",
        arguments = [],
        returns = ValueType::Float,
        handler = |_: &mut State, _arguments, _token, _| {
            Ok(Value::from(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64(),
            ))
        }
    );

    static_function!(
        registry = map,
        name = "tail",
        description = "Returns the last [lines] lines from a given file",
        category = "dev",
        arguments = [
            required_argument!("file", ValueType::String),
            optional_argument!("lines", ValueType::Int)
        ],
        returns = ValueType::Float,
        handler = |_: &mut State, arguments, _token, _| {
            let n = get_optional_argument!("lines", arguments)
                .unwrap_or(Value::from(1))
                .as_a::<Int>()?
                .inner()
                .clone();
            let file = get_argument!("file", arguments)
                .as_a::<Str>()?
                .inner()
                .clone();
            let file = File::open(&file)?;

            let lines = std::io::BufReader::new(file)
                .lines()
                .map(|f| Ok::<Value, Error>(Value::from(f?)))
                .collect::<Result<Vec<_>, _>>()?;

            // return last n
            Ok(Value::from(Array::from(
                lines
                    .iter()
                    .rev()
                    .take(n as usize)
                    .rev()
                    .cloned()
                    .collect::<Vec<_>>(),
            )))
        }
    );

    #[cfg(feature = "crypto-functions")]
    static_function!(
        registry = map,
        name = "sha256",
        description = "Returns the sha256 hash of a given string",
        category = "dev",
        arguments = [required_argument!("input", ValueType::String)],
        returns = ValueType::String,
        handler = |_: &mut State, arguments, _token, _| {
            use sha2::{Digest, Sha256};
            let input = get_argument!("input", arguments)
                .as_a::<Str>()?
                .inner()
                .clone();

            let mut hasher = Sha256::new();
            hasher.update(input);

            let s = format!("{:X}", hasher.finalize());
            Ok(Value::from(s))
        }
    );

    #[cfg(feature = "crypto-functions")]
    static_function!(
        registry = map,
        name = "sha512",
        description = "Returns the sha512 hash of a given string",
        category = "dev",
        arguments = [required_argument!("input", ValueType::String)],
        returns = ValueType::String,
        handler = |_: &mut State, arguments, _token, _| {
            use sha2::{Digest, Sha512};
            let input = get_argument!("input", arguments)
                .as_a::<Str>()?
                .inner()
                .clone();

            let mut hasher = Sha512::new();
            hasher.update(input);

            let s = format!("{:X}", hasher.finalize());
            Ok(Value::from(s))
        }
    );

    #[cfg(feature = "crypto-functions")]
    static_function!(
        registry = map,
        name = "md5",
        description = "Returns the md5 hash of a given string",
        category = "dev",
        arguments = [required_argument!("input", ValueType::String)],
        returns = ValueType::String,
        handler = |_: &mut State, arguments, _token, _| {
            use md5::{Digest, Md5};
            let input = get_argument!("input", arguments)
                .as_a::<Str>()?
                .inner()
                .clone();

            let mut hasher = Md5::new();
            hasher.update(input);

            let s = format!("{:X}", hasher.finalize());
            Ok(Value::from(s))
        }
    );

    #[cfg(feature = "crypto-functions")]
    static_function!(
        registry = map,
        name = "choose",
        description = "Returns a random element from a given array",
        category = "dev",
        arguments = [required_argument!("array", ValueType::Array)],
        returns = ValueType::Any,
        handler = |_: &mut State, arguments, _token, _| {
            use rand::seq::SliceRandom;
            let input = get_argument!("array", arguments)
                .as_a::<Array>()?
                .inner()
                .clone();
            let random = input.choose(&mut rand::thread_rng()).unwrap();
            Ok(random.clone())
        }
    );

    #[cfg(feature = "crypto-functions")]
    static_function!(
        registry = map,
        name = "rand",
        description = "With no arguments, return a float from 0 to 1. Otherwise return an integer from 0 to m, or m to n",
        category = "dev",
        arguments = [optional_argument!("m", ValueType::Int), optional_argument!("n", ValueType::Int)],
        returns = ValueType::Any,
        handler = |_: &mut State, arguments, token, _| {
            use rand::Rng;
            let m = get_optional_argument!("m", arguments).and_then(|v| Some(*v.as_a::<Int>().unwrap().inner()));
            let n = get_optional_argument!("n", arguments).and_then(|v| Some(*v.as_a::<Int>().unwrap().inner()));

            match (m, n) {
                (None, None) => Ok(Value::from(rand::random::<f64>())),
                (Some(m), None)|(None, Some(m)) => {
                    if m == 0 {
                        return Err(Error::Range {
                            input: "0".to_string(),
                            token: token.clone(),
                        });
                    }
                    Ok(Value::from(rand::thread_rng().gen_range(0..(m+1))))
                },
                (Some(m), Some(n)) => {
                    if m >= n {
                        return Err(Error::Range {
                            input: format!("{}..{}", m, n),
                            token: token.clone(),
                        });
                    }
                    Ok(Value::from(rand::thread_rng().gen_range(m..(n+1))))
                },
            }
        }
    );
}
