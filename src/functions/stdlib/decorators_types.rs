use crate::{define_stddecorator, functions::std_function::ParserFunction, Error};
use polyvalue::{
    types::{Bool, Float, I64},
    InnerValue, ValueTrait,
};

define_stddecorator!(
    hex { input: Numeric },
    docs = {
        description: "Base 16 number formatting, such as 0xFF",
        ext_description: "Converts a number to a hexadecimal string. The output will be prefixed with '0x' with a length based on the input type.",
        examples: "
            assert_eq(
                255 @hex,
                '0xff'
            )
        "
    },
    handler = (input) {
        match input.inner() {
            InnerValue::U8(v) => Ok(format!("{:#0x}", v.inner())),
            InnerValue::I8(v) => Ok(format!("{:#0x}", v.inner())),
            InnerValue::U16(v) => Ok(format!("{:#0x}", v.inner())),
            InnerValue::I16(v) => Ok(format!("{:#0x}", v.inner())),
            InnerValue::U32(v) => Ok(format!("{:#0x}", v.inner())),
            InnerValue::I32(v) => Ok(format!("{:#0x}", v.inner())),
            InnerValue::U64(v) => Ok(format!("{:#0x}", v.inner())),
            _ => {
                let input = input.as_a::<i64>()?;
                Ok(format!("{:#0x}", input))
            }
        }
    }
);

define_stddecorator!(
    oct { input: Numeric },
    docs = {
        description: "Base 8 number formatting, such as 0o77",
        ext_description: "Converts a number to an octal string. The output will be prefixed with '0o' with a length based on the input type.",
        examples: "
            assert_eq(
                255 @oct,
                '0o377'
            )
        "
    },
    handler = (input) {
        match input.inner() {
            InnerValue::U8(v) => Ok(format!("{:#0o}", v.inner())),
            InnerValue::I8(v) => Ok(format!("{:#0o}", v.inner())),
            InnerValue::U16(v) => Ok(format!("{:#0o}", v.inner())),
            InnerValue::I16(v) => Ok(format!("{:#0o}", v.inner())),
            InnerValue::U32(v) => Ok(format!("{:#0o}", v.inner())),
            InnerValue::I32(v) => Ok(format!("{:#0o}", v.inner())),
            InnerValue::U64(v) => Ok(format!("{:#0o}", v.inner())),
            _ => {
                let input = input.as_a::<i64>()?;
                Ok(format!("{:#0o}", input))
            }
        }
    }
);

define_stddecorator!(
    bin { input: Numeric },
    docs = {
        description: "Base 2 number formatting, such as 0b101",
        ext_description: "Converts a number to a binary string. The output will be prefixed with '0b' with a length based on the input type.",
        examples: "
            assert_eq(
                255 @bin,
                '0b11111111'
            )
        "
    },
    handler = (input) {
        match input.inner() {
            InnerValue::U8(v) => Ok(format!("{:#0b}", v.inner())),
            InnerValue::I8(v) => Ok(format!("{:#0b}", v.inner())),
            InnerValue::U16(v) => Ok(format!("{:#0b}", v.inner())),
            InnerValue::I16(v) => Ok(format!("{:#0b}", v.inner())),
            InnerValue::U32(v) => Ok(format!("{:#0b}", v.inner())),
            InnerValue::I32(v) => Ok(format!("{:#0b}", v.inner())),
            InnerValue::U64(v) => Ok(format!("{:#0b}", v.inner())),
            _ => {
                let input = input.as_a::<i64>()?;
                Ok(format!("{:#0b}", input))
            }
        }
    }
);

define_stddecorator!(
    sci { input: Numeric },
    docs = {
        description: "Scientific notation",
        ext_description: "Converts a floating point number to sci notation.",
        examples: "
            assert_eq(
                1000000.0 @sci,
                '1e6'
            )
        "
    },
    handler = (input) {
        let input = input.as_a::<f64>()?;
        Ok(format!("{:e}", input))
    }
);

define_stddecorator!(
    float { input: Numeric },
    docs = {
        description: "Floating point number formatting",
        ext_description: "Converts a number to a floating point string.",
        examples: "
            assert_eq(
                1.0 @float,
                '1.0'
            )
        "
    },
    handler = (input) {
        let input = input.as_a::<Float>()?;
        Ok(input.to_string())
    }
);

define_stddecorator!(
    int { input: Numeric },
    docs = {
        description: "Integer number formatting",
        ext_description: "Converts a number to an integer string.",
        examples: "
            assert_eq(
                1000000 @int,
                '1000000'
            )
        "
    },
    handler = (input) {
        let input = input.as_a::<I64>()?;
        Ok(input.to_string())
    }
);

define_stddecorator!(
    bool { input: Any },
    docs = {
        description: "Boolean formatting",
        ext_description: "Converts a number to a boolean string.",
        examples: "
            assert_eq(
                1 @bool,
                'true'
            
            )
        "
    },
    handler = (input) {
        let input = input.as_a::<Bool>()?;
        Ok(input.to_string())
    }
);
