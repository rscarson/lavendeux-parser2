use super::*;
use crate::{
    error::{ErrorDetails, WrapExternalError},
    pest::Rule,
};
use polyvalue::{
    operations::{
        ArithmeticOperation, ArithmeticOperationExt, BitwiseOperation, BitwiseOperationExt,
        BooleanOperation, BooleanOperationExt, IndexingMutationExt,
    },
    Value,
};

#[derive(Debug)]
pub enum AssignmentTarget<'i> {
    Identifier(String),
    Index(String, Vec<Option<Node<'i>>>), // None = last-entry index
    Destructure(Vec<String>),
}

impl std::fmt::Display for AssignmentTarget<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Identifier(id) => write!(f, "{}", id),
            Self::Index(base, indices) => {
                write!(f, "{}", base)?;
                for index in indices {
                    write!(
                        f,
                        "[{}]",
                        if let Some(i) = index {
                            &i.token().input
                        } else {
                            ""
                        }
                    )?;
                }
                Ok(())
            }
            Self::Destructure(ids) => {
                write!(f, "[{}]", ids.join(","))
            }
        }
    }
}

impl AssignmentTarget<'_> {
    /// Consumes a pair, returning an `AssignmentTarget` if it's valid
    pub fn from_pair<'i>(input: PrattPair<'i>) -> Result<Self, Error<'i>> {
        match input.as_rule() {
            Rule::identifier => Ok(Self::Identifier(input.first_pair().as_str().to_string())),

            Rule::POSTFIX_INDEX => {
                let mut children = input.into_inner();
                let base = children.next().unwrap();
                if base.as_rule() != Rule::identifier {
                    return oops!(ConstantValue, base.as_token());
                }
                let base = base.first_pair().as_str().to_string();

                let indices = children.next().unwrap();
                let indices = indices.first_pair().into_inner();

                let indices = indices
                    .map(|c| {
                        Ok::<_, Error<'i>>(if c.as_rule() == Rule::POSTFIX_EMPTYINDEX {
                            None
                        } else {
                            Some(c.to_ast_node()?)
                        })
                    })
                    .collect::<Result<Vec<Option<_>>, _>>()?;
                Ok(Self::Index(base, indices))
            }

            Rule::ARRAY_TERM => {
                todo!(); /*
                         let array = collections::Array::from_pair(input.first_pair())?;
                         let array = array
                             .as_any()
                             .downcast_ref::<collections::Array<'_>>()
                             .unwrap();
                         if array
                             .elements
                             .iter()
                             .any(|e| e.token().rule != Rule::identifier)
                         {
                             return oops!(ConstantValue, input.as_token());
                         } else if array.elements.is_empty() {
                             return oops!(ArrayEmpty, input.as_token());
                         }
                         let ids = array
                             .elements
                             .iter()
                             .map(|e| e.token().input.trim().to_string())
                             .collect::<Vec<_>>();
                         Ok(Self::Destructure(ids)) */
            }
            _ => {
                return oops!(ConstantValue, input.as_token());
            }
        }
    }
}

define_prattnode!(
    DeleteExpression {
        target: AssignmentTarget<'i>
    },
    rules = [PREFIX_DEL],
    new = (input) {
        let token = input.as_token();
        let mut children = input.into_inner();
        let is_decorator = children
            .next()
            .unwrap()
            .first_pair()
            .as_str()
            .ends_with("@");

        let target = children.next().unwrap();
        let mut target = AssignmentTarget::from_pair(target)?;

        match target {
            AssignmentTarget::Identifier(ref mut id) => {
                if is_decorator {
                    *id = format!("@{id}");
                }
            }
            _ if is_decorator => {
                return oops!(
                    DecoratorName {
                        name: target.to_string()
                    },
                    token
                );
            }
            _ => {}
        }

        Ok(Self { target, token }.boxed())
    },
    value = (this, state) {
        match &this.target {
            AssignmentTarget::Identifier(id) => {
                if let Some(function) = state.unregister_function(id)? {
                    Ok(function.signature().into())
                } else if let Some(value) = state.delete_variable(id) {
                    Ok(value)
                } else {
                    oops!(VariableName { name: id.clone() }, this.token.clone())
                }
            }

            AssignmentTarget::Index(id, idx) => {
                let mut value = state
                    .get_variable(id)
                    .ok_or(ErrorDetails::VariableName { name: id.clone() })
                    .with_context(this.token())?;
                let len = value.len();
                let mut pos = &mut value;
                let mut indices = idx
                    .iter()
                    .map(|i| {
                        Ok::<_, Error<'i>>(if let Some(v) = i {
                            Some(v.get_value(state)?)
                        } else {
                            None
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                if len == 0 {
                    return oops!(ArrayEmpty, this.token.clone());
                }
                let final_idx = indices.pop().unwrap().unwrap_or((len - 1).into());

                for index in &mut indices {
                    let len = pos.len();
                    if len == 0 {
                        return oops!(ArrayEmpty, this.token.clone());
                    }
                    let index = index.clone().unwrap_or((len - 1).into());
                    pos = pos.get_index_mut(&index).with_context(this.token())?;
                }

                let removed = pos.delete_index(&final_idx).with_context(this.token())?;
                state.set_variable(id, value);
                Ok(removed)
            }

            AssignmentTarget::Destructure(ids) => {
                for id in ids {
                    if state.get_variable(id).is_none() {
                        return oops!(
                            VariableName {
                                name: id.to_string()
                            },
                            this.token.clone()
                        );
                    }
                }
                Ok(Value::from(
                    ids.iter()
                        .map(|id| state.delete_variable(id).unwrap())
                        .collect::<Vec<_>>(),
                ))
            }
        }
    },

    docs = {
        name: "Deletion Keyword",
        symbols = ["del", "delete", "unset"],
        description: "
            Deletes a value, function, @decorator, or index
            Will return the value deleted (or the function signature if a function was deleted)
            Index can be blank to delete the last value in an array, or negative to count from the end
            Indices can also be a collection to delete multiple values at once
        ",
        examples: "
            a = 2; del a
            a = [1]; del a[]
            a = {'test': 1}; del a['test']

            @dec(x) = 2
            del @dec
        ",
    }
);

#[derive(Debug, Clone, Copy)]
#[rustfmt::skip]
pub enum AssignmentOperation {
    Add, Sub, Mul, Div, Mod, Pow,
    BitAnd, BitOr, BitXor, BitSl, BitSr,
    And, Or, None
}
impl AssignmentOperation {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn is_some(&self) -> bool {
        !self.is_none()
    }
}
impl From<Rule> for AssignmentOperation {
    fn from(value: Rule) -> Self {
        match value {
            Rule::OP_ASSIGN_ADD => Self::Add,
            Rule::OP_ASSIGN_SUB => Self::Sub,
            Rule::OP_ASSIGN_POW => Self::Pow,
            Rule::OP_ASSIGN_MUL => Self::Mul,
            Rule::OP_ASSIGN_DIV => Self::Div,
            Rule::OP_ASSIGN_MOD => Self::Mod,
            Rule::OP_BASSIGN_AND => Self::And,
            Rule::OP_BASSIGN_OR => Self::Or,
            Rule::OP_ASSIGN_AND => Self::BitAnd,
            Rule::OP_ASSIGN_XOR => Self::BitXor,
            Rule::OP_ASSIGN_OR => Self::BitOr,
            Rule::OP_ASSIGN_SL => Self::BitSl,
            Rule::OP_ASSIGN_SR => Self::BitSr,
            Rule::OP_ASSIGN => Self::None,
            _ => panic!("Unrecognized assignment operator rule: {:?}", value),
        }
    }
}

define_prattnode!(
    InfixAssignment {
        target: AssignmentTarget<'i>,
        op: AssignmentOperation,
        value: Node<'i>
    },
    rules = [
        OP_ASSIGN_ADD,
        OP_ASSIGN_SUB,
        OP_ASSIGN_POW,
        OP_ASSIGN_MUL,
        OP_ASSIGN_DIV,
        OP_ASSIGN_MOD,
        OP_ASSIGN_AND,
        OP_ASSIGN_XOR,
        OP_ASSIGN_OR,
        OP_ASSIGN_SL,
        OP_ASSIGN_SR,
        OP_BASSIGN_AND,
        OP_BASSIGN_OR,
        OP_ASSIGN
    ],
    new = (input) {
        let token = input.as_token();
        let mut children = input.into_inner();

        let lhs = children.next().unwrap();
        let op: AssignmentOperation = children.next().unwrap().as_rule().into();
        let value = children.next().unwrap().to_ast_node()?;

        let target = AssignmentTarget::from_pair(lhs)?;

        Ok(Self {
            target,
            op,
            value,
            token,
        }
        .boxed())
    },
    value = (this, state) {
        let rhs = this.value.get_value(state)?;
        match &this.target {
            AssignmentTarget::Identifier(ref id) => {
                let current_value = state.get_variable(id);
                let current_value = if this.op.is_some() {
                    if current_value.is_none() {
                        return oops!(
                            VariableName {
                                name: id.to_string()
                            },
                            this.token.clone()
                        );
                    }
                    apply_assignment_transform(&current_value.unwrap(), &rhs, this.op)?
                } else {
                    rhs
                };

                state.set_variable(id, current_value.clone());
                Ok(current_value)
            }

            AssignmentTarget::Destructure(ref ids) => {
                if this.op.is_some() {
                    return oops!(DestructuringAssignmentWithOperator, this.token.clone());
                }

                let values = rhs.as_a::<Vec<Value>>()?;
                if values.len() != ids.len() {
                    return oops!(
                        DestructuringAssignment {
                            expected_length: ids.len(),
                            actual_length: values.len()
                        },
                        this.token.clone()
                    );
                }
                for (id, value) in ids.iter().zip(values.clone().drain(..)) {
                    state.set_variable(id, value);
                }
                Ok(values.into())
            }

            AssignmentTarget::Index(ref base, indices) => {
                // The last index is the one that will be used to set the value
                let mut indices = indices.iter().map(|i| i).collect::<Vec<_>>();

                // Move through the indices to get the final pointer
                let mut dst = state
                    .get_variable(&base)
                    .ok_or(ErrorDetails::VariableName { name: base.clone() })
                    .with_context(this.token())?;

                let final_index = indices.pop().unwrap();

                let mut ptr = &mut dst;
                for index in indices {
                    let index = if let Some(v) = index {
                        v.get_value(state)?
                    } else {
                        (ptr.len() - 1).into()
                    };
                    ptr = ptr.get_index_mut(&index).with_context(this.token())?;
                }

                // Get final index
                let final_index = if let Some(v) = final_index {
                    v.get_value(state)?
                } else {
                    (ptr.len()).into()
                };

                // Set the value
                ptr.set_index(&final_index, rhs.clone())
                    .with_context(this.token())?;

                // Transform
                if this.op.is_some() {
                    dst = apply_assignment_transform(&dst, &rhs, this.op)?
                }

                // Set state and return
                state.set_variable(&base, dst.clone());
                Ok(dst)
            }
        }
    },

    docs = {
        name: "Assignment Operator",
        symbols = ["=", "+=", "-=", "*=", "/=", "%=", "**=", "&=", "|=", "^=", "<<=", ">>="],
        description: "
            Assigns a value to a variable, index, or destructuring assignment
            Target is either a literal with optional indices, or a destructuring assignment
            If an index is empty, a new value will be appended to the array
            If the target is a destructuring assignment, the value must be a collection of the same length
            If the operator is present, the value will be transformed before assignment
        ",
        examples: "
            [a, b] = [1, 2]
            a = 1; a += 1
            a = [1]; a[] = 2
        ",
    }
);

fn apply_assignment_transform<'i>(
    lhs: &Value,
    rhs: &Value,
    op: AssignmentOperation,
) -> Result<Value, Error<'i>> {
    Ok(match op {
        AssignmentOperation::Add => Value::arithmetic_op(lhs, rhs, ArithmeticOperation::Add)?,
        AssignmentOperation::Sub => Value::arithmetic_op(lhs, rhs, ArithmeticOperation::Subtract)?,
        AssignmentOperation::Mul => Value::arithmetic_op(lhs, rhs, ArithmeticOperation::Multiply)?,
        AssignmentOperation::Div => Value::arithmetic_op(lhs, rhs, ArithmeticOperation::Divide)?,
        AssignmentOperation::Mod => Value::arithmetic_op(lhs, rhs, ArithmeticOperation::Modulo)?,
        AssignmentOperation::Pow => {
            Value::arithmetic_op(lhs, rhs, ArithmeticOperation::Exponentiate)?
        }

        AssignmentOperation::BitAnd => Value::bitwise_op(lhs, rhs, BitwiseOperation::And)?,
        AssignmentOperation::BitOr => Value::bitwise_op(lhs, rhs, BitwiseOperation::Or)?,
        AssignmentOperation::BitXor => Value::bitwise_op(lhs, rhs, BitwiseOperation::Xor)?,
        AssignmentOperation::BitSl => Value::bitwise_op(lhs, rhs, BitwiseOperation::LeftShift)?,
        AssignmentOperation::BitSr => Value::bitwise_op(lhs, rhs, BitwiseOperation::RightShift)?,

        AssignmentOperation::And => Value::boolean_op(lhs, rhs, BooleanOperation::And)?,
        AssignmentOperation::Or => Value::boolean_op(lhs, rhs, BooleanOperation::Or)?,

        AssignmentOperation::None => rhs.clone(),
    })
}
