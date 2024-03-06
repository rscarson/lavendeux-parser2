//! Core Node
//!
//! High-level nodes that are used to build the syntax tree.
//!
use super::*;
use crate::{syntax_tree::pratt, Error, ToToken};
use polyvalue::Value;

define_node!(
    Script { lines: Vec<Node<'i>> },
    rules = [SCRIPT],

    new = (input) {
        let token = input.to_token();
        let lines = input.into_inner().map(|child| child.to_ast_node()).collect::<Result<_, _>>()?;
        Ok(Self { lines, token }.boxed())
    },

    value = (this, state) {
        Ok(Value::array(this.lines.iter().map(|l| l.get_value(state)).collect::<Result<Vec<_>, _>>()?))
    },
    into_owned = (this) {
        Self {
            lines: this.lines.into_iter().map(|l| l.into_owned()).collect(),
            token: this.token.clone(),
        }.boxed()
    }
);

define_node!(
    Block {
        run_statements: Vec<Node<'i>>,
        ret_statement: Node<'i>
    },
    rules = [BLOCK],

    new = (input) {
        let token = input.to_token();
        let mut run_statements = input.into_inner()
            .filter(|c| c.as_str().trim().len() > 0)
            .map(|child| child.to_ast_node())
            .collect::<Result<Vec<_>, Error>>()?;

        if run_statements.len() == 0 {
            oops!(EmptyBlock, token)
        } else {
            let ret_statement = run_statements.pop().unwrap();
            Ok(Self {
                run_statements,
                ret_statement,
                token
            }.boxed())
        }
    },

    value = (this, state) {
        for statement in &this.run_statements {
            statement.get_value(state)?;
        }
        this.ret_statement.get_value(state)
    },
    into_owned = (this) {
        Self {
            run_statements: this.run_statements.into_iter().map(|s| s.into_owned()).collect(),
            ret_statement: this.ret_statement.into_owned(),
            token: this.token.clone(),
        }.boxed()
    }
);

define_node!(
    Expression { inner: Node<'i> },
    rules = [EXPR],
    new = (input) {
        let mut input = input.into_inner();
        if input.len() == 1 {
            return input.next().unwrap().to_ast_node();
        } else {
            pratt::Parser::parse(input)
        }
    },
    value = (this, state) {
        this.inner.get_value(state)
    },
    into_owned = (this) {
        Self {
            inner: this.inner.into_owned(),
            token: this.token.clone(),
        }
        .boxed()
    }
);
