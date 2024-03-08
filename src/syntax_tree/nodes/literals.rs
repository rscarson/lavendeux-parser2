use super::{Node, Token};
use crate::{error::WrapExternalError, Rule};
use polyvalue::{types::*, Value};
use std::str::FromStr;

define_handler!(
    IntLiteral(pairs, token, _state) {
        let literal = unwrap_next!(pairs, token);
        let str = literal.as_str();
        let size = pairs
            .next()
            .map(|v| v.as_rule());

        let value = match size {
            Some(Rule::intsize_i32) => I32::from_str(str).with_context(&token)?.into(),
            Some(Rule::intsize_i16) => I16::from_str(str).with_context(&token)?.into(),
            Some(Rule::intsize_i8) => I8::from_str(str).with_context(&token)?.into(),
            Some(Rule::intsize_u64) => U64::from_str(str).with_context(&token)?.into(),
            Some(Rule::intsize_u32) => U32::from_str(str).with_context(&token)?.into(),
            Some(Rule::intsize_u16) => U16::from_str(str).with_context(&token)?.into(),
            Some(Rule::intsize_u8) => U8::from_str(str).with_context(&token)?.into(),
            _ => I64::from_str(str).with_context(&token)?.into()
        };

        Ok(Node::Literal(value, token))
    }
);

define_handler!(
    FloatLiteral(_pairs, token, _state) {
        let value: Value = Float::from_str(&token.input).with_context(&token)?.into();
        Ok(Node::Literal(value, token))
    }
);

define_handler!(
    StringLiteral(_pairs, token, _state) {
        // Remove the first and last characters - the quotes around our string
        // This would not work great with graphemes like é, but we know that it's
        // either ' or " so this should be safe
        let mut c = token.input.chars();
        c.next();
        c.next_back();

        // Now we split along our \\ backslash escapes, and rejoin after
        // to prevent going over them twice. This method isn't super
        // neat, there's likely a better way
        let mut out = String::new();
        let mut await_escape = false;
        for char in c {
            match char {
                '\\' => {
                    if await_escape {
                        out.push('\\');
                        await_escape = false;
                    } else {
                        await_escape = true;
                    }
                }
                _ => {
                    if await_escape {
                        out.push(match char {
                            '\'' => '\'',
                            '"' => '"',
                            'n' => '\n',
                            'r' => '\r',
                            't' => '\t',
                            _ => char,
                        });
                        await_escape = false;
                    } else {
                        out.push(char);
                    }
                }
            }
        }

        Ok(Node::Literal(Value::string(out), token))
    }
);

define_handler!(
    BoolLiteral(_pairs, token, _state) {
        let value = Bool::from_str(&token.input).with_context(&token)?;
        Ok(Node::Literal(Value::bool(value), token))
    }
);

define_handler!(
    RegexLiteral(_pairs, token, _state) {
        Ok(Node::Literal(Value::string(token.input.as_ref()), token))
    }
);

define_handler!(
    FixedLiteral(_pairs, token, _state) {
        // remove the D suffix first
        let input = &token.input[..token.input.len() - 1];

        let value = Fixed::from_str(input).with_context(&token)?;
        Ok(Node::Literal(Value::fixed(value), token))
    }
);

define_handler!(
    CurrencyLiteral(_pairs, token, _state) {
        let value = Currency::from_str(&token.input).with_context(&token)?;
        Ok(Node::Literal(Value::currency(value), token))
    }
);

define_handler!(
    ConstLiteral(_pairs, token, _state) {
        let value = match token.input.as_ref() {
            "pi" => Value::from(std::f64::consts::PI),
            "e" => Value::from(std::f64::consts::E),
            "tau" => Value::from(std::f64::consts::TAU),
            "nil" => Value::from(false),

            _ => {
                return oops!(Internal {
                    msg: format!("Unexpected const literal")
                }, token);
            }
        };
        Ok(Node::Literal(value, token))
    }
);
document_operator!(
    name = "Constants",
    rules = [],
    symbols = ["pi", "e", "tau", "nil"],
    description = "
        A constant value.
        A predefined set of values that are always available.

        - `pi` - The mathematical constant π
        - `e` - The mathematical constant e
        - `tau` - The mathematical constant τ
        - `nil` - The nil value - used to represent nothing or an empty value, especially in the context of a side-effect conditional
    ",
    examples = "
        pi; e; tau; nil
    ",
);
