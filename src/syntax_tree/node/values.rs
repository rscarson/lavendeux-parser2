//! Value Nodes
//!
//! High-level nodes that are used to build the syntax tree.
//! These nodes are how the user will interact with the syntax tree.
//!
use super::*;
use crate::error::WrapError;
use crate::{Error, Rule, State, ToToken, Value};
use pest::iterators::Pair;
use polyvalue::operations::IndexingMutationExt;
use polyvalue::{types::*, ValueTrait, ValueType};
use std::str::FromStr;

fn parse_string(input: &str) -> String {
    // Sanity check
    if input.len() < 2 {
        return String::new();
    }

    // Remove the first and last characters - the quotes around our string
    // This would not work great with graphemes like é, but we know that it's
    // either ' or " so this should be safe
    let mut c = input.chars();
    c.next();
    c.next_back();

    // Now we split along our \\ backslash escapes, and rejoin after
    // to prevent going over them twice. This method isn't super
    // neat, there's likely a better way
    c.as_str()
        .split("\\\\")
        .map(|s| {
            s.replace("\\'", "\'")
                .replace("\\\"", "\"")
                .replace("\\n", "\n")
                .replace("\\\n", "\n")
                .replace("\\\r", "\r")
                .replace("\\r", "\r")
                .replace("\\t", "\t")
        })
        .collect::<Vec<String>>()
        .join("\\")
}

define_node!(
    ValueLiteral { value: Value },
    rules = [
        currency_literal,
        fixed_literal,
        sci_literal,
        float_literal,
        bool_literal,
        regex_literal,
        sized_literal,
        string_literal
    ],
    new = |input: Pair<Rule>| {
        let token = input.to_token();
        let value = match input.as_rule() {
            Rule::int_literal => I64::from_str(input.as_str()).to_error(&token)?.into(),
            Rule::float_literal | Rule::sci_literal => {
                Float::from_str(input.as_str()).to_error(&token)?.into()
            }

            Rule::hex_literal | Rule::bin_literal | Rule::oct_literal => {
                I64::from_str_radix(input.as_str()).to_error(&token)?.into()
            }

            Rule::string_literal => parse_string(input.as_str()).into(),
            Rule::bool_literal => Bool::from_str(input.as_str()).to_error(&token)?.into(),

            Rule::regex_literal => Str::new(input.as_str().to_string()).into(),

            Rule::sized_literal => {
                let input = input.as_str();

                if let Some(integer) = input.strip_suffix("u8") {
                    polyvalue::types::U8::from_str(integer)
                        .to_error(&token)?
                        .into()
                } else if let Some(integer) = input.strip_suffix("u16") {
                    polyvalue::types::U16::from_str(integer)
                        .to_error(&token)?
                        .into()
                } else if let Some(integer) = input.strip_suffix("u32") {
                    polyvalue::types::U32::from_str(integer)
                        .to_error(&token)?
                        .into()
                } else if let Some(integer) = input.strip_suffix("u64") {
                    polyvalue::types::U64::from_str(integer)
                        .to_error(&token)?
                        .into()
                } else if let Some(integer) = input.strip_suffix("i8") {
                    polyvalue::types::I8::from_str(integer)
                        .to_error(&token)?
                        .into()
                } else if let Some(integer) = input.strip_suffix("i16") {
                    polyvalue::types::I16::from_str(integer)
                        .to_error(&token)?
                        .into()
                } else if let Some(integer) = input.strip_suffix("i32") {
                    polyvalue::types::I32::from_str(integer)
                        .to_error(&token)?
                        .into()
                } else if let Some(integer) = input.strip_suffix("i64") {
                    polyvalue::types::I64::from_str(integer)
                        .to_error(&token)?
                        .into()
                } else {
                    polyvalue::types::I64::from_str(input)
                        .to_error(&token)?
                        .into()
                }
            }

            Rule::fixed_literal => {
                // remove the D suffix first
                let input = &input.as_str()[..input.as_str().len() - 1];
                Fixed::from_str(input).to_error(&token)?.into()
            }
            Rule::currency_literal => Currency::from_str(input.as_str()).to_error(&token)?.into(),

            _ => Err(Error::Internal(format!(
                "Unexpected rule {:?} in ValueLiteral",
                input.as_rule()
            )))?,
        };

        Ok(Self { value, token }.boxed())
    },
    value = |this: &ValueLiteral, _state: &mut State| { Ok(this.value.clone()) }
);

define_node!(
    ConstantValue { value: Value },
    rules = [const_literal],
    new = |input: Pair<Rule>| {
        let token = input.to_token();
        let text = input.as_str();

        let value = match text {
            "pi" => Value::from(std::f64::consts::PI),
            "e" => Value::from(std::f64::consts::E),
            "tau" => Value::from(std::f64::consts::TAU),

            _ => {
                return Err(Error::Internal(format!(
                    "Unexpected const literal {}",
                    text
                )));
            }
        };

        Ok(Self { value, token }.boxed())
    },
    value = |this: &ConstantValue, _state: &mut State| { Ok(this.value.clone()) }
);

define_node!(
    Identifier { name: String },
    rules = [identifier],
    new = |input: Pair<Rule>| {
        let mut token = input.to_token();
        let name = input.as_str().to_string();
        token.references = Some(name.clone());

        Ok(Self { name, token }.boxed())
    },
    value = |this: &Identifier, state: &mut State| {
        let value = state.get_variable(&this.name).ok_or(Error::VariableName {
            name: this.name.clone(),
            token: this.token.clone(),
        })?;
        Ok(value)
    }
);

define_node!(
    ArrayValue {
        elements: Vec<Node>
    },
    rules = [array_literal],

    new = |input: Pair<Rule>| {
        let token = input.to_token();
        let elements = input
            .into_inner()
            .map(|child| Ok(child.to_ast_node()?))
            .collect::<Result<Vec<Node>, Error>>()?;

        Ok(Self { elements, token }.boxed())
    },

    value = |this: &ArrayValue, state: &mut State| {
        let values = this.elements.iter().map(|node| node.get_value(state)).collect::<Result<Vec<_>, _>>()?;
        Ok(Value::Array(values.into()))
    }
);

define_node!(
    ObjectValue {
        pairs: Vec<(Node, Node)>
    },
    rules = [object_literal],

    new = |input: Pair<Rule>| {
        let token = input.to_token();
        let mut children = input.into_inner();

        let mut pairs: Vec<(Node, Node)> = Vec::new();
        while children.peek().is_some() {
            let key = children.next().unwrap().to_ast_node()?;
            let value = children.next().unwrap().to_ast_node()?;
            pairs.push((key, value));
        }

        Ok(Self { pairs, token }.boxed())
    },

    value = |this: &ObjectValue, state: &mut State| {
        let mut object = ObjectInner::new();
        for (key, value) in &this.pairs {
            let key = key.get_value(state)?;
            let value = value.get_value(state)?;
            object.insert(key, value).to_error(&this.token)?;
        }

        Ok(Value::Object(object.into()))
    }
);

define_node!(
    RangeValue {
        start: Node,
        end: Node
    },
    rules = [RANGE_EXPRESSION],
    new = |input: Pair<Rule>| {
        let token = input.to_token();
        let mut children = input.into_inner();

        let start = children
            .next()
            .ok_or(Error::IncompleteRangeExpression {
                token: token.clone(),
            })?
            .to_ast_node()?;

        children.next().unwrap(); // Skip the ..

        let end = children
            .next()
            .ok_or(Error::IncompleteRangeExpression {
                token: token.clone(),
            })?
            .to_ast_node()?;

        Ok(Self { start, end, token }.boxed())
    },
    value = |this: &RangeValue, state: &mut State| {
        let start = this.start.get_value(state)?;
        let end = this.end.get_value(state)?;

        // We will enforce 3 things here:
        // 1. The start and end must be the same type
        // 2. The start and end must be integer or single character strings
        // 3. The start must be less than or equal to the end
        match (&start, &end) {
            (Value::String(start), Value::String(end)) => {
                if start.inner().len() != 1 || end.inner().len() != 1 {
                    return Err(Error::InvalidRange {
                        start: start.to_string(),
                        end: end.to_string(),
                        token: this.token.clone(),
                    });
                }

                let start = start.inner().chars().next().unwrap();
                let end = end.inner().chars().next().unwrap();

                if start > end {
                    return Err(Error::InvalidRange {
                        start: start.to_string(),
                        end: end.to_string(),
                        token: this.token.clone(),
                    });
                }

                // as array spanning the range inclusively
                let array = (start..=end)
                    .map(|i| Value::from(i.to_string()))
                    .collect::<Vec<_>>();
                Ok(Value::from(array))
            }

            (Value::I64(start), Value::I64(end)) => {
                if start > end {
                    return Err(Error::InvalidRange {
                        start: start.to_string(),
                        end: end.to_string(),
                        token: this.token.clone(),
                    });
                }

                Ok(Value::from(*start.inner()..=*end.inner()))
            }

            _ => Err(Error::RangeTypeMismatch {
                token: this.token.clone(),
            }),
        }
    }
);

define_node!(
    CastingExpression {
        value: Node,
        target_type: ValueType
    },
    rules = [CASTING_EXPRESSION],
    new = |input: Pair<Rule>| {
        let token = input.to_token();
        let mut children = input.into_inner();

        let value = children.next().unwrap().to_ast_node()?;
        let target_type = children.next().unwrap().as_str();
        let target_type = ValueType::try_from(target_type).to_error(&token)?;

        Ok(Self {
            value,
            target_type,
            token,
        }
        .boxed())
    },
    value = |this: &CastingExpression, state: &mut State| {
        let value = this.value.get_value(state)?;
        Ok(value.as_type(this.target_type).to_error(&this.token)?)
    }
);

define_node!(
    DeleteExpression {
        src: String,
        indices: Vec<Node>
    },
    rules = [DELETE_EXPRESSION],

    new = |input: Pair<Rule>| {
        let token = input.to_token();
        let mut children = input.into_inner();

        children.next(); // Skip the delete keyword
        let src = children.next().unwrap().as_str().to_string();
        let indices = children
            .map(|child| Ok(child.to_ast_node()?))
            .collect::<Result<Vec<Node>, Error>>()?;

        Ok(Self {
            src,
            indices,
            token,
        }
        .boxed())
    },

    value = |this: &DeleteExpression, state: &mut State| {
        let mut indices = this.indices.iter().map(|i| i.get_value(state)).collect::<Result<Vec<_>, _>>()?;
        if let Some(final_idx) = indices.pop() {
            let mut value = state.get_variable(&this.src).ok_or(Error::VariableName {
                name: this.src.clone(),
                token: this.token.clone(),
            })?;

            let mut pos = &mut value;
            for index in &mut indices {
                pos = pos.get_index_mut(index).to_error(&this.token)?;
            }

            let removed = pos.delete_index(&final_idx).to_error(&this.token)?;
            state.set_variable(&this.src, value);
            Ok(removed)
        } else {
            if let Some(function) = state.delete_user_function(&this.src) {
                Ok(function.to_std_function().signature().into())
            } else {
                state.delete_variable(&this.src).ok_or(Error::VariableName {
                    name: this.src.clone(),
                    token: this.token.clone(),
                })
            }
        }
    }
);

#[cfg(test)]
mod test {
    use polyvalue::{
        types::{Object, I64},
        ValueTrait,
    };

    use super::*;
    use crate::assert_tree;

    macro_rules! assert_value {
        ($input:literal, $rule:ident, $expected:literal) => {
            assert_tree!($input, $rule, ValueLiteral, |tree: &mut ValueLiteral| {
                assert_eq!($expected, tree.value.to_string());
            });
        };
    }

    #[test]
    fn test_currency_literal() {
        assert_value!("$1", currency_literal, "$1");
        assert_value!("$1.20", currency_literal, "$1.20");
        assert_value!("1.20$", currency_literal, "$1.20");
    }

    #[test]
    fn test_sci_literal() {
        assert_value!("1e2", sci_literal, "100.0");
        assert_value!("1e-2", sci_literal, "0.01");
        assert_value!("1.2e2", sci_literal, "120.0");
        assert_value!("1.2e+2", sci_literal, "120.0");
    }

    #[test]
    fn test_float_literal() {
        assert_value!("1.2", float_literal, "1.2");
        assert_value!(".2", float_literal, "0.2");
    }

    #[test]
    fn test_hex_literal() {
        assert_value!("0x1", sized_literal, "1");
        assert_value!("0x1a", sized_literal, "26");
        assert_value!("0x1A", sized_literal, "26");
        assert_value!("0x1Aa", sized_literal, "426");
        assert_value!("0xA0", sized_literal, "160");
    }

    #[test]
    fn test_bin_literal() {
        assert_value!("0b1", sized_literal, "1");
        assert_value!("0b10", sized_literal, "2");
        assert_value!("0b101", sized_literal, "5");
    }

    #[test]
    fn test_oct_literal() {
        assert_value!("0o1", sized_literal, "1");
        assert_value!("0o10", sized_literal, "8");
        assert_value!("0o101", sized_literal, "65");
        assert_value!("0101", sized_literal, "65");
    }

    #[test]
    fn test_sized_literal() {
        assert_value!("1", sized_literal, "1");
        assert_value!("10", sized_literal, "10");
        assert_value!("100", sized_literal, "100");
        assert_value!("100_000", sized_literal, "100000");
    }

    #[test]
    fn test_bool_literal() {
        assert_value!("true", bool_literal, "true");
        assert_value!("false", bool_literal, "false");
        assert_value!("TRUe", bool_literal, "true");
    }

    #[test]
    fn test_string_literal() {
        assert_value!("\"\"", string_literal, "");
        assert_value!("\"hello\"", string_literal, "hello");
        assert_value!("\"hello world\"", string_literal, "hello world");
        assert_value!("\"hello \\\"world\\\"\"", string_literal, "hello \"world\"");
    }

    #[test]
    fn test_identifier() {
        assert_tree!("hello", identifier, Identifier, |tree: &mut Identifier| {
            assert_eq!("hello", tree.name);

            let state = &mut State::new();
            state.set_variable("hello", Value::from(1));
            let value = tree.get_value(state).unwrap();
            assert_eq!(1, *value.as_a::<I64>().unwrap().inner());
        });
    }

    #[test]
    fn test_array_literal() {
        assert_tree!(
            "[1, 2, 3]",
            array_literal,
            ArrayValue,
            |tree: &mut ArrayValue| {
                assert_eq!(3, tree.elements.len());
                let value = tree.get_value(&mut State::new()).unwrap();
                assert_eq!("[1, 2, 3]", value.to_string());
            }
        );
    }

    #[test]
    fn test_object_literal() {
        assert_tree!(
            "{'a': 1, 'b': 2, 3: 3}",
            object_literal,
            ObjectValue,
            |tree: &mut ObjectValue| {
                assert_eq!(3, tree.pairs.len());
                let value = tree.get_value(&mut State::new()).unwrap();
                let obj = value.as_a::<Object>().unwrap();
                assert!(obj.inner().contains_key(&Value::from("a")));
                assert!(obj.inner().contains_key(&Value::from("b")));
                assert!(obj.inner().contains_key(&Value::from(3i64)));
            }
        );
    }

    #[test]
    fn test_range_expression() {
        assert_tree!(
            "1..3",
            RANGE_EXPRESSION,
            RangeValue,
            |tree: &mut RangeValue| {
                let value = tree.get_value(&mut State::new()).unwrap();
                assert_eq!("1..3", value.to_string());
            }
        );

        assert_tree!(
            "'a'..'c'",
            RANGE_EXPRESSION,
            RangeValue,
            |tree: &mut RangeValue| {
                let value = tree.get_value(&mut State::new()).unwrap();
                assert_eq!("[a, b, c]", value.to_string());
            }
        );

        assert_tree!(
            "1..1",
            RANGE_EXPRESSION,
            RangeValue,
            |tree: &mut RangeValue| {
                let value = tree.get_value(&mut State::new()).unwrap();
                assert_eq!("1..1", value.to_string());
            }
        );

        assert_tree!(
            "'a'..'a'",
            RANGE_EXPRESSION,
            RangeValue,
            |tree: &mut RangeValue| {
                let value = tree.get_value(&mut State::new()).unwrap();
                assert_eq!("[a]", value.to_string());
            }
        );

        assert_tree!(
            "1..0",
            RANGE_EXPRESSION,
            RangeValue,
            |tree: &mut RangeValue| {
                let value = tree.get_value(&mut State::new());
                assert!(value.is_err());
            }
        );

        assert_tree!(
            "'c'..'a'",
            RANGE_EXPRESSION,
            RangeValue,
            |tree: &mut RangeValue| {
                let value = tree.get_value(&mut State::new());
                assert!(value.is_err());
            }
        );

        assert_tree!(
            "1..1.5",
            RANGE_EXPRESSION,
            RangeValue,
            |tree: &mut RangeValue| {
                let value = tree.get_value(&mut State::new());
                assert!(value.is_err());
            }
        );
    }

    #[test]
    fn test_delete() {
        let mut state = State::new();
        state.set_variable("a", Value::from(1));
        state.set_variable("b", Value::from(vec![Value::from(2)]));
        state.set_variable(
            "c",
            Value::from(vec![Value::from(2), Value::from(vec![Value::from(2)])]),
        );

        assert_tree!(
            "delete a",
            DELETE_EXPRESSION,
            DeleteExpression,
            |tree: &mut DeleteExpression| {
                let value = tree.get_value(&mut state).unwrap();
                assert_eq!(Value::from(1), value);
                assert!(state.get_variable("a").is_none());
            }
        );
        assert_tree!(
            "delete b[0]",
            DELETE_EXPRESSION,
            DeleteExpression,
            |tree: &mut DeleteExpression| {
                let value = tree.get_value(&mut state).unwrap();
                assert_eq!(Value::from(2), value);
                assert_eq!(state.get_variable("b").unwrap().to_string(), "[]");
            }
        );
        assert_tree!(
            "delete c[1][0]",
            DELETE_EXPRESSION,
            DeleteExpression,
            |tree: &mut DeleteExpression| {
                let value = tree.get_value(&mut state).unwrap();
                assert_eq!(Value::from(2), value);
                assert_eq!(state.get_variable("c").unwrap().to_string(), "[2, []]");
            }
        );
    }
}
