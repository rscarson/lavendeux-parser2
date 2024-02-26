use crate::{define_stddecorator, functions::std_function::ParserFunction, oops, Error, State};
use polyvalue::{operations::ArithmeticOperationExt, types::I64, Value, ValueTrait};

define_stddecorator!(
    roman { input: Numeric },
    docs = {
        description: "Interprets an integer as a roman numeral",
        ext_description: "Like the roman system before it; this function only supports numbers up to 3999.",
        examples: "
            assert_eq(
                123 @roman,
                'CXXIII'
            )
        "
    },
    handler = |input: Value| -> Result<String, Error> {
        let mut input = input.as_a::<i64>()?;
        if input > 3999 {
            return oops!(Overflow);
        }

        let roman_numerals = vec![
            (1000, "M"),
            (900, "CM"),
            (500, "D"),
            (400, "CD"),
            (100, "C"),
            (90, "XC"),
            (50, "L"),
            (40, "XL"),
            (10, "X"),
            (9, "IX"),
            (5, "V"),
            (4, "IV"),
            (1, "I"),
        ];
        let mut roman_numeral = String::new();
        for (n, r) in roman_numerals {
            while input >= n {
                roman_numeral.push_str(r);
                input -= n;
            }
        }
        Ok(roman_numeral)
    }
);

define_stddecorator!(
    ord { input: Numeric },
    docs = {
        description: "Interprets an integer as an ordinal number",
        ext_description: "This function will append the appropriate suffix to the input number.",
        examples: "
            assert_eq(
                123 @ord,
                '123rd'
            )
        "
    },
    handler = |input: Value| -> Result<String, Error> {
        let input = input.as_a::<i64>()?;
        let ordinal = match input % 10 {
            1 => format!("{}st", input),
            2 => format!("{}nd", input),
            3 => format!("{}rd", input),
            _ => format!("{}th", input),
        };
        Ok(ordinal)
    }
);

define_stddecorator!(
    utc { input: Numeric },
    docs = {
        description: "Interprets an integer as a timestamp, and formats it in UTC standard",
        ext_description: "This function will convert the input number to a UTC timestamp.",
        examples: "
            assert_eq(
                123 @utc,
                '1970-01-01T00:02:03Z'
            )
        "
    },
    handler = |input: Value| -> Result<String, Error> {
        let input = input.as_a::<I64>()?;
        let input = *I64::arithmetic_op(
            &input,
            &I64::new(1000),
            polyvalue::operations::ArithmeticOperation::Multiply,
        )?.inner();

        match chrono::NaiveDateTime::from_timestamp_millis(input) {
            Some(t) => {
                let datetime: chrono::DateTime<chrono::Utc> =
                    chrono::DateTime::from_naive_utc_and_offset(t, chrono::Utc);
                Ok(datetime.format("%Y-%m-%dT%H:%M:%SZ").to_string())
            }
            None => oops!(
                Range {
                    input: input.to_string()
                }
            ),
        }
    }
);

define_stddecorator!(
    percent { input: Numeric },
    docs = {
        description: "Interprets a number as a percentage",
        ext_description: "This function will append a percentage sign to the input number times 100",
        examples: "
            assert_eq(
                0.123 @percent,
                '12.3%'
            
            )
        "
    },
    handler = |input: Value| -> Result<String, Error> {
        let input = input.as_a::<f64>()?;
        Ok(format!("{}%", input * 100.0))
    }
);
