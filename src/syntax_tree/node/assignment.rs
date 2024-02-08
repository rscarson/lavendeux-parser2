//! Assignment Nodes
//!
//! Nodes for assignment statements.
//! To variables, functions and compound values
//!
use super::*;
use crate::{error::WrapError, Error, Rule, State, ToToken, Value};
use pest::iterators::Pair;
use polyvalue::{
    operations::{
        ArithmeticOperation, ArithmeticOperationExt, BitwiseOperation, BitwiseOperationExt,
        BooleanOperation, BooleanOperationExt, IndexingMutationExt,
    },
    types::Array,
    ValueTrait,
};

// identifier ~ "=" ~ TOPLEVEL_EXPRESSION
define_node!(
    VariableAssignment {
        name: String,
        value: Node
    },
    rules = [VARIABLE_ASSIGNMENT_EXPRESSION],
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

#[derive(Debug, Clone, PartialEq)]
pub enum OperativeAssignmentOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Exponent,
    BitAnd,
    BitOr,
    BitXor,
    BitShiftLeft,
    BitShiftRight,
    And,
    Or,
}
define_node!(
    OperativeAssignment {
        name: String,
        operation: OperativeAssignmentOperation,
        value: Node
    },
    rules = [OPERATIVE_ASSIGNMENT_EXPRESSION],
    new = |input: Pair<Rule>| {
        let token = input.to_token();
        let mut children = input.into_inner();

        let name = children.next().unwrap().as_str().to_string();
        let operation = match children.next().unwrap().as_str() {
            "+=" => OperativeAssignmentOperation::Add,
            "-=" => OperativeAssignmentOperation::Subtract,
            "*=" => OperativeAssignmentOperation::Multiply,
            "/=" => OperativeAssignmentOperation::Divide,
            "%=" => OperativeAssignmentOperation::Modulo,
            "**=" => OperativeAssignmentOperation::Exponent,

            "&=" => OperativeAssignmentOperation::BitAnd,
            "|=" => OperativeAssignmentOperation::BitOr,
            "^=" => OperativeAssignmentOperation::BitXor,
            "<<=" => OperativeAssignmentOperation::BitShiftLeft,
            ">>=" => OperativeAssignmentOperation::BitShiftRight,

            "&&=" => OperativeAssignmentOperation::And,
            "||=" => OperativeAssignmentOperation::Or,

            _ => unreachable!(),
        };
        let value = children.next().unwrap().to_ast_node()?;

        Ok(Self {
            name,
            operation,
            value,
            token,
        }
        .boxed())
    },
    value = |assignment: &OperativeAssignment, state: &mut State| {
        let left = state
            .get_variable(&assignment.name)
            .ok_or(Error::VariableName {
                name: assignment.name.clone(),
                token: assignment.token().clone(),
            })?;
        let right = (*assignment.value).get_value(state)?;

        let result = match assignment.operation {
            OperativeAssignmentOperation::Add => {
                Value::arithmetic_op(&left, &right, ArithmeticOperation::Add)
            }
            OperativeAssignmentOperation::Subtract => {
                Value::arithmetic_op(&left, &right, ArithmeticOperation::Subtract)
            }
            OperativeAssignmentOperation::Multiply => {
                Value::arithmetic_op(&left, &right, ArithmeticOperation::Multiply)
            }
            OperativeAssignmentOperation::Divide => {
                Value::arithmetic_op(&left, &right, ArithmeticOperation::Divide)
            }
            OperativeAssignmentOperation::Modulo => {
                Value::arithmetic_op(&left, &right, ArithmeticOperation::Modulo)
            }
            OperativeAssignmentOperation::Exponent => {
                Value::arithmetic_op(&left, &right, ArithmeticOperation::Exponentiate)
            }

            OperativeAssignmentOperation::BitAnd => {
                Value::bitwise_op(&left, &right, BitwiseOperation::And)
            }
            OperativeAssignmentOperation::BitOr => {
                Value::bitwise_op(&left, &right, BitwiseOperation::Or)
            }
            OperativeAssignmentOperation::BitXor => {
                Value::bitwise_op(&left, &right, BitwiseOperation::Xor)
            }
            OperativeAssignmentOperation::BitShiftLeft => {
                Value::bitwise_op(&left, &right, BitwiseOperation::LeftShift)
            }
            OperativeAssignmentOperation::BitShiftRight => {
                Value::bitwise_op(&left, &right, BitwiseOperation::RightShift)
            }

            OperativeAssignmentOperation::And => {
                Value::boolean_op(&left, &right, BooleanOperation::And)
            }
            OperativeAssignmentOperation::Or => {
                Value::boolean_op(&left, &right, BooleanOperation::Or)
            }
        }
        .to_error(&assignment.token)?;

        state.set_variable(&assignment.name, result.clone());
        Ok(result)
    }
);

// identifier ~ ("[" ~ TOPLEVEL_EXPRESSION ~ "]")+ ~ "=" ~ TOPLEVEL_EXPRESSION
define_node!(
    IndexAssignment {
        name: String,
        indices: Vec<Node>,
        value: Node
    },
    rules = [INDEX_ASSIGNMENT_EXPRESSION],

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
            ptr = ptr.get_index_mut(index).to_error(&assignment.token)?;
        }

        // Set the value
        ptr.set_index(&final_index, value.clone()).to_error(&assignment.token)?;

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
    rules = [DESTRUCTURING_ASSIGNMENT_EXPRESSION],

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
        let values = value.as_a::<Array>().to_error(&assignment.token)?.inner().clone();
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
    use super::*;
    use crate::{assert_tree, assert_tree_error};

    #[test]
    fn test_operative_assignment() {
        assert_tree!(
            "a += 1",
            OPERATIVE_ASSIGNMENT_EXPRESSION,
            OperativeAssignment,
            |tree: &mut OperativeAssignment| {
                assert_eq!(tree.name, "a");
                assert_eq!(tree.operation, OperativeAssignmentOperation::Add);
                assert_eq!(tree.value.to_string(), "1");

                let mut state = State::new();
                state.set_variable("a", Value::from(1));
                tree.get_value(&mut state).unwrap();
                assert_eq!(state.get_variable("a").unwrap().to_string(), "2");
            }
        );

        assert_tree!(
            "a -= 1",
            OPERATIVE_ASSIGNMENT_EXPRESSION,
            OperativeAssignment,
            |tree: &mut OperativeAssignment| {
                assert_eq!(tree.name, "a");
                assert_eq!(tree.operation, OperativeAssignmentOperation::Subtract);
                assert_eq!(tree.value.to_string(), "1");

                let mut state = State::new();
                state.set_variable("a", Value::from(1));
                tree.get_value(&mut state).unwrap();
                assert_eq!(state.get_variable("a").unwrap().to_string(), "0");
            }
        );

        assert_tree!(
            "a *= 2",
            OPERATIVE_ASSIGNMENT_EXPRESSION,
            OperativeAssignment,
            |tree: &mut OperativeAssignment| {
                assert_eq!(tree.name, "a");
                assert_eq!(tree.operation, OperativeAssignmentOperation::Multiply);
                assert_eq!(tree.value.to_string(), "2");

                let mut state = State::new();
                state.set_variable("a", Value::from(2));
                tree.get_value(&mut state).unwrap();
                assert_eq!(state.get_variable("a").unwrap().to_string(), "4");
            }
        );

        assert_tree!(
            "a /= 2",
            OPERATIVE_ASSIGNMENT_EXPRESSION,
            OperativeAssignment,
            |tree: &mut OperativeAssignment| {
                assert_eq!(tree.name, "a");
                assert_eq!(tree.operation, OperativeAssignmentOperation::Divide);
                assert_eq!(tree.value.to_string(), "2");

                let mut state = State::new();
                state.set_variable("a", Value::from(2));
                tree.get_value(&mut state).unwrap();
                assert_eq!(state.get_variable("a").unwrap().to_string(), "1");
            }
        );

        let mut parser = crate::Lavendeux::new(Default::default());

        parser.state_mut().set_variable("a", Value::from(2));
        assert_eq!(parser.parse("a %= 2").unwrap(), vec![0i64.into()]);

        parser.state_mut().set_variable("a", Value::from(2));
        assert_eq!(parser.parse("a **= 2").unwrap(), vec![4i64.into()]);

        parser.state_mut().set_variable("a", Value::from(2));
        assert_eq!(parser.parse("a &= 2").unwrap(), vec![2i64.into()]);

        parser.state_mut().set_variable("a", Value::from(2));
        assert_eq!(parser.parse("a |= 2").unwrap(), vec![2i64.into()]);

        parser.state_mut().set_variable("a", Value::from(2));
        assert_eq!(parser.parse("a ^= 2").unwrap(), vec![0i64.into()]);

        parser.state_mut().set_variable("a", Value::from(2));
        assert_eq!(parser.parse("a <<= 2").unwrap(), vec![8i64.into()]);

        parser.state_mut().set_variable("a", Value::from(2));
        assert_eq!(parser.parse("a >>= 2").unwrap(), vec![0i64.into()]);

        parser.state_mut().set_variable("a", Value::from(2));
        assert_eq!(parser.parse("a &&= 0").unwrap(), vec![false.into()]);

        parser.state_mut().set_variable("a", Value::from(2));
        assert_eq!(parser.parse("a ||= 0").unwrap(), vec![true.into()]);
    }

    #[test]
    fn test_destructuring_assignment() {
        assert_tree!(
            "(a, b, c) = a",
            DESTRUCTURING_ASSIGNMENT_EXPRESSION,
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

        let mut parser = crate::Lavendeux::new(Default::default());
        assert!(parser.parse("(a, b) = [1]").is_err());
        assert!(parser.parse("(a, b) = [1, 2, 3]").is_err());
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
                assert_eq!(tree.expressions, vec!["1"]);
            }
        );

        assert_tree!(
            "test(a) = 2*a",
            FUNCTION_ASSIGNMENT_STATEMENT,
            FunctionAssignment,
            |tree: &mut FunctionAssignment| {
                assert_eq!(tree.name, "test");
                assert_eq!(tree.arguments[0], "a");
                assert_eq!(tree.expressions, vec!["2*a"]);
            }
        );
    }

    #[test]
    fn test_variable_assignment() {
        assert_tree!(
            "a = 1",
            VARIABLE_ASSIGNMENT_EXPRESSION,
            VariableAssignment,
            |tree: &mut VariableAssignment| {
                assert_eq!(tree.name, "a");
                assert_eq!(tree.value.to_string(), "1");

                let mut state = State::new();
                tree.get_value(&mut state).unwrap();
                assert_eq!(state.get_variable("a").unwrap().to_string(), "1");
            }
        );

        assert_tree_error!("pi = 2", Syntax);
    }

    #[test]
    fn test_index_assignment() {
        assert_tree!(
            "a[-1] = 1",
            INDEX_ASSIGNMENT_EXPRESSION,
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
            INDEX_ASSIGNMENT_EXPRESSION,
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
            INDEX_ASSIGNMENT_EXPRESSION,
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
            INDEX_ASSIGNMENT_EXPRESSION,
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
            INDEX_ASSIGNMENT_EXPRESSION,
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
