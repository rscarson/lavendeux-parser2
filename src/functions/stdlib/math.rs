use crate::{
    define_stdfunction,
    error::{ErrorDetails, WrapOption},
};
use polyvalue::{fpdec::Round, types::CurrencyInner, InnerValue, Value, ValueTrait};

define_stdfunction!(
    min {
        options: Standard::Array
    },
    returns = Numeric,
    docs = {
        category: "Math",
        description: "Returns the smallest value in the given array",
        ext_description: "
            The array can contain any number of elements, and they can be of any type.
            Since all values in lavendeux are comparable, the function will work with any type of array.
        ",
        examples: "
            assert_eq(
                min([1, 2, 3, 4, 5]),
                1
            )
        "
    },
    handler = (state) {
        let options = required_arg!(state::options).as_a::<Vec<Value>>()?;
        let min = options.iter().min().or_error(ErrorDetails::ArrayEmpty)?;
        Ok(min.clone())
    }
);

define_stdfunction!(
    max {
        options: Standard::Array
    },
    returns = Numeric,
    docs = {
        category: "Math",
        description: "Returns the largest value in the given array",
        ext_description: "
            The array can contain any number of elements, and they can be of any type.
            Since all values in lavendeux are comparable, the function will work with any type of array.
        ",
        examples: "
            assert_eq(
                max([1, 2, 3, 4, 5]),
                5
            )
        "
    },
    handler = (state) {
        let options = required_arg!(state::options).as_a::<Vec<Value>>()?;
        if options.is_empty() {
            return oops!(ArrayEmpty)
        }
        let max = options.iter().max().or_error(ErrorDetails::ArrayEmpty)?;
        Ok(max.clone())
    }
);

define_stdfunction!(
    ceil {
        value: Standard::Numeric
    },
    returns = Numeric,
    docs = {
        category: "Math",
        description: "Rounds a number up to the nearest whole number",
        ext_description: "
            The function will round the input number up to the nearest whole number.
            If the input number is already a whole number, the function will return the input number.
        ",
        examples: "
            assert_eq(
                ceil(1.5),
                2.0
            )
        "
    },
    handler = (state) {
        let value = required_arg!(state::value).as_a::<f64>()?;
        Ok(value.ceil().into())
    }
);

define_stdfunction!(
    floor {
        value: Standard::Numeric
    },
    returns = Numeric,
    docs = {
        category: "Math",
        description: "Rounds a number down to the nearest whole number",
        ext_description: "
            The function will round the input number down to the nearest whole number.
            If the input number is already a whole number, the function will return the input number.
        ",
        examples: "
            assert_eq(
                floor(1.5),
                1.0
            )
        "
    },
    handler = (state) {
        let value = required_arg!(state::value).as_a::<f64>()?;
        Ok(value.floor().into())
    }
);

define_stdfunction!(
    abs {
        value: Standard::Numeric
    },
    returns = Numeric,
    docs = {
        category: "Math",
        description: "Returns the absolute value of a number",
        ext_description: "
            The function will return the absolute value of the input number.
        ",
        examples: "
            assert_eq(
                abs(-5),
                5
            )
        "
    },
    handler = (state) {
        let value = required_arg!(state::value);
        match value.inner() {
            InnerValue::Fixed(n) => Ok(Value::fixed(n.inner().abs())),
            InnerValue::Currency(n) => {
                let symbol = n.symbol().clone();
                let precision = n.precision();
                let value = n.inner().value().inner().abs();
                Ok(CurrencyInner::new(symbol, precision, value.into()).into())
            },

            InnerValue::Float(n) => Ok(Value::from(n.inner().abs())),

            InnerValue::U8(n) => Ok(Value::from(n.clone())),
            InnerValue::U16(n) => Ok(Value::from(n.clone())),
            InnerValue::U32(n) => Ok(Value::from(n.clone())),
            InnerValue::U64(n) => Ok(Value::from(n.clone())),

            InnerValue::I8(n) => Ok(Value::from(n.abs())),
            InnerValue::I16(n) => Ok(Value::from(n.abs())),
            InnerValue::I32(n) => Ok(Value::from(n.abs())),
            InnerValue::I64(n) => Ok(Value::from(n.abs())),

            _ => oops!(
                Internal {
                    msg: "Invalid argument type".to_string()
                }
            ),

        }
    }
);

define_stdfunction!(
    round {
        value: Standard::Numeric,
        precision: Optional::Int
    },
    returns = Numeric,
    docs = {
        category: "Math",
        description: "Rounds a number to the nearest whole number",
        ext_description: "
            The function will round the input number to the nearest whole number.
            If the input number is already a whole number, the function will return the input number.
        ",
        examples: "
            assert_eq(
                round(1.5),
                2.0
            )
        "
    },
    handler = (state) {
        let value = required_arg!(state::value);
        let precision = optional_arg!(state::precision).unwrap_or(0.into()).as_a::<i64>()?;

        match value.inner() {
            InnerValue::Fixed(n) => Ok(Value::from(n.inner().clone().round(precision as i8))),
            InnerValue::Currency(n) => {
                let symbol = n.symbol().clone();
                let precision = n.precision();
                let value = n.inner().value().inner().clone().round(precision);
                Ok(CurrencyInner::new(symbol, precision, value.into()).into())
            },

            InnerValue::Float(n) => {
                let n = n.inner();
                let n = n * 10.0_f64.powi(precision as i32);
                let n = n.round();
                let n = n / 10.0_f64.powi(precision as i32);
                Ok(Value::from(n))
            }
            _ => oops!(
                Internal {
                    msg: "Invalid argument type".to_string()
                }
            ),
        }
    }
);

define_stdfunction!(
    log2 {
        value: Standard::Numeric
    },
    returns = Numeric,
    docs = {
        category: "Math",
        description: "Returns the base-2 logarithm of a number",
        ext_description: "",
        examples: "
            assert_eq(
                log2(8),
                3
            )
        "
    },
    handler = (state) {
        let value = required_arg!(state::value);
        let type_name = value.own_type();
        let value = value.as_a::<f64>()?;
        Ok(Value::from(value.log2()).as_type(type_name)?)
    }
);

define_stdfunction!(
    ilog2 {
        value: Standard::Int
    },
    returns = Numeric,
    docs = {
        category: "Math",
        description: "Returns the base-2 logarithm of a number, rounded down to the nearest whole number",
        ext_description: "",
        examples: "
            assert_eq(
                ilog2(8),
                3
            )
        "
    },
    handler = (state) {
        let value = required_arg!(state::value);
        let type_name = value.own_type();
        let value = value.as_a::<i64>()?;
        Ok(Value::from(value.ilog2()).as_type(type_name)?)
    }
);

define_stdfunction!(
    log10 {
        value: Standard::Numeric
    },
    returns = Numeric,
    docs = {
        category: "Math",
        description: "Returns the base-10 logarithm of a number",
        ext_description: "",
        examples: "
            assert_eq(
                log10(100),
                2
            )
        "
    },
    handler = (state) {
        let value = required_arg!(state::value);
        let type_name = value.own_type();
        let value = value.as_a::<f64>()?;
        Ok(Value::from(value.log10()).as_type(type_name)?)
    }
);

define_stdfunction!(
    ln {
        value: Standard::Numeric
    },
    returns = Numeric,
    docs = {
        category: "Math",
        description: "Returns the natural logarithm of a number",
        ext_description: "",
        examples: "
            assert_eq(
                ln(2.718281828459045),
                1.0
            )
        "
    },
    handler = (state) {
        let value = required_arg!(state::value);
        let type_name = value.own_type();
        let value = value.as_a::<f64>()?;
        Ok(Value::from(value.ln()).as_type(type_name)?)
    }
);

define_stdfunction!(
    log {
        value: Standard::Numeric,
        base: Optional::Numeric
    },
    returns = Numeric,
    docs = {
        category: "Math",
        description: "Returns the logarithm of a number to a given base",
        ext_description: "",
        examples: "
            assert_eq(
                log(8, 2),
                3.0
            )
        "
    },
    handler = (state) {
        let value = required_arg!(state::value).as_a::<f64>()?;
        let base = optional_arg!(state::base).unwrap_or(10.into()).as_a::<f64>()?;
        Ok(value.log(base).into())
    }
);

define_stdfunction!(
    sqrt {
        value: Standard::Numeric
    },
    returns = Numeric,
    docs = {
        category: "Math",
        description: "Returns the square root of a number",
        ext_description: "",
        examples: "
            assert_eq(
                sqrt(9),
                3.0
            )
        "
    },
    handler = (state) {
        let value = required_arg!(state::value).as_a::<f64>()?;
        Ok(value.sqrt().into())
    }
);

define_stdfunction!(
    root {
        value: Standard::Numeric,
        root: Standard::Numeric
    },
    returns = Numeric,
    docs = {
        category: "Math",
        description: "Returns the nth root of a number",
        ext_description: "",
        examples: "
            assert_eq(
                root(8, 3),
                2.0
            )
        "
    },
    handler = (state) {
        let value = required_arg!(state::value).as_a::<f64>()?;
        let root = required_arg!(state::root).as_a::<f64>()?;
        Ok(value.powf(1.0 / root).into())
    }
);
