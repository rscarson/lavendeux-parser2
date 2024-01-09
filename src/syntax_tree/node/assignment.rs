//! Assignment Nodes
//!
//! Nodes for assignment statements.
//! To variables, functions and compound values
//!
use super::*;
use crate::{parse_input, user_function::UserFunction, Error, Rule, State, ToToken, Value};
use pest::iterators::Pair;
use polyvalue::{operations::IndexingOperationExt, types::Array, ValueTrait};

// identifier ~ "(" ~ ")" ~ "=" ~ TOPLEVEL_EXPRESSION |
// identifier ~ "(" ~ identifier ~ ("," ~ identifier)* ~ ")" ~ "=" ~ TOPLEVEL_EXPRESSION
define_node!(
    FunctionAssignment {
        name: String,
        arguments: Vec<String>,
        expression: String
    },
    rules = [FUNCTION_ASSIGNMENT_STATEMENT],

    new = |input:Pair<Rule>| {
        let token = input.to_token();
        let mut children = input.into_inner();

        // Name of the function is the first child
        let name = children.next().unwrap().as_str().to_string();

        // Parse arguments
        let mut arguments = Vec::new();
        while children.peek().is_some() {
            let arg = children.next().unwrap();
            arguments.push(arg.as_str().to_string());
        }

        // Confirm validity of the function body by parsing it here
        let expression = arguments.pop().unwrap();
        parse_input(&expression, Rule::TOPLEVEL_EXPRESSION)?;

        Ok(Self {
            name,
            arguments,
            expression,
            token
        }.boxed())
    },

    value = |assignment: &FunctionAssignment, state: &mut State| {
        let function = UserFunction::new(&assignment.name, assignment.arguments.clone(), assignment.expression.clone())?;
        state.set_user_function(function);
        Ok(Value::from(assignment.expression.clone()))
    }
);

// identifier ~ "=" ~ TOPLEVEL_EXPRESSION
define_node!(
    VariableAssignment {
        name: String,
        value: Node
    },
    rules = [VARIABLE_ASSIGNMENT_STATEMENT],
    new = |input: Pair<Rule>| {
        let token = input.to_token();
        let mut children = input.into_inner();

        let name = children.next().unwrap().as_str().to_string();
        let value = children.next().unwrap().to_ast_node()?;
        Ok(Self { name, value, token }.boxed())
    },
    value = |assignment: &VariableAssignment, state: &mut State| {
        let value = (*assignment.value).get_value(state)?;
        state.set_variable(&assignment.name, value.clone());
        Ok(value)
    }
);

// identifier ~ ("[" ~ TOPLEVEL_EXPRESSION ~ "]")+ ~ "=" ~ TOPLEVEL_EXPRESSION
define_node!(
    IndexAssignment {
        name: String,
        indices: Vec<Node>,
        value: Node
    },
    rules = [INDEX_ASSIGNMENT_STATEMENT],

    new = |input:Pair<Rule>| {
        let token = input.to_token();
        let mut children = input.into_inner();

        let name = children.next().unwrap().as_str().to_string();
        let mut indices = Vec::new();
        while children.peek().is_some() {
            let index = children.next().unwrap();
            indices.push(index.to_ast_node()?);
        }

        let value = indices.pop().unwrap();

        Ok(Self {
            name,
            indices,
            value,
            token,
        }.boxed())
    },

    value = |assignment: &IndexAssignment, state: &mut State| {
        let value = (*assignment.value).get_value(state)?;

        // The last index is the one that will be used to set the value
        let mut indices = assignment.indices.iter().map(|node| node.get_value(state)).collect::<Result<Vec<_>, _>>()?;
        let final_index = indices.pop().unwrap();

        // Move through the indices to get the final pointer
        let mut dst = state.get_variable(&assignment.name).ok_or(Error::VariableName {
            name: assignment.name.clone(),
            token: assignment.token().clone(),
        })?;
        let mut ptr = &mut dst;
        for index in &indices {
            ptr = ptr.get_index_mut(index)?;
        }

        // Set the value
        ptr.set_index(&final_index, value.clone())?;

        // Set state and return
        state.set_variable(&assignment.name, dst);
        Ok(value)
    }
);

define_node!(
    DestructuringAssignment {
        names: Vec<String>,
        value: Node
    },
    rules = [DESTRUCTURING_ASSIGNMENT_STATEMENT],

    new = |input:Pair<Rule>| {
        let token = input.to_token();
        let mut children = input.into_inner().rev();

        let value = children.next().unwrap().to_ast_node()?; // last child is the value

        let mut children = children.rev();

        let mut names = Vec::new();
        while let Some(next) = children.next() {
            let name = next.as_str().to_string();
            names.push(name);
        }

        Ok(Self {
            names,
            value,
            token,
        }.boxed())
    },

    value = |assignment: &DestructuringAssignment, state: &mut State| {
        let value = (*assignment.value).get_value(state)?;
        let values = value.as_a::<Array>()?.inner().clone();
        if values.len() != assignment.names.len() {
            return Err(Error::DestructuringAssignment {
                expected_length: assignment.names.len(),
                actual_length: values.len(),
                token: assignment.token().clone(),
            });
        }

        for (name, value) in assignment.names.iter().zip(values) {
            state.set_variable(name, value);
        }

        Ok(value)
    }
);

#[cfg(test)]
mod test {
    use crate::{assert_tree, assert_tree_error};

    use super::*;

    #[test]
    fn test_destructuring_assignment() {
        assert_tree!(
            "(a, b, c) = a",
            DESTRUCTURING_ASSIGNMENT_STATEMENT,
            DestructuringAssignment,
            |tree: &mut DestructuringAssignment| {
                assert_eq!(tree.names.len(), 3);
                assert_eq!(tree.names[0], "a");
                assert_eq!(tree.names[1], "b");
                assert_eq!(tree.names[2], "c");
                assert_eq!(tree.value.to_string(), "a");

                let mut state = State::new();
                state.set_variable(
                    "a",
                    Value::from(vec![Value::from(1), Value::from(2), Value::from(3)]),
                );
                tree.get_value(&mut state).unwrap();
                assert_eq!(state.get_variable("a").unwrap().to_string(), "1");
                assert_eq!(state.get_variable("b").unwrap().to_string(), "2");
                assert_eq!(state.get_variable("c").unwrap().to_string(), "3");
            }
        );
    }

    #[test]
    fn test_function_assignment() {
        assert_tree!(
            "test() = 1",
            FUNCTION_ASSIGNMENT_STATEMENT,
            FunctionAssignment,
            |tree: &mut FunctionAssignment| {
                assert_eq!(tree.name, "test");
                assert_eq!(0, tree.arguments.len());
                assert_eq!(tree.expression, "1");
            }
        );

        assert_tree!(
            "test(a) = 2a",
            FUNCTION_ASSIGNMENT_STATEMENT,
            FunctionAssignment,
            |tree: &mut FunctionAssignment| {
                assert_eq!(tree.name, "test");
                assert_eq!(tree.arguments[0], "a");
                assert_eq!(tree.expression, "2a");
            }
        );
    }

    #[test]
    fn test_variable_assignment() {
        assert_tree!(
            "a = 1",
            VARIABLE_ASSIGNMENT_STATEMENT,
            VariableAssignment,
            |tree: &mut VariableAssignment| {
                assert_eq!(tree.name, "a");
                assert_eq!(tree.value.to_string(), "1");

                let mut state = State::new();
                tree.get_value(&mut state).unwrap();
                assert_eq!(state.get_variable("a").unwrap().to_string(), "1");
            }
        );

        assert_tree_error!("pi = 2", Pest);
    }

    #[test]
    fn test_index_assignment() {
        assert_tree!(
            "a[-1] = 1",
            INDEX_ASSIGNMENT_STATEMENT,
            IndexAssignment,
            |tree: &mut IndexAssignment| {
                assert_eq!(tree.name, "a");
                assert_eq!(tree.indices.len(), 1);
                assert_eq!(tree.indices[0].to_string(), "-1");
                assert_eq!(tree.value.to_string(), "1");

                let mut state = State::new();
                state.set_variable("a", Value::from(vec![Value::from(0), Value::from(1)]));
                let value = tree.get_value(&mut state).unwrap();
                assert_eq!(value.to_string(), "1");
            }
        );

        assert_tree!(
            "a[2] = 1",
            INDEX_ASSIGNMENT_STATEMENT,
            IndexAssignment,
            |tree: &mut IndexAssignment| {
                assert_eq!(tree.name, "a");
                assert_eq!(tree.indices.len(), 1);
                assert_eq!(tree.indices[0].to_string(), "2");

                let mut state = State::new();
                state.set_variable("a", Value::from(vec![Value::from(0)]));
                tree.get_value(&mut state)
                    .expect_err("Should not be able to assign to out-of-bounds index");
            }
        );

        assert_tree!(
            "a[0] = 1",
            INDEX_ASSIGNMENT_STATEMENT,
            IndexAssignment,
            |tree: &mut IndexAssignment| {
                assert_eq!(tree.name, "a");
                assert_eq!(tree.indices.len(), 1);
                assert_eq!(tree.indices[0].to_string(), "0");
                assert_eq!(tree.value.to_string(), "1");

                let mut state = State::new();
                state.set_variable("a", Value::from(vec![Value::from(0)]));
                tree.get_value(&mut state).unwrap();
                assert_eq!(state.get_variable("a").unwrap().to_string(), "[1]");
            }
        );

        assert_tree!(
            "a[0][1] = 1",
            INDEX_ASSIGNMENT_STATEMENT,
            IndexAssignment,
            |tree: &mut IndexAssignment| {
                assert_eq!(tree.name, "a");
                assert_eq!(tree.indices.len(), 2);
                assert_eq!(tree.indices[0].to_string(), "0");
                assert_eq!(tree.indices[1].to_string(), "1");
                assert_eq!(tree.value.to_string(), "1");

                let mut state = State::new();
                state.set_variable("a", Value::from(vec![Value::from(vec![Value::from(0)])]));
                tree.get_value(&mut state).unwrap();
                assert_eq!(state.get_variable("a").unwrap().to_string(), "[[0, 1]]");
            }
        );

        assert_tree!(
            "a[0][0][1] = 1",
            INDEX_ASSIGNMENT_STATEMENT,
            IndexAssignment,
            |tree: &mut IndexAssignment| {
                assert_eq!(tree.name, "a");
                assert_eq!(tree.indices.len(), 3);
                assert_eq!(tree.indices[0].to_string(), "0");
                assert_eq!(tree.indices[1].to_string(), "0");
                assert_eq!(tree.indices[2].to_string(), "1");
                assert_eq!(tree.value.to_string(), "1");

                let mut state = State::new();
                state.set_variable(
                    "a",
                    Value::from(vec![Value::from(vec![Value::from(vec![Value::from(0)])])]),
                );
                tree.get_value(&mut state).unwrap();
                assert_eq!(state.get_variable("a").unwrap().to_string(), "[[[0, 1]]]");
            }
        );
    }
}
