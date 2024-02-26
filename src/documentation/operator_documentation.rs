use crate::pest::Rule;
pub struct OperatorDocumentation {
    pub rules: &'static [Rule],
    pub name: &'static str,
    pub symbols: &'static [&'static str],

    pub description: &'static str,
    pub examples: &'static str,
}

#[macro_export]
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

inventory::collect!(OperatorDocumentation);
pub fn all() -> Vec<&'static OperatorDocumentation> {
    let mut all: Vec<_> = inventory::iter::<OperatorDocumentation>
        .into_iter()
        .collect();
    all.sort_by(|a, b| a.name.cmp(b.name));
    all
}

#[cfg(test)]
mod test {
    use crate::{error::ErrorDetails, syntax_tree, Error};

    use super::*;

    #[test]
    fn test_all_rules_documented() {
        // Meta rules that should not be documented
        let meta_rules = &[
            Rule::SCRIPT,
            Rule::STATEMENT,
            Rule::EXPR,
            Rule::BLOCK,
            //
            // Value literals are documented separately
            Rule::currency_literal,
            Rule::fixed_literal,
            Rule::sci_literal,
            Rule::float_literal,
            Rule::bool_literal,
            Rule::regex_literal,
            Rule::string_literal,
            Rule::int_literal,
            //
            // These are part of the meta for other rules
            Rule::BREAK_KEYWORD,
            Rule::SKIP_KEYWORD,
            Rule::RETURN_EXPRESSION,
        ];

        let docs = all();
        let mut errors = vec![];
        for rule in syntax_tree::resolver::all().keys() {
            let found = docs.iter().any(|d| d.rules.contains(rule));
            if !found && !meta_rules.contains(rule) {
                errors.push(Error {
                    details: ErrorDetails::Custom {
                        message: format!("Rule {:?} is not documented", rule),
                    },
                    source: None,
                    context: None,
                });
            }
        }

        for operator in docs {
            let result = crate::Lavendeux::new(Default::default()).parse(operator.examples);
            if let Err(e) = result {
                errors.push(Error {
                    details: ErrorDetails::Custom {
                        message: format!("{} Example Error", operator.name),
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
