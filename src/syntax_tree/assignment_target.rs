use crate::{
    error::{ErrorDetails, WrapOption},
    Error, State,
};

use super::{
    traits::{IntoOwned, NodeExt},
    Node,
};
use polyvalue::{
    operations::{IndexingMutationExt, IndexingOperationExt},
    Value, ValueType,
};

#[derive(Debug, Clone)]
pub enum AssignmentTarget<'i> {
    Identifier(String),
    Index(String, Vec<Option<Node<'i>>>), // None = last-entry index
    Destructure(Vec<AssignmentTarget<'i>>),
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
            Self::Destructure(targets) => {
                write!(
                    f,
                    "[{}]",
                    targets
                        .iter()
                        .map(|t| t.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                )
            }
        }
    }
}

impl IntoOwned for AssignmentTarget<'_> {
    type Owned = AssignmentTarget<'static>;
    fn into_owned(self) -> Self::Owned {
        match self {
            Self::Identifier(id) => Self::Owned::Identifier(id),
            Self::Index(base, indices) => Self::Owned::Index(
                base,
                indices
                    .into_iter()
                    .map(|i| i.map(|i| i.into_owned()))
                    .collect(),
            ),
            Self::Destructure(targets) => {
                Self::Owned::Destructure(targets.into_iter().map(|t| t.into_owned()).collect())
            }
        }
    }
}

impl<'i> AssignmentTarget<'i> {
    pub fn get_index_handle<'v>(base: Value, indices: &[Option<Value>]) -> Result<Value, Error> {
        let mut base = base;
        for index in indices {
            let default_idx = Value::from(if base.len() == 0 { 0 } else { base.len() - 1 });
            let index = index.as_ref().unwrap_or(&default_idx);

            if index.is_a(ValueType::Collection) && !index.is_a(ValueType::String) {
                base = base.get_indices(index)?;
            } else {
                base = base.get_index(index)?;
            }
        }
        Ok(base)
    }

    pub fn get_mut_index_handle<'v>(
        base: &'v mut Value,
        indices: &[Option<Value>],
    ) -> Result<&'v mut Value, Error> {
        let mut base = base;
        for index in indices {
            let default_idx = Value::from(if base.len() == 0 { 0 } else { base.len() - 1 });
            let index = index.as_ref().unwrap_or(&default_idx);
            base = base.get_index_mut(index)?;
        }
        Ok(base)
    }

    pub fn get_value(&self, state: &mut State) -> Result<Value, Error> {
        match self {
            Self::Identifier(id) => state
                .get_variable(id)
                .cloned()
                .or_error(ErrorDetails::VariableName { name: id.clone() }),
            Self::Index(base, indices) => {
                let mut idx = vec![];
                for index in indices {
                    idx.push(index.as_ref().map(|i| i.evaluate(state)).transpose()?);
                }

                let base = state
                    .get_variable(base)
                    .cloned()
                    .or_error(ErrorDetails::VariableName { name: base.clone() })?;
                Self::get_index_handle(base, &idx)
            }
            Self::Destructure(_) => {
                oops!(Internal {
                    msg: format!("Destructuring references are only valid on assignments!")
                })
            }
        }
    }

    pub fn get_value_in_parent(&self, state: &mut State) -> Result<Value, Error> {
        match self {
            Self::Identifier(id) => state
                .get_variable_as_parent(id)
                .cloned()
                .or_error(ErrorDetails::VariableName { name: id.clone() }),
            Self::Index(base, indices) => {
                let mut idx = vec![];
                for index in indices {
                    idx.push(index.as_ref().map(|i| i.evaluate(state)).transpose()?);
                }

                let base = state
                    .get_variable_as_parent(base)
                    .cloned()
                    .or_error(ErrorDetails::VariableName { name: base.clone() })?;
                Self::get_index_handle(base, &idx)
            }
            Self::Destructure(_) => {
                oops!(Internal {
                    msg: format!("Destructuring references are only valid on assignments!")
                })
            }
        }
    }

    pub fn get_value_mut<'s>(&self, state: &'s mut State) -> Result<&'s mut Value, Error> {
        match self {
            Self::Identifier(id) => state
                .get_variable_mut(id)
                .or_error(ErrorDetails::VariableName { name: id.clone() }),
            Self::Index(base, indices) => {
                let mut idx = vec![];
                for index in indices {
                    idx.push(index.as_ref().map(|i| i.evaluate(state)).transpose()?);
                }

                let base = state
                    .get_variable_mut(base)
                    .or_error(ErrorDetails::VariableName { name: base.clone() })?;
                Self::get_mut_index_handle(base, &idx)
            }
            Self::Destructure(_) => {
                oops!(Internal {
                    msg: format!("Destructuring references are only valid on assignments!")
                })
            }
        }
    }

    pub fn update_value(&self, state: &mut State, value: Value) -> Result<(), Error> {
        match self {
            Self::Identifier(id) => {
                state.set_variable(id, value);
                Ok(())
            },
            Self::Index(base, indices) => {
                let mut idx = vec![];
                for index in indices {
                    idx.push(index.as_ref().map(|i| i.evaluate(state)).transpose()?);
                }

                let mut base = state
                    .get_variable_mut(base)
                    .or_error(ErrorDetails::VariableName { name: base.clone() })?;

                if idx.is_empty() {
                    *base = value;
                    return Ok(());
                }

                let target_idx = idx.pop().unwrap();
                base = Self::get_mut_index_handle(base, &idx)?;

                let target_idx = target_idx.unwrap_or(base.len().into());
                base.set_index(&target_idx, value)?;
                Ok(())
            }
            Self::Destructure(targets) => {
                if targets.len() != value.len() {
                    oops!(DestructuringAssignment {
                        expected_length: targets.len(),
                        actual_length: value.len()
                    })
                } else {
                    let values = value.as_a::<Vec<Value>>()?;
                    for (target, value) in targets.iter().zip(values.into_iter()) {
                        target.update_value(state, value)?;
                    }
                    Ok(())
                }
            }
        }
    }

    pub fn update_value_in_parent(&self, state: &mut State, value: Value) -> Result<(), Error> {
        match self {
            Self::Identifier(id) => {
                state.set_variable_as_parent(id, value);
                Ok(())
            },
            Self::Index(base, indices) => {
                let mut idx = vec![];
                for index in indices {
                    idx.push(index.as_ref().map(|i| i.evaluate(state)).transpose()?);
                }

                let mut base = state
                    .get_variable_mut_as_parent(base)
                    .or_error(ErrorDetails::VariableName { name: base.clone() })?;
                base = Self::get_mut_index_handle(base, &idx)?;

                if idx.is_empty() {
                    *base = value;
                    return Ok(());
                }

                let target_idx = idx.pop().unwrap();
                base = Self::get_mut_index_handle(base, &idx)?;

                let target_idx = target_idx.unwrap_or(base.len().into());
                base.set_index(&target_idx, value)?;
                Ok(())
            }
            Self::Destructure(targets) => {
                if targets.len() != value.len() {
                    oops!(DestructuringAssignment {
                        expected_length: targets.len(),
                        actual_length: value.len()
                    })
                } else {
                    let values = value.as_a::<Vec<Value>>()?;
                    for (target, value) in targets.iter().zip(values.into_iter()) {
                        target.update_value_in_parent(state, value)?;
                    }
                    Ok(())
                }
            }
        }
    }

    pub fn delete(&self, state: &mut State) -> Result<Value, Error> {
        match self {
            Self::Identifier(id) => {
                if let Some(function) = state.unregister_function(id)? {
                    Ok(function.signature().into())
                } else if let Some(value) = state.delete_variable(id) {
                    Ok(value)
                } else {
                    oops!(VariableName { name: id.clone() })
                }
            }

            AssignmentTarget::Index(base, indices) => {
                let mut idx = vec![];
                for index in indices {
                    idx.push(index.as_ref().map(|i| i.evaluate(state)).transpose()?);
                }

                if indices.is_empty() {
                    return oops!(ArrayEmpty);
                }

                let mut base = state
                    .get_variable_mut(base)
                    .or_error(ErrorDetails::VariableName { name: base.clone() })?;
                let target_idx = idx.pop().unwrap();
                base = Self::get_mut_index_handle(base, &idx)?;

                let target_idx = target_idx.unwrap_or((base.len() - 1).into());

                Ok(base.delete_index(&target_idx)?)
            }

            AssignmentTarget::Destructure(ids) => {
                let results = ids
                    .iter()
                    .map(|id| id.delete(state))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Value::from(results))
            }
        }
    }
}
