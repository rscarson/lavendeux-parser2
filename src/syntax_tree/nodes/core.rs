use crate::{
    error::WrapExternalError,
    syntax_tree::{pratt, traits::IntoNode},
    Token,
};

use super::Node;
use polyvalue::Value;

define_ast!(
    Core {
        Script(statements: Vec<Node<'i>>) {
            build = (pairs, token, state) {
                let statements = pairs
                    .map(|pair| pair.into_node(state))
                    .collect::<Result<Vec<_>, _>>()?;
                let node = Self { statements, token };
                Ok(node.into())
            },
            eval = (this, state) {
                Ok(
                    Value::array(
                        this.statements
                            .iter()
                            .map(|s| s.evaluate(state))
                            .collect::<Result<Vec<_>, _>>()?
                    )
                )
            },
            owned = (this) {
                Self::Owned {
                    statements: this.statements
                        .into_iter()
                        .map(|s| s.into_owned())
                        .collect(),
                    token: this.token.into_owned(),
                }
            },
            docs = {
                name: "Script",
                symbols = ["<statement> [ ; | \\n ] <statement>"],
                description: "
                    A series of expressions or function definitions that are executed in order, and are separated by semicolons or linebreaks.
                ",
                examples: "
                    1 + 2 ; 3 @hex
                    min([1, 2, 3])
                ",
            }
        },

        Block(statements: Vec<Node<'i>>) {
            build = (pairs, token, state) {
                let statements = pairs
                    .map(|pair| pair.into_node(state))
                    .collect::<Result<Vec<_>, _>>()?;

                if statements.len() == 0 {
                    return oops!(EmptyBlock, token);
                }

                let node = Self { statements, token };
                Ok(node.into())
            },
            eval = (this, state) {
                let mut value = None;
                for statement in &this.statements {
                    value = Some(statement.evaluate(state)?);
                }
                Ok(value.unwrap_or_else(|| Value::from(false)))
            },
            owned = (this) {
                Self::Owned {
                    statements: this.statements
                        .into_iter()
                        .map(|s| s.into_owned())
                        .collect(),
                    token: this.token.into_owned(),
                }
            },
            docs = {
                name: "Block",
                symbols = ["{ <statements> }"],
                description: "
                    A series of expressions that are executed in order, and are separated by semicolons or linebreaks.
                    The last statement's value is returned.
                    A block must return a value, and thus cannot be empty.
                    If a block doesn't need to return a value (side-effect `if`s for example), use `nil`
                ",
                examples: "
                    if true {
                        1; 2
                    } else nil
                ",
            }
        }
    }
);
