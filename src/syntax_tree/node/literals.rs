//! Literal Nodes
//!
//! Nodes mapping to primitive types
//!
use std::str::FromStr;

use super::*;
use crate::{
    define_prattnode,
    error::{ErrorDetails, WrapExternalError},
    oops, Rule, State, ToToken,
};
use pest::iterators::Pair;
use polyvalue::{types::*, Value, ValueTrait, ValueType};

define_prattnode!(
    DecoratorExpression {
        expression: Node,
        decorator: String
    },
    rules = [POSTFIX_DECORATE],
    new = |input: PrattPair| {
        let token = input.as_token();
        let mut children = input.into_inner();
        let expression = children.next().unwrap().to_ast_node()?;
        let decorator = children.next().unwrap().first_pair().into_inner().next().unwrap().as_str().to_string();

        Ok(Self {
            expression,
            decorator,
            token: token,
        }
        .boxed())
    },
    value = |this: &Self, state: &mut State| {
        let value = this.expression.get_value(state)?;
        let result = state.decorate(&this.decorator, value).with_context(this.token())?;
        Ok(Value::from(result))
    },

    docs = {
        name: "Decorator",
        symbols = ["@name"],
        description: "
            Converts a value to a formatted string.
            It calls a function named '@name' with the value as an argument.
        ",
        examples: "
            assert_eq(
                5 @float,
                '5.0'
            )
        ",
    }
);

define_prattnode!(
    CastExpression {
        expression: Node,
        target: Node
    },
    rules = [OP_CAST],
    new = |input: PrattPair| {
        let token = input.as_token();
        let mut children = input.into_inner();
        let expression = children.next().unwrap().to_ast_node()?;
        children.next(); // Skip the `as` keyword
        let target = children.next().unwrap().to_ast_node()?;

        Ok(Self {
            expression,
            target,
            token: token,
        }
        .boxed())
    },
    value = |this: &Self, state: &mut State| {
        let value = this.expression.get_value(state)?;

        let target = if this.target.token().rule == Rule::identifier {
            this.target.token().input.to_string()
        } else {
            this.target.get_value(state)?.to_string()
        };

        let target = ValueType::try_from(target.as_str()).with_context(&this.token())?;
        value.as_type(target).with_context(this.token())
    },

    docs = {
        name: "Cast",
        symbols = ["as"],
        description: "
            Casts a value to a different type.
            The type can be a string or an identifier.
            The operator is right-associative
        ",
        examples: "
            5 as float
            5 as 'float'
            5 as i8
        ",
    }
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
    value = |this: &Self, state: &mut State| {
        state
            .get_variable(&this.name)
            .ok_or(ErrorDetails::VariableName {
                name: this.name.clone(),
            })
            .with_context(this.token())
    },

    docs = {
        name: "Identifier",
        symbols = ["a", "b", "c"],
        description: "
            A variable name.
            The value of the variable is looked up in the state.
        ",
        examples: "
            [a, b, c] = [1, 2, 3]
            a; b; c
        ",
    }
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
                return oops!(Internal {
                    msg: format!("Unexpected const literal {text}")
                });
            }
        };

        Ok(Self { value, token }.boxed())
    },
    value = |this: &Self, _state: &mut State| { Ok(this.value.clone()) },

    docs = {
        name: "Constant",
        symbols = ["pi", "e", "tau"],
        description: "
            A constant value.
            A predefined set of values that are always available.
        ",
        examples: "
            pi; e; tau
        ",
    }
);

define_node!(
    ValueLiteral { value: Value },
    rules = [
        currency_literal,
        fixed_literal,
        sci_literal,
        float_literal,
        bool_literal,
        regex_literal,
        string_literal,
        int_literal
    ],
    new = |input: Pair<Rule>| {
        let token = input.to_token();
        let value = match input.as_rule() {
            Rule::int_literal => {
                let mut children = input.into_inner();
                let str = children.next().unwrap().as_str();
                let size = children
                    .next()
                    .map(|v| v.as_rule())
                    .unwrap_or(Rule::intsize_i64);

                match size {
                    Rule::intsize_i64 => I64::from_str(str).with_context(&token)?.into(),
                    Rule::intsize_i32 => I32::from_str(str).with_context(&token)?.into(),
                    Rule::intsize_i16 => I16::from_str(str).with_context(&token)?.into(),
                    Rule::intsize_i8 => I8::from_str(str).with_context(&token)?.into(),
                    Rule::intsize_u64 => U64::from_str(str).with_context(&token)?.into(),
                    Rule::intsize_u32 => U32::from_str(str).with_context(&token)?.into(),
                    Rule::intsize_u16 => U16::from_str(str).with_context(&token)?.into(),
                    Rule::intsize_u8 => U8::from_str(str).with_context(&token)?.into(),
                    _ => {
                        return oops!(
                            Internal {
                                msg: format!("Unexpected int size `{size:?}`")
                            },
                            token.clone()
                        );
                    }
                }
            }
            Rule::float_literal | Rule::sci_literal => {
                Float::from_str(input.as_str()).with_context(&token)?.into()
            }

            Rule::string_literal => parse_string(input.as_str()).into(),
            Rule::bool_literal => Bool::from_str(input.as_str()).with_context(&token)?.into(),

            Rule::regex_literal => Str::new(input.as_str().to_string()).into(),

            Rule::fixed_literal => {
                // remove the D suffix first
                let input = &input.as_str()[..input.as_str().len() - 1];
                Fixed::from_str(input).with_context(&token)?.into()
            }
            Rule::currency_literal => Currency::from_str(input.as_str())
                .with_context(&token)?
                .into(),

            _ => {
                return oops!(
                    Internal {
                        msg: format!("Unexpected value literal `{}`", input.as_str())
                    },
                    token
                );
            }
        };

        Ok(Self { value, token }.boxed())
    },
    value = |this: &Self, _state: &mut State| { Ok(this.value.clone()) }
);

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
