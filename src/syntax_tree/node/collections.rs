//! Compound value nodes
//!
//! Nodes mapping to array, object, range, etc
//!
use super::*;
use crate::{define_prattnode, error::WrapExternalError, oops, Rule, State, ToToken};
use pest::iterators::Pair;
use polyvalue::{operations::IndexingOperationExt, Value, ValueType};

define_node!(
    Array { elements: Vec<Node> },
    rules = [ARRAY_TERM],
    new = |input: Pair<Rule>| {
        let token = input.to_token();
        let mut children = input.into_inner();
        children.next(); // Skip the array term


        let elements = children.map(|c| c.to_ast_node()).collect::<Result<_, _>>()?;
        Ok(Self { elements, token }.boxed())
    },
    value = |this: &Array, state: &mut State| {
        let elements = this.elements.iter().map(|element| element.get_value(state)).collect::<Result<Vec<_>, _>>()?;
        Ok(Value::array(elements))
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
);

define_node!(
    Object {
        pairs: Vec<(Node, Node)>
    },
    rules = [OBJECT_TERM],
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
    value = |this: &Object, state: &mut State| {
        let values = this.pairs.iter()
            .map(|(key, value)| Ok::<(_, _), Error>((key.get_value(state)?, value.get_value(state)?)))
            .collect::<Result<Vec<(_, _)>, _>>()?;
        Value::try_from(values).with_context(this.token())
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
);

define_prattnode!(
    Range {
        start: Node,
        end: Node
    },
    rules = [OP_RANGE],
    new = |input: PrattPair| {
        let token = input.as_token();
        let mut children = input.into_inner();
        let start = children.next().unwrap().to_ast_node()?;
        children.next(); // Skip the range operator
        let end = children.next().unwrap().to_ast_node()?;

        Ok(Self { start, end, token }.boxed())
    },
    value = |this: &Self, state: &mut State| {
        let start = this.start.get_value(state)?;
        let end = this.end.get_value(state)?;

        let (start, end) = start.resolve(&end)?;
        match start.own_type() {
            ValueType::String => {
                let start = start.as_a::<String>().unwrap();
                let end = end.as_a::<String>().unwrap();
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

            ValueType::Bool | _ if start.is_a(ValueType::Int) => {
                let start = start.as_a::<i64>().unwrap();
                let end = end.as_a::<i64>().unwrap();

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
);

define_prattnode!(
    IndexingExpression {
        target: Node,
        indices: Vec<Option<Node>>
    },
    rules = [POSTFIX_INDEX],

    new = |input: PrattPair| {
        let token = input.as_token();
        let mut children = input.into_inner();
        let target = children.next().unwrap().to_ast_node()?;
        let mut indices = Vec::new();
        let children = children.next().unwrap();
        let mut children = children.first_pair().into_inner();
        while let Some(index) = children.next() {
            if index.as_rule() == Rule::POSTFIX_EMPTYINDEX {
                indices.push(None);
            } else {
                indices.push(Some(index.to_ast_node()?));
            }
        }

        Ok(Self { target, indices, token }.boxed())
    },

    value = |this: &IndexingExpression, state: &mut State| {
        let base = this.target.get_value(state)?;
        let mut result = base;
        for index in &this.indices {
            let index = if let Some(index) = index {
                index.get_value(state)?
            } else {
                (result.len()-1).into()
            };

            if index.is_a(ValueType::Collection) && !index.is_a(ValueType::String) {
                result = result.get_indices(&index).with_context(this.token())?;
            } else {
                result = result.get_index(&index).with_context(this.token())?;
            }
        }

        Ok(result)
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
);
