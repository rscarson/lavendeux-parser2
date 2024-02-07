use crate::std_functions::Function;
use std::collections::HashMap;

mod currency;
mod numeric;
mod types;

pub fn register_all(map: &mut HashMap<String, Function>) {
    types::register_all(map);
    numeric::register_all(map);
    currency::register_all(map);
}

#[cfg(test)]
#[macro_export]
macro_rules! test_decorator {
    ($name:literal, $input:expr, $expected:expr) => {
        assert_eq!(
            state::State::new()
                .decorate($name, &Token::dummy(), $input)
                .unwrap(),
            $expected
        );
    };
    () => {};
}
