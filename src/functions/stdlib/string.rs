use crate::define_stdfunction;
use polyvalue::{Value, ValueType};

/**********************************************
 *
 * Character functions
 *
 *********************************************/

define_stdfunction!(
    ord { c: Standard::String },
    returns = I64,
    docs = {
        category: "String",
        description: "Returns the Unicode code point of the character at the specified index.",
        ext_description: "
            Will always return a 32bit value, regardless of the width of the character.
            This is the complement of chr(); Output from one is valid input for the other.
        ",
        examples: "
            assert_eq(97u32, ord('a'))
        "
    },
    handler = (state, _reference) {
        let input = required_arg!(state::c).to_string();
        if input.len() != 1 {
            return oops!(Custom {
                msg: "ord() expected a single character".to_string()
            });
        }
        let c = input.chars().next().unwrap();
        Ok(Value::from(c as u32))
    },
);

define_stdfunction!(
    chr { i: Standard::I64 },
    returns = String,
    docs = {
        category: "String",
        description: "Returns a string containing the character represented by the Unicode code point.",
        ext_description: "
            This is the complement of ord(); Output from one is valid input for the other.
        ",
        examples: "
            assert_eq('a', chr(97))
        "
    },
    handler = (state, _reference) {
        let input = required_arg!(state::i).as_a::<u32>()?;
        match std::char::from_u32(input) {
            Some(c) => Ok(Value::from(c.to_string())),
            None => oops!(Custom {
                msg: "chr() expected a valid Unicode code point".to_string()
            }),
        }
    },
);

/**********************************************
 *
 * String Manipulation
 *********************************************/

define_stdfunction!(
    uppercase { s: Standard::String },
    returns = String,
    docs = {
        category: "String",
        description: "Converts a string to uppercase.",
        ext_description: "This function is locale-insensitive and will handle all Unicode characters.",
        examples: "
            assert_eq('HELLO', uppercase('hello'))
        "
    },
    handler = (state, _reference) {
        let input = required_arg!(state::s).to_string();
        Ok(Value::from(input.to_uppercase().to_string()))
    },
);

define_stdfunction!(
    lowercase { s: Standard::String },
    returns = String,
    docs = {
        category: "String",
        description: "Converts a string to lowercase.",
        ext_description: "This function is locale-insensitive and will handle all Unicode characters.",
        examples: "
            assert_eq('hello', lowercase('HELLO'))
        "
    },
    handler = (state, _reference) {
        let input = required_arg!(state::s).to_string();
        Ok(Value::from(input.to_lowercase().to_string()))
    },
);

define_stdfunction!(
    trim { s: Standard::String },
    returns = String,
    docs = {
        category: "String",
        description: "Removes leading and trailing whitespace from a string.",
        ext_description: "This function is locale-insensitive and will handle all Unicode characters.",
        examples: "
            assert_eq('hello', trim('  hello  '))
        "
    },
    handler = (state, _reference) {
        let input = required_arg!(state::s).to_string();
        Ok(Value::from(input.trim().to_string()))
    },
);

define_stdfunction!(
    trim_start { s: Standard::String },
    returns = String,
    docs = {
        category: "String",
        description: "Removes leading whitespace from a string.",
        ext_description: "This function is locale-insensitive and will handle all Unicode characters.",
        examples: "
            assert_eq('hello  ', trim_start('  hello  '))
        "
    },
    handler = (state, _reference) {
        let input = required_arg!(state::s).to_string();
        Ok(Value::from(input.trim_start().to_string()))
    },
);

define_stdfunction!(
    trim_end { s: Standard::String },
    returns = String,
    docs = {
        category: "String",
        description: "Removes trailing whitespace from a string.",
        ext_description: "This function is locale-insensitive and will handle all Unicode characters.",
        examples: "
            assert_eq('  hello', trim_end('  hello  '))
        "
    },
    handler = (state, _reference) {
        let input = required_arg!(state::s).to_string();
        Ok(Value::from(input.trim_end().to_string()))
    },
);

define_stdfunction!(
    replace {
        s: Standard::String,
        from: Standard::String,
        to: Standard::String
    },
    returns = String,
    docs = {
        category: "String",
        description: "Replaces all occurrences of a substring within a string with another string.",
        ext_description: "This function is locale-insensitive and will handle all Unicode characters.",
        examples: "
            assert_eq('hello world', replace('hello there', 'there', 'world'))
        "
    },
    handler = (state, _reference) {
        let input = required_arg!(state::s).to_string();
        let from = required_arg!(state::from).to_string();
        let to = required_arg!(state::to).to_string();
        Ok(Value::from(input.replace(&from, &to)))
    },
);

define_stdfunction!(
    repeat {
        s: Standard::String,
        n: Standard::I64
    },
    returns = String,
    docs = {
        category: "String",
        description: "Repeats a string a specified number of times.",
        ext_description: "This function is locale-insensitive and will handle all Unicode characters.",
        examples: "
            assert_eq('hellohellohello', repeat('hello', 3))
        "
    },
    handler = (state, _reference) {
        let input = required_arg!(state::s).to_string();
        let n = required_arg!(state::n).as_a::<i32>()?;
        Ok(Value::from(input.repeat(n as usize)))
    },
);

define_stdfunction!(
    chars {
        s: Standard::String
    },
    returns = Array,
    docs = {
        category: "String",
        description: "Splits a string into its individual characters.",
        ext_description: "This function will handle all Unicode characters.",
        examples: "
            assert_eq(['h', 'e', 'l', 'l', 'o'], chars('hello'))
        "
    },
    handler = (state, _reference) {
        let input = required_arg!(state::s).to_string();
        let chars: Vec<Value> = input.chars().map(|c| c.to_string().into()).collect();
        Ok(Value::from(chars))
    },
);

define_stdfunction!(
    escape {
        s: Standard::String
    },
    returns = String,
    docs = {
        category: "String",
        description: "Escapes special characters in a string.",
        ext_description: "This function will handle all Unicode characters.",
        examples: "
            assert_eq('hello\\\\nworld', escape('hello\nworld'))
        "
    },
    handler = (state, _reference) {
        let input = required_arg!(state::s).to_string();
        let mut output = String::new();
        for c in input.chars() {
            match c {
                '\n' => output.push_str("\\n"),
                '\r' => output.push_str("\\r"),
                '\t' => output.push_str("\\t"),
                '\\' => output.push_str("\\\\"),
                '"' => output.push_str("\\\""),
                _ => output.push(c),
            }
        }
        Ok(Value::from(output))
    },
);

define_stdfunction!(
    pad_right {
        s: Standard::String,
        length: Standard::I64,
        pad: Optional::String
    },
    returns = String,
    docs = {
        category: "String",
        description: "Pads a string to a specified length with a specified character.",
        ext_description: "This function will handle all Unicode characters.",
        examples: "
            assert_eq('hello!!!!!!', pad_right('hello', 11, '!'))
            assert_eq('hello      ', pad_right('hello', 11))
        "
    },
    handler = (state, _reference) {
        let input = required_arg!(state::s).to_string();
        let length = required_arg!(state::length).as_a::<u64>()? as usize;
        let pad = optional_arg!(state::pad).unwrap_or(Value::string(" ")).to_string().chars().next().unwrap_or(' ').to_string();

        let padding = length - input.len();
        if padding <= 0 {
            Ok(Value::from(input))
        } else {
            let pad = pad.repeat(padding);
            Ok((input + &pad).into())
        }
    },
);

define_stdfunction!(
    pad_left {
        s: Standard::String,
        length: Standard::I64,
        pad: Optional::String
    },
    returns = String,
    docs = {
        category: "String",
        description: "Pads a string to a specified length with a specified character.",
        ext_description: "This function will handle all Unicode characters.",
        examples: "
            assert_eq('!!!!!!hello', pad_left('hello', 11, '!'))
            assert_eq('      hello', pad_left('hello', 11))
        "
    },
    handler = (state, _reference) {
        let input = required_arg!(state::s).to_string();
        let length = required_arg!(state::length).as_a::<i64>()?;
        let pad = optional_arg!(state::pad).unwrap_or(Value::string(" ")).to_string().chars().next().unwrap_or(' ').to_string();

        let padding: i64 = length - input.len() as i64;
        if padding <= 0 {
            Ok(Value::from(input))
        } else {
            let pad = pad.repeat(padding as usize);
            Ok((pad + &input).into())
        }
    },
);

/**********************************************
 *
 * String Formatting
 *
 *********************************************/

define_stdfunction!(
    format {
        s: Standard::String,
        args: Standard::Array
    },
    returns = String,
    docs = {
        category: "String",
        description: "Formats a string using positional arguments.",
        ext_description: "The 2nd argument is an array of values to be consumed in order",
        examples: "
            assert_eq('hello world', format('hello {}', ['world']))
        "
    },
    handler = (state, _reference) {
        let input = required_arg!(state::s).to_string();
        let args = required_arg!(state::args).as_a::<Vec<Value>>()?;
        let args: Vec<String> = args
            .iter()
            .map(|v| v.to_string())
            .collect();

        let mut result = input;
        for arg in args {
            let arg = arg.clone().to_string();
            // Replace first instance of {} with arg
            result = result.replacen("{}", &arg, 1);
        }

        Ok(result.into())
    },
);

define_stdfunction!(
    prettyjson { s: Standard::Object },
    returns = String,
    docs = {
        category: "String",
        description: "Formats a JSON string for human readability.",
        ext_description: "This function will handle all Unicode characters.",
        examples: "
            assert_eq(
                '{\n  \"hello\": \"world\"\n}',
                prettyjson({\"hello\": \"world\"})
            )
        "
    },
    handler = (state, _reference) {
        let input = required_arg!(state::s).as_type(ValueType::Object)?.to_json_string();
        let input = serde_json::from_str::<serde_json::Value>(&input)?;
        Ok(Value::from(serde_json::to_string_pretty(&input)?))
    },
);

define_stdfunction!(
    join {
        parts: Standard::Array,
        joiner: Optional::String
    },
    returns = String,
    docs = {
        category: "String",
        description: "Concatenates an array of values into a single string.",
        ext_description: "
            Converts all its arguments to strings and then concatenates them.
            If a joiner is provided, it will be used to separate the parts.
        ",
        examples: "
            assert_eq('hello world', join(['hello', ' ', 'world']))
            assert_eq('hello world', ['hello', 'world'].join(' '))
        "
    },
    handler = (state, _reference) {
        let joiner = optional_arg!(state::joiner).unwrap_or(Value::string("")).to_string();
        let parts = required_arg!(state::parts).as_a::<Vec<Value>>()?;
        let parts: Vec<String> = parts
            .iter()
            .map(|v| v.to_string())
            .collect();
        Ok(Value::from(parts.join(&joiner)))
    },
);

/**********************************************
 *
 * String Encoding
 * urlencode, urldecode, atob, btoa
 *********************************************/

#[cfg(feature = "encoding-functions")]
define_stdfunction!(
    url_encode { s: Standard::String },
    returns = String,
    docs = {
        category: "String",
        description: "Encodes a string as a URL-safe string.",
        ext_description: "This function will handle all Unicode characters.",
        examples: "
            assert_eq('hello%20world', url_encode('hello world'))
        "
    },
    handler = (state, _reference) {
        let input = required_arg!(state::s).to_string();
        Ok(Value::from(urlencoding::encode(&input).into_owned()))
    },
);

#[cfg(feature = "encoding-functions")]
define_stdfunction!(
    url_decode { s: Standard::String },
    returns = String,
    docs = {
        category: "String",
        description: "Decodes a URL-safe string into a normal string.",
        ext_description: "This function will handle all Unicode characters.",
        examples: "
            assert_eq('hello world', url_decode('hello%20world'))
        "
    },
    handler = (state, _reference) {
        let input = required_arg!(state::s).to_string();
        Ok(Value::from(urlencoding::decode(&input)?.into_owned()))
    },
);

#[cfg(feature = "encoding-functions")]
define_stdfunction!(
    base64_encode { s: Standard::String },
    returns = String,
    docs = {
        category: "String",
        description: "Encodes a string into base64",
        ext_description: "This function will handle all Unicode characters.",
        examples: "
            assert_eq('aGVsbG8gd29ybGQ=', base64_encode('hello world'))
        "
    },
    handler = (state, _reference) {
        let input = required_arg!(state::s).to_string();

        use base64::{engine::general_purpose, Engine as _};
        let mut buf = String::new();
        general_purpose::STANDARD.encode_string(&input, &mut buf);
        Ok(Value::from(buf))
    },
);

#[cfg(feature = "encoding-functions")]
define_stdfunction!(
    base64_decode { s: Standard::String },
    returns = String,
    docs = {
        category: "String",
        description: "Decodes a base64 string into a string.",
        ext_description: "This function will handle all Unicode characters.",
        examples: "
            assert_eq('hello world', base64_decode('aGVsbG8gd29ybGQ='))
        "
    },
    handler = (state, _reference) {
        let input = required_arg!(state::s).to_string();

        use base64::{engine::general_purpose, Engine as _};
        if let Ok(bytes) = general_purpose::STANDARD.decode(input) {
            if let Ok(s) = std::str::from_utf8(&bytes) {
                return Ok(Value::from(s));
            }
        }

        oops!(
            ValueFormat {
                expected_format: "base64".to_string()
            }
        )
    },
);

define_stdfunction!(
    from_json {
        s: Standard::String
    },
    returns = Any,
    docs = {
        category: "String",
        description: "Parses a JSON string into a value.",
        ext_description: "This function will handle all Unicode characters.",
        examples: "
            assert_eq({\"hello\": \"world\"}, from_json('{\"hello\": \"world\"}'))
        "
    },
    handler = (state, _reference) {
        let input = required_arg!(state::s).to_string();
        let input = serde_json::from_str::<serde_json::Value>(&input)?;
        Ok(Value::try_from(input)?)
    },
);

define_stdfunction!(
    to_json {
        v: Standard::Any
    },
    returns = String,
    docs = {
        category: "String",
        description: "Converts a value into a JSON string.",
        ext_description: "
            Objects will be encoded as (key, value) pairs, due to differences between JSON and lavendeux.
        ",
        examples: "
            assert_eq('{\"hello\":\"world\"}', to_json({'hello': 'world'}))
        "
    },
    handler = (state, _reference) {
        let input = required_arg!(state::v).to_json_string();
        Ok(Value::from(input))
    },
);
