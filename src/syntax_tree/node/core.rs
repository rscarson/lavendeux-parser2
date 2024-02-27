//! Core Node
//!
//! High-level nodes that are used to build the syntax tree.
//!
use super::*;
use crate::{syntax_tree::pratt, Error, Rule, State, ToToken};
use pest::iterators::Pair;
use polyvalue::Value;

define_node!(
    Script { lines: Vec<Node> },
    rules = [SCRIPT],

    new = |input: Pair<Rule>| {
        let token = input.to_token();
        let lines = input.into_inner().map(|child| child.to_ast_node()).collect::<Result<_, _>>()?;
        Ok(Self { lines, token }.boxed())
    },

    value = |this: &Self, state: &mut State| {
        Ok(Value::array(this.lines.iter().map(|l| l.get_value(state)).collect::<Result<Vec<_>, _>>()?))
    }
);

define_node!(
    Block {
        run_statements: Vec<Node>,
        ret_statement: Node
    },
    rules = [BLOCK],

    new = |input:Pair<Rule>| {
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

    value = |this: &Self, state: &mut State| {
        for statement in &this.run_statements {
            statement.get_value(state)?;
        }
        this.ret_statement.get_value(state)
    }
);

define_node!(
    Expression { inner: Node },
    rules = [EXPR],
    new = |input: Pair<Rule>| { pratt::Parser::parse(input) },
    value = |this: &Self, state: &mut State| { this.inner.get_value(state) }
);
