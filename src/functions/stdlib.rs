use super::std_function::ParserFunction;
use std::collections::HashMap;

mod bitwise;
mod collections;
mod dev;
mod math;
mod string;
mod system;
mod trig;

#[cfg(feature = "crypto-functions")]
mod random;

mod decorators_currency;
mod decorators_numeric;
mod decorators_types;

#[cfg(feature = "network-functions")]
mod network;

inventory::collect!(&'static dyn ParserFunction);
/// Returns a map of all standard library functions
/// Used by the state to load stdlib
pub fn all() -> HashMap<String, Box<dyn ParserFunction>> {
    inventory::iter::<&'static dyn ParserFunction>
        .into_iter()
        .map(|f| (f.name().to_string(), f.clone_self()))
        .collect()
}

#[cfg(test)]
mod test {
    use crate::{error::ErrorDetails, Error};

    use super::*;

    #[test]
    fn test_stdlib_documentation() {
        let mut parser = crate::Lavendeux::new(Default::default());
        let stdlib = all();

        let mut errors = vec![];

        for (name, function) in stdlib {
            let examples = function.documentation().examples().unwrap();
            let skip_example = examples.starts_with("#skip");
            let examples = examples.trim_start_matches("#skip").trim();
            if examples.is_empty() {
                errors.push(Error {
                    details: ErrorDetails::Custom {
                        msg: format!(
                            "No examples for function {}::{name}",
                            function.documentation().category()
                        ),
                    },
                    source: None,
                    context: None,
                });
                continue;
            }

            if skip_example {
                continue;
            }

            match parser.parse(examples) {
                Ok(_) => {}
                Err(e) => {
                    errors.push(Error {
                        details: ErrorDetails::Custom {
                            msg: format!(
                                "Failed to parse example for function {}::{name}",
                                function.documentation().category()
                            ),
                        },
                        source: Some(Box::new(e)),
                        context: None,
                    });
                }
            }
        }

        for e in errors.iter() {
            eprintln!("\n{}\n", e);
        }

        assert!(
            errors.is_empty(),
            "Some documentation tests failed. See output for details."
        );
    }
}
