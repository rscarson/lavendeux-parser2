use crate::pest::Rule;
pub struct OperatorDocumentation {
    pub rules: &'static [Rule],
    pub name: &'static str,
    pub symbols: &'static [&'static str],

    pub description: &'static str,
    pub examples: &'static str,
}

inventory::collect!(OperatorDocumentation);
pub fn all() -> Vec<&'static OperatorDocumentation> {
    let mut all: Vec<_> = inventory::iter::<OperatorDocumentation>
        .into_iter()
        .collect();
    all.sort_by(|a, b| a.name.cmp(b.name));
    all
}

/// Generates a documentation entry for an operator
macro_rules! document_operator {
    (
        name = $name:literal,
        rules = [$($rule:ident),*],
        symbols = [$($symbol:literal),*],
        description = $description:literal,
        examples = $examples:literal$(,)?
    ) => {
        inventory::submit! {
            crate::documentation::OperatorDocumentation {
                name: $name,
                rules: &[$(Rule::$rule),*],
                symbols: &[$($symbol),*],
                description: indoc::indoc! { $description },
                examples: indoc::indoc! { $examples }
            }
        }
    };
}

#[cfg(test)]
mod test {
    use crate::{error::ErrorDetails, Error};

    use super::*;

    #[test]
    fn test_all_rules_documented() {
        let docs = all();
        let mut errors = vec![];

        for operator in docs {
            let result = crate::Lavendeux::new(Default::default()).parse(operator.examples);
            if let Err(e) = result {
                errors.push(Error {
                    details: ErrorDetails::Custom {
                        msg: format!("{} Example Error", operator.name),
                    },
                    source: Some(Box::new(e)),
                    context: None,
                });
            }
        }
        for error in &errors {
            eprintln!("{}\n", error);
        }

        assert!(
            errors.is_empty(),
            "{} errors found in operator documentation",
            errors.len()
        );
    }
}
