use crate::define_stdfunction;
use polyvalue::{
    operations::{BitwiseOperation, BitwiseOperationExt},
    types::I64,
    InnerValue,
};

macro_rules! define_standard_bitwise_fn {
    ($operation:ident, $examples:literal, $bitwise_op:ident) => {
        define_stdfunction!(
            $operation {
                left: Standard::Int,
                right: Standard::Int
            },
            returns = Int,
            docs = {
                category: "Bitwise",
                description: concat!("Performs a bitwise ", stringify!($operation), " operation on two integers"),
                ext_description: "
                    Floats and Fixed-point numbers will be truncated to integers before the operation is performed.
                ",
                examples: $examples,
            },
            handler = (state, _reference) {
                let left = required_arg!(state::left);
                let right = required_arg!(state::right);
                Ok(left.bitwise_op(right, BitwiseOperation::$bitwise_op)?)
            },
        );
    };
}

define_standard_bitwise_fn!(xor, "assert_eq(0b1010, xor(0b1100, 0b0110))", Xor);
define_standard_bitwise_fn!(and, "assert_eq(0b0100, and(0b1100, 0b0110))", And);
define_standard_bitwise_fn!(or, "assert_eq(0b1110, or(0b1100, 0b0110))", Or);

define_stdfunction!(
    not {
        value: Standard::Int
    },
    returns = Int,
    docs = {
        category: "Bitwise",
        description: "Performs a bitwise NOT operation on an integer",
        ext_description: "
            Floats and Fixed-point numbers will be truncated to integers before the operation is performed.
        ",
        examples: "
            assert_eq(0b1111_1111u8, not(0b0000_0000u8))
        ",
    },
    handler = (state, _reference) {
        let value = required_arg!(state::value);
        Ok(value.bitwise_not()?)
    },
);

define_stdfunction!(
    llshift {
        value: Standard::Int,
        shift: Standard::Int
    },
    returns = Int,
    docs = {
        category: "Bitwise",
        description: "Performs a logical bitwise left shift operation on an integer",
        ext_description: "
            Floats and Fixed-point numbers will be truncated to integers before the operation is performed.
            Will always ignore the sign bit.
        ",
        examples: "
            assert_eq(
                0b1000_0010i8,
                llshift(0b0100_0001i8, 1)
            )
        ",
    },
    handler = (state, _reference) {
        let value = required_arg!(state::value);
        let shift = required_arg!(state::shift).as_a::<i32>()?;

        Ok(match value.inner() {
            InnerValue::U8(v) => v.logical_lshift(shift)?.into(),
            InnerValue::U16(v) => v.logical_lshift(shift)?.into(),
            InnerValue::U32(v) => v.logical_lshift(shift)?.into(),
            InnerValue::U64(v) => v.logical_lshift(shift)?.into(),
            InnerValue::I8(v) => v.logical_lshift(shift)?.into(),
            InnerValue::I16(v) => v.logical_lshift(shift)?.into(),
            InnerValue::I32(v) => v.logical_lshift(shift)?.into(),
            _ => {
                let v = value.as_a::<I64>()?;
                v.logical_lshift(shift)?.into()
            }
        })
    },
);

define_stdfunction!(
    lrshift {
        value: Standard::Int,
        shift: Standard::Int
    },
    returns = Int,
    docs = {
        category: "Bitwise",
        description: "Performs a logical bitwise right shift operation on an integer",
        ext_description: "
            Floats and Fixed-point numbers will be truncated to integers before the operation is performed.
            Will always ignore the sign bit.
        ",
        examples: "
            assert_eq(
                0b0100_0000i8,
                lrshift(0b1000_0001i8, 1)
            )
        ",
    },
    handler = (state, _reference) {
        let value = required_arg!(state::value);
        let shift = required_arg!(state::shift).as_a::<i32>()?;

        Ok(match value.inner() {
            InnerValue::U8(v) => v.logical_rshift(shift)?.into(),
            InnerValue::U16(v) => v.logical_rshift(shift)?.into(),
            InnerValue::U32(v) => v.logical_rshift(shift)?.into(),
            InnerValue::U64(v) => v.logical_rshift(shift)?.into(),
            InnerValue::I8(v) => v.logical_rshift(shift)?.into(),
            InnerValue::I16(v) => v.logical_rshift(shift)?.into(),
            InnerValue::I32(v) => v.logical_rshift(shift)?.into(),
            _ => {
                let v = value.as_a::<I64>()?;
                v.logical_rshift(shift)?.into()
            }
        })
    },
);
