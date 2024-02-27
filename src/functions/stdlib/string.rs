use crate::{define_stdfunction, functions::std_function::ParserFunction, State};
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
    handler = |state: &mut State| {
        let input = state.get_variable("c").unwrap().to_string();
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
    handler = |state: &mut State| {
        let input = state.get_variable("i").unwrap().as_a::<u32>()?;
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
    handler = |state: &mut State| {
        let input = state.get_variable("s").unwrap().to_string();
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
    handler = |state: &mut State| {
        let input = state.get_variable("s").unwrap().to_string();
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
    handler = |state: &mut State| {
        let input = state.get_variable("s").unwrap().to_string();
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
    handler = |state: &mut State| {
        let input = state.get_variable("s").unwrap().to_string();
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
    handler = |state: &mut State| {
        let input = state.get_variable("s").unwrap().to_string();
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
    handler = |state: &mut State| {
        let input = state.get_variable("s").unwrap().to_string();
        let from = state.get_variable("from").unwrap().to_string();
        let to = state.get_variable("to").unwrap().to_string();
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
    handler = |state: &mut State| {
        let input = state.get_variable("s").unwrap().to_string();
        let n = state.get_variable("n").unwrap().as_a::<i32>()?;
        Ok(Value::from(input.repeat(n as usize)))
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
    handler = |state: &mut State| {
        let input = state.get_variable("s").unwrap().to_string();
        let args = state.get_variable("args").unwrap().as_a::<Vec<Value>>()?;
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
    handler = |state: &mut State| {
        let input = state.get_variable("s").unwrap().as_type(ValueType::Object)?.to_json_string();
        let input = serde_json::from_str::<serde_json::Value>(&input)?;
        Ok(Value::from(serde_json::to_string_pretty(&input)?))
    },
);

define_stdfunction!(
    concat {
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
            assert_eq('hello world', concat(['hello', ' ', 'world']))
        "
    },
    handler = |state: &mut State| {
        let joiner = state.get_variable("joiner").unwrap_or(Value::string("")).to_string();
        let parts = state.get_variable("parts").unwrap().as_a::<Vec<Value>>()?;
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
    handler = |state: &mut State| {
        let input = state.get_variable("s").unwrap().to_string();
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
    handler = |state: &mut State| {
        let input = state.get_variable("s").unwrap().to_string();
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
    handler = |state: &mut State| {
        let input = state.get_variable("s").unwrap().to_string();

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
    handler = |state: &mut State| {
        let input = state.get_variable("s").unwrap().to_string();

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
