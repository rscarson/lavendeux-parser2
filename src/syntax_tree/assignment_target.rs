use super::{
    traits::{IntoOwned, NodeExt},
    Node,
};
use crate::{
    error::{ErrorDetails, WrapOption},
    Error, State,
};
use polyvalue::{
    operations::{IndexingMutationExt, IndexingOperationExt},
    Value, ValueType,
};

#[derive(Debug, Clone)]
enum TargetBase<'i> {
    Identifier(String),
    Const(Box<Node<'i>>),
}

/// Represents a reference within the state object.
/// This always consists of at least one base (an address within the state object)
/// Index also contains a list of indices, which are used to access the value within the state object.
/// Destructuring targets are used to assign multiple values at once.
#[derive(Debug, Clone)]
pub struct Target<'i> {
    base: TargetBase<'i>,
    indices: Vec<Option<Node<'i>>>,
}
impl<'i> Target<'i> {
    pub fn from_target(parent: Self, indices: Vec<Option<Node<'i>>>) -> Self {
        Self {
            base: parent.base,
            indices,
        }
    }

    /// Creates a new target with the given name and indices.
    pub fn with_identifier(base: String, indices: Vec<Option<Node<'i>>>) -> Self {
        Self {
            base: TargetBase::Identifier(base),
            indices,
        }
    }

    /// Creates a new target with the given constant value and indices.
    pub fn with_const(base: Node<'i>, indices: Vec<Option<Node<'i>>>) -> Self {
        Self {
            base: TargetBase::Const(Box::new(base)),
            indices,
        }
    }

    /// Returns a value, if one exists
    pub fn get(&self, state: &mut State) -> Result<Value, Error> {
        let mut base = match self.base {
            TargetBase::Identifier(ref name) => state
                .get(name)
                .cloned()
                .or_error(ErrorDetails::VariableName { name: name.clone() })?,
            TargetBase::Const(ref node) => node.evaluate(state)?,
        };

        let indices = self
            .indices
            .iter()
            .map(|i| i.as_ref().map(|i| i.evaluate(state)).transpose())
            .collect::<Result<Vec<_>, _>>()?;

        for index in indices {
            let index = index
                .unwrap_or_else(|| Value::from(if base.len() == 0 { 0 } else { base.len() - 1 }));

            if index.is_a(ValueType::Collection) && !index.is_a(ValueType::String) {
                base = base.get_indices(&index)?;
            } else {
                base = base.get_index(&index)?;
            }
        }

        Ok(base)
    }

    /// Get the value this target refers to.
    pub fn get_mut<'s>(&self, state: &'s mut State) -> Result<&'s mut Value, Error> {
        let indices = self
            .indices
            .iter()
            .map(|i| i.as_ref().map(|i| i.evaluate(state)).transpose())
            .collect::<Result<Vec<_>, _>>()?;

        let mut base = match self.base {
            TargetBase::Identifier(ref name) => state
                .stack_mut()
                .get_mut(name)
                .or_error(ErrorDetails::VariableName { name: name.clone() })?,
            TargetBase::Const(_) => return oops!(ConstantValue),
        };

        for index in indices {
            let index = index
                .unwrap_or_else(|| Value::from(if base.len() == 0 { 0 } else { base.len() - 1 }));
            base = base.get_index_mut(&index)?;
        }

        Ok(base)
    }

    /// Write a value to the target, if it is not a constant.
    pub fn write(&self, state: &mut State, value: Value) -> Result<(), Error> {
        if self.indices.is_empty() {
            match &self.base {
                TargetBase::Identifier(name) => {
                    state.set(name, value);
                    Ok(())
                }
                TargetBase::Const(_) => oops!(ConstantValue),
            }
        } else {
            let base = self.get_mut(state)?;
            *base = value;
            Ok(())
        }
    }

    /// Deletes the value this target refers to.
    pub fn delete(&self, state: &mut State) -> Result<Value, Error> {
        if matches!(self.base, TargetBase::Const(_)) {
            self.get(state)
        } else {
            let name = match &self.base {
                TargetBase::Identifier(name) => name,
                TargetBase::Const(_) => return oops!(ConstantValue),
            };

            if self.indices.is_empty() {
                state
                    .stack_mut()
                    .delete(name)
                    .or_error(ErrorDetails::ConstantValue)
            } else {
                let mut indices = self
                    .indices
                    .iter()
                    .map(|i| i.as_ref().map(|i| i.evaluate(state)).transpose())
                    .collect::<Result<Vec<_>, _>>()?;

                let mut base = state
                    .stack_mut()
                    .get_mut(name)
                    .or_error(ErrorDetails::VariableName { name: name.clone() })?;

                let final_index = indices.pop().unwrap();

                for index in indices {
                    let index = index.unwrap_or_else(|| {
                        Value::from(if base.len() == 0 { 0 } else { base.len() - 1 })
                    });
                    base = base.get_index_mut(&index)?;
                }

                let final_index = final_index.unwrap_or_else(|| {
                    Value::from(if base.len() == 0 { 0 } else { base.len() - 1 })
                });

                Ok(base.delete_index(&final_index)?)
            }
        }
    }
}

impl IntoOwned for Target<'_> {
    type Owned = Target<'static>;
    fn into_owned(self) -> Self::Owned {
        Self::Owned {
            base: match self.base {
                TargetBase::Identifier(name) => TargetBase::Identifier(name),
                TargetBase::Const(node) => TargetBase::Const(Box::new(node.into_owned())),
            },
            indices: self
                .indices
                .into_iter()
                .map(|i| i.map(Node::into_owned))
                .collect(),
        }
    }
}
