//! Core Node
//!
//! High-level nodes that are used to build the syntax tree.
//! These nodes are how the user will interact with the syntax tree.
//!
use super::*;
use crate::{Error, Rule, State, ToToken, Value};
use pest::iterators::Pair;

// LINE* ~ EOI
define_node!(
    Script {
        statements: Vec<Node>
    },
    rules = [SCRIPT],

    new = |input:Pair<Rule>| {
        let token = input.to_token();


        let statements = input
            .into_inner()
            .filter(|child| child.as_rule() != Rule::EOI && child.as_rule() != Rule::EOL)
            .map(|child| child.to_ast_node())
            .collect::<Result<Vec<Node>, Error>>()?;

        Ok(Self {
            statements,
            token
        }.boxed())
    },

    value = |script: &Script, state: &mut State| {
        let values = script.statements.iter().map(|node| node.get_value(state)).collect::<Result<Vec<_>, _>>()?;
        Ok(Value::array(values))
    }
);

// EXPRESSION ~ "@" ~ identifier ~ EOL
// | EXPRESSION ~ EOL
// | EOL
define_node!(
    Line {
        expression: Option<Node>,
        decorator: Option<String>
    },
    rules = [LINE],

    new = |input:Pair<Rule>| {
        let token = input.to_token();
        let mut children = input.into_inner();

        let next = children.next();
        if next.is_none() {
            return Ok(Self {
                expression: None,
                decorator: None,
                token
            }.boxed());
        } else {
            let expression = next.unwrap().to_ast_node()?;
            let decorator = children.next().map(|c| c.as_str().to_string());

            Ok(Self {
                expression: Some(expression),
                decorator,
                token
            }.boxed())
        }
    },

    value = |line: &Line, state: &mut State| {
        if let Some(expression) = &line.expression {
            let value = expression.get_value(state)?;
            if let Some(decorator) = &line.decorator {
                let result = state.decorate(decorator, line.token(), value)?;
                Ok(Value::from(result))
            } else {
                Ok(value)
            }
        } else {
            Ok(Value::from(""))
        }
    }
);

define_node!(
    Block {
        lines: Vec<Node>
    },
    rules = [BLOCK],

    new = |input:Pair<Rule>| {
        let token = input.to_token();
        let children = input.into_inner();

        let lines = children
            .map(|child| child.to_ast_node())
            .collect::<Result<Vec<Node>, Error>>()?;

        Ok(Self {
            lines,
            token
        }.boxed())
    },

    value = |block: &Block, state: &mut State| {
        let mut result = Value::from("");
        for line in &block.lines {
            result = line.get_value(state)?;
        }

        Ok(result)
    }
);

#[cfg(test)]
mod test {
    use super::*;
    use crate::assert_tree;

    #[test]
    fn test_line() {
        assert_tree!("1", LINE, Line, |node: &mut Line| {
            assert_eq!(node.expression.as_ref().unwrap().to_string(), "1");
            assert_eq!(node.decorator, None);
            assert_eq!(node.get_value(&mut State::new()).unwrap().to_string(), "1");
        });

        assert_tree!("1 @bool", LINE, Line, |node: &mut Line| {
            assert_eq!(node.expression.as_ref().unwrap().to_string(), "1");
            assert_eq!(node.decorator.as_ref().unwrap(), "bool");
            assert_eq!(
                node.get_value(&mut State::new()).unwrap().to_string(),
                "true"
            );
        });

        assert_tree!("\n", LINE, Line, |node: &mut Line| {
            assert!(node.expression.as_ref().is_none());
            assert_eq!(node.decorator.as_ref(), None);
            assert_eq!(node.get_value(&mut State::new()).unwrap().to_string(), "");
        });
    }

    #[test]
    fn test_script() {
        assert_tree!("1\n2", SCRIPT, Script, |node: &mut Script| {
            assert_eq!(node.statements.len(), 2);
            assert_eq!(node.statements[0].to_string(), "1\n");
            assert_eq!(node.statements[1].to_string(), "2");
            assert_eq!(
                node.get_value(&mut State::new()).unwrap().to_string(),
                "[1, 2]"
            );
        });

        assert_tree!("1 \\\n@bool", SCRIPT, Script, |node: &mut Script| {
            assert_eq!(node.statements.len(), 1);
            assert_eq!(
                node.get_value(&mut State::new()).unwrap().to_string(),
                "[true]"
            );
        });

        assert_tree!("", SCRIPT, Script, |node: &mut Script| {
            assert_eq!(node.statements.len(), 0);
        });
    }
}
