use crate::{
    get_argument, required_argument, static_decorator, static_function, std_functions::Function,
    Error, State,
};
use polyvalue::{
    types::{CurrencyInner, Fixed},
    Value, ValueType,
};
use std::collections::HashMap;

pub fn register_all(map: &mut HashMap<String, Function>) {
    static_decorator!(
        registry = map,
        name = "usd",
        description = "Interprets a number as a USD amount",
        expected_type = ValueType::Numeric,
        handler = &|input: Value| {
            let input = input.as_a::<Fixed>()?;
            Ok(CurrencyInner::as_dollars(input).to_string())
        }
    );

    static_decorator!(
        registry = map,
        name = "eur",
        description = "Interprets a number as a Euro amount",
        expected_type = ValueType::Numeric,
        handler = &|input: Value| {
            let input = input.as_a::<Fixed>()?;
            Ok(CurrencyInner::as_euros(input).to_string())
        }
    );

    static_decorator!(
        registry = map,
        name = "cad",
        description = "Interprets a number as a CAD amount",
        expected_type = ValueType::Numeric,
        handler = &|input: Value| {
            let input = input.as_a::<Fixed>()?;
            Ok(CurrencyInner::as_dollars(input).to_string())
        }
    );

    static_decorator!(
        registry = map,
        name = "aud",
        description = "Interprets a number as a AUD amount",
        expected_type = ValueType::Numeric,
        handler = &|input: Value| {
            let input = input.as_a::<Fixed>()?;
            Ok(CurrencyInner::as_dollars(input).to_string())
        }
    );

    static_decorator!(
        registry = map,
        name = "gbp",
        description = "Interprets a number as a GBP amount",
        expected_type = ValueType::Numeric,
        handler = &|input: Value| {
            let input = input.as_a::<Fixed>()?;
            Ok(CurrencyInner::as_pounds(input).to_string())
        }
    );

    static_decorator!(
        registry = map,
        name = "jpy",
        description = "Interprets a number as a JPY amount",
        expected_type = ValueType::Numeric,
        handler = &|input: Value| {
            let input = input.as_a::<Fixed>()?;
            Ok(CurrencyInner::as_yen(input).to_string())
        }
    );

    static_decorator!(
        registry = map,
        name = "cny",
        description = "Interprets a number as a CNY amount",
        expected_type = ValueType::Numeric,
        handler = &|input: Value| {
            let input = input.as_a::<Fixed>()?;
            Ok(CurrencyInner::as_yuan(input).to_string())
        }
    );

    static_decorator!(
        registry = map,
        name = "rub",
        description = "Interprets a number as a RUB amount",
        expected_type = ValueType::Numeric,
        handler = &|input: Value| {
            let input = input.as_a::<Fixed>()?;
            Ok(CurrencyInner::as_rubles(input).to_string())
        }
    );

    static_decorator!(
        registry = map,
        name = "inr",
        description = "Interprets a number as a INR amount",
        expected_type = ValueType::Numeric,
        handler = &|input: Value| {
            let input = input.as_a::<Fixed>()?;
            Ok(CurrencyInner::as_rupees(input).to_string())
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
    fn test_currency() {
        test_decorator!("inr", Value::from(123), "₹123.00");
        test_decorator!("rub", Value::from(123), "₽123.00");
        test_decorator!("cny", Value::from(123), "元123.00");
        test_decorator!("jpy", Value::from(123), "¥123.00");
        test_decorator!("gbp", Value::from(123), "£123.00");
        test_decorator!("aud", Value::from(123), "$123.00");
        test_decorator!("cad", Value::from(123), "$123.00");
        test_decorator!("eur", Value::from(123), "€123.00");
        test_decorator!("usd", Value::from(123), "$123.00");
    }
}
