use super::{values::Reference, Node};
use crate::{
    error::{ErrorDetails, WrapExternalError, WrapOption},
    syntax_tree::{assignment_target::AssignmentTarget, traits::IntoNode},
    Error, Rule,
};
use polyvalue::{Value, ValueType};

define_ast!(
    Collections {
        Array(elements: Vec<Node<'i>>) {
            build = (pairs, token, state) {
                pairs.next(); // Skip the bracket
                let elements = pairs
                    .map(|pair| pair.into_node(state))
                    .collect::<Result<Vec<_>, _>>().with_context(&token)?;
                Ok(Self { elements, token }.into())
            },
            eval = (this, state) {
                let elements = this
                    .elements
                    .iter()
                    .map(|element| element.evaluate(state))
                    .collect::<Result<Vec<_>, _>>().with_context(this.token())?;
                Ok(Value::array(elements))
            },
            owned = (this) {
                Self::Owned {
                    elements: this.elements.into_iter().map(|e| e.into_owned()).collect(),
                    token: this.token.into_owned(),
                }
            },

            docs = {
                name: "Array Literals",
                symbols = ["[ a, b, ... ]"],
                description: "
                    A collection of values.
                    Arrays can contain any type of value, including other arrays.
                    Arrays are 0-indexed, meaning the first element is at index 0.
                    The indexing operator (a[b]) can be used to access elements of an array.
                ",
                examples: "
                    [1, 2, 3, 4, 5]
                    [\"Hello\", \"World\"]
                    [1, [2, 3], 4]
                ",
            }
        },

        Object(entries: Vec<(Node<'i>, Node<'i>)>) {
            build = (pairs, token, state) {
                let mut entries: Vec<(_, _)> = Vec::new();
                while let Some(key) = pairs.next() {
                    let key = key.into_node(state).with_context(&token)?;
                    let value = unwrap_node!(pairs, state, token)?;
                    entries.push((key, value));
                }

                Ok(Self { entries, token }.into())
            },
            eval = (this, state) {
                let values = this.entries.iter()
                    .map(|(key, value)| Ok::<(_, _), Error>((key.evaluate(state).with_context(this.token())?, value.evaluate(state).with_context(this.token())?)))
                    .collect::<Result<Vec<(_, _)>, _>>().with_context(this.token())?;
                Value::try_from(values).with_context(this.token())
            },
            owned = (this) {
                Self::Owned {
                    entries: this.entries.into_iter().map(|(k, v)| (k.into_owned(), v.into_owned())).collect(),
                    token: this.token.into_owned(),
                }
            },

            docs = {
                name: "Object Literals",
                symbols = ["{ key: value, ... }"],
                description: "
                    A collection of key-value pairs.
                    Values can contain any type, including other objects.
                    Keys can be any non-collection type
                    The indexing operator (a[b]) can be used to access elements of an object.
                ",
                examples: "
                    { \"name\": \"John\", \"age\": 25 }
                    { \"name\": \"John\", \"address\": { \"city\": \"New York\", \"state\": \"NY\" } }
                ",
            }
        },

        Range(
            start: Node<'i>,
            end: Node<'i>
        ) {
            build = (pairs, token, state) {
                let start = unwrap_node!(pairs, state, token)?;
                pairs.next(); // Skip the '..'
                let end = unwrap_node!(pairs, state, token)?;
                Ok(Self { start, end, token }.into())
            },

            eval = (this, state) {
                let start = this.start.evaluate(state).with_context(this.token())?;
                let end = this.end.evaluate(state).with_context(this.token())?;

                let (start, end) = start.resolve(end).with_context(this.token())?;
                match start.own_type() {
                    ValueType::String => {
                        let start = start.as_a::<String>()?;
                        let end = end.as_a::<String>()?;
                        if start.len() != 1 || end.len() != 1 {
                            return oops!(
                                InvalidRange {
                                    start: start.to_string(),
                                    end: end.to_string()
                                },
                                this.token.clone()
                            );
                        }

                        let start = start.chars().next().unwrap();
                        let end = end.chars().next().unwrap();

                        if start > end {
                            return oops!(
                                RangeStartGT {
                                    start: start.to_string(),
                                    end: end.to_string()
                                },
                                this.token.clone()
                            );
                        }

                        // as array spanning the range inclusively
                        let array = (start..=end)
                            .map(|i| Value::from(i.to_string()))
                            .collect::<Vec<_>>();
                        Ok(Value::from(array))
                    }

                    _ if start.is_a(ValueType::Int) => {
                        let start = start.as_a::<i64>()?;
                        let end = end.as_a::<i64>()?;

                        if start > end {
                            return oops!(
                                RangeStartGT {
                                    start: start.to_string(),
                                    end: end.to_string()
                                },
                                this.token.clone()
                            );
                        }

                        Ok(Value::range(start..=end))
                    }

                    _ => {
                        oops!(RangeTypeMismatch, this.token.clone())
                    }
                }
            },

            owned = (this) {
                Self::Owned {
                    start: this.start.into_owned(),
                    end: this.end.into_owned(),
                    token: this.token.into_owned(),
                }
            },

            docs = {
                name: "Range Literals",
                symbols = ["first..last"],
                description: "
                    A range of values.
                    Ranges can be used to create arrays of numbers or characters.
                    Ranges are inclusive, meaning the start and end values are included in the array.
                    Start and end values must be of the same type, and start must be <= end.
                    Character ranges are inclusive and can only be used with single-character strings.
                ",
                examples: "
                    1..5
                    'a'..'z'
                ",
            }
        },

        IndexingExpression(base: Node<'i>, indices: Vec<Option<Node<'i>>>) {
            build = (pairs, token, state) {
                let base = unwrap_node!(pairs, state, token)?;
                let indices = unwrap_next!(pairs, token);
                let indices = indices
                    .map(|pair| {
                    if pair.as_rule() == Rule::POSTFIX_EMPTYINDEX {
                        Ok::<_, Error>(None)
                    } else {
                        Ok(Some(pair.into_node(state).with_context(&token)?))
                    }
                })
                .collect::<Result<Vec<_>, _>>().with_context(&token)?;

                let is_reference = match base {
                    Node::Values(ref node) => matches!(&**node, super::Values::Reference(_)),
                    _ => false,
                };

                if is_reference {
                    let target = as_reference!(base).or_error(ErrorDetails::ConstantValue).with_context(&token)?;
                    Ok(Reference::new(AssignmentTarget::Index(target.to_string(), indices), token).into())
                } else {
                    Ok(Self { base, indices, token }.into())
                }
            },

            eval = (this, state) {
                let base = this.base.evaluate(state).with_context(this.token())?;
                let indices = this
                    .indices
                    .iter()
                    .map(|index| {
                        if let Some(index) = index {
                            Ok::<_, Error>(Some(index.evaluate(state).with_context(this.token())?))
                        } else {
                            Ok(None)
                        }
                    })
                    .collect::<Result<Vec<_>, _>>().with_context(this.token())?;
                let value = AssignmentTarget::get_index_handle(base, &indices).with_context(this.token())?;
                Ok(value.clone())
            },

            owned = (this) {
                Self::Owned {
                    base: this.base.into_owned(),
                    indices: this.indices.into_iter().map(|i| i.map(|i| i.into_owned())).collect(),
                    token: this.token.into_owned(),
                }
            },

            docs = {
                name: "Indexing",
                symbols = ["a[b]", "a[]"],
                description: "
                    Accessing elements of a collection.
                    The indexing operator can be used to access elements of a collection or string.
                    If the index is a collection, it is used to access multiple elements.
                    If the index is a string, it is used to access a character.
                    If the index is blank, it is used to access the last element of the collection.
                    Negative indices can be used to access elements from the end of the collection.
                ",
                examples: "
                    [1, 2, 3][0]
                    [1, 2, 3][0..1]
                    { \"name\": \"John\", \"age\": 25 }[\"name\"]
                ",
            }
        }
    }
);
