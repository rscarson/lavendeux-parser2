use crate::define_stddecorator;
use polyvalue::types::{CurrencyInner, Fixed};

define_stddecorator!(
    usd { input: Numeric },
    docs = {
        description: "Interprets a number as a USD amount",
        ext_description: "Includes a dollar sign and two decimal places.",
        examples: "
            assert_eq(
                100 @usd,
                '$100.00'
            )
        "
    },
    handler = (input) {
        let input = input.as_a::<Fixed>()?;
        Ok(CurrencyInner::as_dollars(input).to_string())
    }
);

define_stddecorator!(
    eur { input: Numeric },
    docs = {
        description: "Interprets a number as a Euro amount",
        ext_description: "Includes a euro sign and two decimal places.",
        examples: "
            assert_eq(
                100 @eur,
                '€100.00'
            )
        "
    },
    handler = (input) {
        let input = input.as_a::<Fixed>()?;
        Ok(CurrencyInner::as_euros(input).to_string())
    }
);

define_stddecorator!(
    cad { input: Numeric },
    docs = {
        description: "Interprets a number as a CAD amount",
        ext_description: "Includes a dollar sign and two decimal places.",
        examples: "
            assert_eq(
                100 @cad,
                '$100.00'
            )
        "
    },
    handler = (input) {
        let input = input.as_a::<Fixed>()?;
        Ok(CurrencyInner::as_dollars(input).to_string())
    }
);

define_stddecorator!(
    aud { input: Numeric },
    docs = {
        description: "Interprets a number as a AUD amount",
        ext_description: "Includes a dollar sign and two decimal places.",
        examples: "
            assert_eq(
                100 @aud,
                '$100.00'
            )
        "
    },
    handler = (input) {
        let input = input.as_a::<Fixed>()?;
        Ok(CurrencyInner::as_dollars(input).to_string())
    }
);

define_stddecorator!(
    gbp { input: Numeric },
    docs = {
        description: "Interprets a number as a GBP amount",
        ext_description: "Includes a pound sign and two decimal places.",
        examples: "
            assert_eq(
                100 @gbp,
                '£100.00'
            )
        "
    },
    handler = (input) {
        let input = input.as_a::<Fixed>()?;
        Ok(CurrencyInner::as_pounds(input).to_string())
    }
);

define_stddecorator!(
    jpy { input: Numeric },
    docs = {
        description: "Interprets a number as a JPY amount",
        ext_description: "Includes a yen sign and no decimal places.",
        examples: "
            assert_eq(
                100 @jpy,
                '¥100'
            )
        "
    },
    handler = (input) {
        let input = input.as_a::<Fixed>()?;
        Ok(CurrencyInner::as_yen(input).to_string())
    }
);

define_stddecorator!(
    cny { input: Numeric },
    docs = {
        description: "Interprets a number as a CNY amount",
        ext_description: "Includes a yuan sign and two decimal places.",
        examples: "
            assert_eq(
                100 @cny,
                '¥100.00'
            )
        "
    },
    handler = (input) {
        let input = input.as_a::<Fixed>()?;
        Ok(CurrencyInner::as_yuan(input).to_string())
    }
);

define_stddecorator!(
    inr { input: Numeric },
    docs = {
        description: "Interprets a number as a INR amount",
        ext_description: "Includes a rupee sign and two decimal places.",
        examples: "
            assert_eq(
                100 @inr,
                '₹100.00'
            )
        "
    },
    handler = (input) {
        let input = input.as_a::<Fixed>()?;
        Ok(CurrencyInner::as_rupees(input).to_string())
    }
);

define_stddecorator!(
    rub { input: Numeric },
    docs = {
        description: "Interprets a number as a RUB amount",
        ext_description: "Includes a ruble sign and two decimal places.",
        examples: "
            assert_eq(
                100 @rub,
                '₽100.00'
            )
        "
    },
    handler = (input) {
        let input = input.as_a::<Fixed>()?;
        Ok(CurrencyInner::as_rubles(input).to_string())
    }
);
