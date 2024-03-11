use crate::{
    documentation::{DocumentationFormatter, PlaintextFormatter},
    error::ErrorDetails,
    functions::{stdlib, ParserFunction},
    network::ApiRegistry,
    syntax_tree::Reference,
    Error, Value,
};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

/// Implementation of the stack of scopes for the parser state
#[derive(Debug, Clone, Default)]
pub struct StateScopes {
    globals: HashMap<String, Value>,
    frames: Vec<(String, Option<Value>)>,
    locks: Vec<usize>,
    frame_starts: Vec<usize>,
}
impl StateScopes {
    const MAX_DEPTH: usize = 999;
    const GC_STACKSIZE: usize = 1024;

    /// Creates a blank stack
    pub fn new() -> Self {
        Self::default()
    }

    /// Prune invalid frames from the stack without affecting references
    pub fn prune_frames(&mut self) {
        loop {
            if match self.frames.last() {
                Some((_, v)) => v.is_none(),
                None => false,
            } {
                self.frames.pop();
            } else {
                break;
            }
        }
    }

    /// Returns the size of the stack in bytes
    pub fn stack_size(&self) -> usize {
        self.frames.len() * std::mem::size_of::<(String, Option<Value>)>()
    }

    /// Sorts the stack contents such that all empty values at the top, then removes them
    /// This is unsafe because it can cause references to be invalidated. It also probably
    /// isn't very fast.
    /// Only runs if the stack is larger than `GC_STACKSIZE` and we are in the global frame
    pub fn unsafely_sort_compress_stack(&mut self) {
        if self.frame_starts.is_empty() && self.stack_size() > Self::GC_STACKSIZE {
            self.frames.sort_by_key(|(_, v)| v.is_none());
            // find the first empty value
            let first_empty = self.frames.iter().position(|(_, v)| v.is_none());
            if let Some(first_empty) = first_empty {
                self.frames.truncate(first_empty);
            }
        }
    }

    /// Release all locks, and clear all frames
    /// Leaves the global frame intact
    pub fn reset(&mut self) {
        self.frames.clear();
        self.locks.clear();
        self.frame_starts.clear();
    }

    /// Returns the size of the stack, in frames
    pub fn stack_len(&self) -> usize {
        self.frames.len()
    }

    /// Increases the depth of the stack
    pub fn scope_into(&mut self) -> Result<(), Error> {
        if self.frame_starts.len() >= Self::MAX_DEPTH {
            oops!(StackOverflow)
        } else {
            self.frame_starts.push(self.stack_len());
            Ok(())
        }
    }

    /// Decreases the depth of the stack
    pub fn scope_out(&mut self) {
        if !self.frame_starts.is_empty() {
            self.frames.truncate(self.frame_starts.pop().unwrap());
            if self.stack_len() < self.last_valid_scope() {
                self.unlock_scope();
            }
        }
        self.prune_frames();
    }

    /// Locks the current scope, preventing access to variables in higher scopes
    pub fn lock_scope(&mut self) {
        self.locks.push(self.stack_len());
    }

    /// Unlocks the current scope, granting access to variables in higher scopes
    pub fn unlock_scope(&mut self) {
        self.locks.pop();
    }

    /// Returns the index from the bottom of the last frame valid for reading
    pub fn last_valid_scope(&self) -> usize {
        self.locks.last().cloned().unwrap_or_default()
    }

    /// Get a reference to the all valid scopes
    pub fn get_valid_scopes(&self) -> &[(String, Option<Value>)] {
        let start = self.last_valid_scope();
        &self.frames[start..]
    }

    /// Get a reference to the all valid scopes
    /// If ignore_lock is true, the last lock is ignored
    pub fn get_valid_scopes_mut(&mut self) -> &mut [(String, Option<Value>)] {
        let start = self.last_valid_scope();
        &mut self.frames[start..]
    }

    /// Set a global variable in the bottom of the stack
    pub fn set_global(&mut self, name: &str, value: Value) {
        self.globals.insert(name.to_string(), value);
    }

    /// Get a global variable from the bottom of the stack
    pub fn get_global(&self, name: &str) -> Option<&Value> {
        self.globals.get(name)
    }

    /// Get a value from the stack
    pub fn get(&self, name: &str) -> Option<&Value> {
        for (k, v) in self.get_valid_scopes().iter().rev() {
            if name == k {
                return v.as_ref();
            }
        }
        None
    }

    /// Get a value from the stack
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Value> {
        for (k, v) in self.get_valid_scopes_mut().iter_mut().rev() {
            if name == k {
                return v.as_mut();
            }
        }
        None
    }

    /// Get the address of a value in the stack
    pub fn address_of(&self, name: &str) -> Option<usize> {
        for (i, (k, _)) in self.get_valid_scopes().iter().rev().enumerate() {
            if name == k {
                return Some(self.last_valid_scope() + i);
            }
        }
        None
    }

    /// Get a value from the stack by reference
    pub fn get_by_ref(&self, address: usize) -> Option<&Value> {
        self.frames.get(address).map(|(_, v)| v.as_ref())?
    }

    /// Get a value from the stack by reference
    pub fn get_by_ref_mut(&mut self, address: usize) -> Option<&mut Value> {
        self.frames.get_mut(address).map(|(_, v)| v.as_mut())?
    }

    /// Delete a value from the stack by reference
    pub fn delete_by_ref(&mut self, address: usize) -> Option<Value> {
        self.frames.get_mut(address).map(|(_, v)| v.take())?
    }

    /// Write a value to the stack
    pub fn set(&mut self, name: &str, value: Value) {
        if let Some(v) = self.get_mut(name) {
            *v = value;
        } else {
            self.frames.push((name.to_string(), Some(value)));
        }
    }

    /// Write a value to the top of the stack
    pub fn set_top(&mut self, name: &str, value: Value) {
        self.frames.push((name.to_string(), Some(value)));
    }

    /// Deletes a value from the stack
    pub fn delete(&mut self, name: &str) -> Option<Value> {
        self.prune_frames();
        for (k, v) in self.get_valid_scopes_mut().iter_mut().rev() {
            if name == k {
                return v.take();
            }
        }
        None
    }

    /// Returns all variables in the state that are valid for reading
    pub fn all_variables_in_scope(&self) -> HashMap<&str, &Value> {
        let mut variables = HashMap::new();
        for (k, v) in self.get_valid_scopes() {
            if let Some(v) = v {
                variables.insert(k.as_str(), v);
            }
        }
        variables
    }

    /// Returns all variables in the state, regardless of locks
    pub fn all_variables(&self) -> HashMap<&str, &Value> {
        let mut variables = HashMap::new();
        for (k, v) in &self.frames {
            if let Some(v) = v {
                variables.insert(k.as_str(), v);
            }
        }
        variables
    }
}
