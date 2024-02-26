use crate::{
    error::WrapExternalError, get_argument, required_argument, static_decorator, static_function,
    std_functions::Function, State,
};
use polyvalue::{
    types::{Bool, Float, I64},
    InnerValue, ValueTrait, ValueType,
};
use std::collections::HashMap;

pub fn register_all(map: &mut HashMap<String, Function>) {
    //
    // Value-literal decorators
    // These should match the input format of Lavendeux's value literals
    //

    static_decorator!(
        registry = map,
        name = "hex",
        description = "Base 16 number formatting, such as 0xFF",
        expected_type = ValueType::Numeric,
        handler = &|input, token| {
            match input.inner() {
                InnerValue::U8(v) => Ok(format!("{:#0x}", v.inner())),
                InnerValue::I8(v) => Ok(format!("{:#0x}", v.inner())),
                InnerValue::U16(v) => Ok(format!("{:#0x}", v.inner())),
                InnerValue::I16(v) => Ok(format!("{:#0x}", v.inner())),
                InnerValue::U32(v) => Ok(format!("{:#0x}", v.inner())),
                InnerValue::I32(v) => Ok(format!("{:#0x}", v.inner())),
                InnerValue::U64(v) => Ok(format!("{:#0x}", v.inner())),
                _ => {
                    let input = *input.as_a::<I64>().with_context(token)?.inner();
                    Ok(format!("{:#0x}", input))
                }
            }
        }
    );

    static_decorator!(
        registry = map,
        name = "oct",
        description = "Base 8 number formatting, such as 0o77",
        expected_type = ValueType::Numeric,
        handler = &|input, token| {
            match input.inner() {
                InnerValue::U8(v) => Ok(format!("{:#0o}", v.inner())),
                InnerValue::I8(v) => Ok(format!("{:#0o}", v.inner())),
                InnerValue::U16(v) => Ok(format!("{:#0o}", v.inner())),
                InnerValue::I16(v) => Ok(format!("{:#0o}", v.inner())),
                InnerValue::U32(v) => Ok(format!("{:#0o}", v.inner())),
                InnerValue::I32(v) => Ok(format!("{:#0o}", v.inner())),
                InnerValue::U64(v) => Ok(format!("{:#0o}", v.inner())),
                _ => {
                    let input = *input.as_a::<I64>().with_context(token)?.inner();
                    Ok(format!("{:#0o}", input))
                }
            }
        }
    );

    static_decorator!(
        registry = map,
        name = "bin",
        description = "Base 2 number formatting, such as 0b101",
        expected_type = ValueType::Numeric,
        handler = &|input, token| {
            match input.inner() {
                InnerValue::U8(v) => Ok(format!("{:#0b}", v.inner())),
                InnerValue::I8(v) => Ok(format!("{:#0b}", v.inner())),
                InnerValue::U16(v) => Ok(format!("{:#0b}", v.inner())),
                InnerValue::I16(v) => Ok(format!("{:#0b}", v.inner())),
                InnerValue::U32(v) => Ok(format!("{:#0b}", v.inner())),
                InnerValue::I32(v) => Ok(format!("{:#0b}", v.inner())),
                InnerValue::U64(v) => Ok(format!("{:#0b}", v.inner())),
                _ => {
                    let input = *input.as_a::<I64>().with_context(token)?.inner();
                    Ok(format!("{:#0b}", input))
                }
            }
        }
    );

    static_decorator!(
        registry = map,
        name = "sci",
        description = "Scientific notation formatting, such as 1.2e3",
        expected_type = ValueType::Numeric,
        handler = &|input, token| {
            let input = *input.as_a::<I64>().with_context(token)?.inner();
            Ok(format!("{:e}", input))
        }
    );

    static_decorator!(
        registry = map,
        name = "float",
        description = "Formats a number as a floating point number",
        expected_type = ValueType::Numeric,
        handler = &|input, token| {
            let input = input.as_a::<Float>().with_context(token)?;
            Ok(input.to_string())
        }
    );

    static_decorator!(
        registry = map,
        name = "int",
        description = "Format a number as an integer",
        expected_type = ValueType::Numeric,
        handler = &|input, token| {
            let input = input.as_a::<I64>().with_context(token)?;
            Ok(input.to_string())
        }
    );

    static_decorator!(
        registry = map,
        name = "bool",
        description = "Format a value as a boolean",
        expected_type = ValueType::Any,
        handler = &|input, token| {
            let input = input.as_a::<Bool>().with_context(token)?;
            Ok(input.to_string())
        }
    );
}

//
// Tests
//

#[cfg(test)]
mod test {
    use crate::{state, test_decorator, Token};
    use polyvalue::Value;

    #[test]
    fn test_literal() {
        test_decorator!("hex", Value::from(123), "0x7b");
        test_decorator!("oct", Value::from(123), "0o173");
        test_decorator!("bin", Value::from(123), "0b1111011");
        test_decorator!("sci", Value::from(123), "1.23e2");
        test_decorator!("float", Value::from(123), "123.0");
        test_decorator!("int", Value::from(123.0), "123");
        test_decorator!("bool", Value::from(123), "true");
    }
}
