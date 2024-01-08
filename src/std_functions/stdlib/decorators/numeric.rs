use crate::{
    get_argument, required_argument, static_decorator, static_function, std_functions::Function,
    Error, State,
};
use polyvalue::{
    operations::ArithmeticOperationExt,
    types::{Float, Int},
    Value, ValueTrait, ValueType,
};
use std::collections::HashMap;

pub fn register_all(map: &mut HashMap<String, Function>) {
    static_decorator!(
        registry = map,
        name = "roman",
        description = "Interprets an integer as a roman numeral",
        expected_type = ValueType::Numeric,
        handler = &|input: Value| {
            let mut input = *input.as_a::<Int>()?.inner();
            if input > 3999 {
                return Err(Error::Overflow);
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

    static_decorator!(
        registry = map,
        name = "ordinal",
        description = "Interprets an integer as an ordinal number",
        expected_type = ValueType::Numeric,
        handler = &|input: Value| {
            let input = *input.as_a::<Int>()?.inner();
            let ordinal = match input % 10 {
                1 => format!("{}st", input),
                2 => format!("{}nd", input),
                3 => format!("{}rd", input),
                _ => format!("{}th", input),
            };
            Ok(ordinal)
        }
    );

    static_decorator!(
        registry = map,
        name = "utc",
        description = "Interprets an integer as a timestamp, and formats it in UTC standard",
        expected_type = ValueType::Numeric,
        handler = &|input: Value| {
            let input = input.as_a::<Int>()?;
            let input = Int::arithmetic_op(
                &input,
                &Int::new(1000),
                polyvalue::operations::ArithmeticOperation::Multiply,
            )?;
            let input = *input.inner();

            match chrono::NaiveDateTime::from_timestamp_millis(input) {
                Some(t) => {
                    let datetime: chrono::DateTime<chrono::Utc> =
                        chrono::DateTime::from_naive_utc_and_offset(t, chrono::Utc);
                    Ok(datetime.format("%Y-%m-%d %H:%M:%S").to_string())
                }
                None => Err(Error::Range(input.to_string())),
            }
        }
    );

    static_decorator!(
        registry = map,
        name = "percent",
        description = "Interprets a number as a percentage",
        expected_type = ValueType::Numeric,
        handler = &|input: Value| {
            let input = *input.as_a::<Float>()?.inner();
            Ok(format!("{}%", input * 100.0))
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
    fn test_numeric() {
        test_decorator!("roman", Value::from(123), "CXXIII");
        test_decorator!("ordinal", Value::from(123), "123rd");
        test_decorator!("percent", Value::from(0.123), "12.3%");
        test_decorator!("utc", Value::from(123), "1970-01-01 00:02:03");
    }
}
