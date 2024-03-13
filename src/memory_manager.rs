#![allow(dead_code)]
use crate::{Error, Value};
use std::collections::HashMap;

/// Implementation of the stack of scopes for the parser state
#[derive(Debug, Clone, Default)]
pub struct MemoryManager {
    globals: HashMap<String, Value>,
    frames: Vec<(String, Option<Value>)>,
    locks: Vec<usize>,
    frame_starts: Vec<usize>,
}
impl MemoryManager {
    const GC_STACKSIZE: usize = 1024;
    const MAX_DEPTH: usize = 15000;

    /// Creates a blank stack
    pub fn new() -> Self {
        Self::default()
    }

    /// Prune invalid frames from the stack without affecting references
    fn prune_frames(&mut self) {
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
    fn stack_size(&self) -> usize {
        self.frames.len() * std::mem::size_of::<(String, Option<Value>)>()
    }

    /// Sorts the stack contents such that all empty values at the top, then removes them
    /// This is unsafe because it can cause references to be invalidated. It also probably
    /// isn't very fast.
    /// Only runs if the stack is larger than `GC_STACKSIZE` and we are in the global frame
    fn unsafely_sort_compress_stack(&mut self) {
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
        self.frames
            .truncate(self.frame_starts.first().map(|s| *s).unwrap_or_default());
        self.locks.clear();
        self.frame_starts.clear();

        self.unsafely_sort_compress_stack();
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
            if self.stack_len() < self.last_valid_scope(0) {
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
    /// Will act as if the stack is frame_offset frames up from the current frame
    fn last_valid_scope(&self, frame_offset: usize) -> usize {
        for lock in self.locks.iter().rev() {
            if *lock <= self.stack_len() - frame_offset {
                return *lock;
            }
        }
        0
    }

    /// Get a reference to the all valid scopes
    /// Will act as if the stack is frame_offset frames up from the current frame
    fn get_valid_scopes(&self, frame_offset: usize) -> &[(String, Option<Value>)] {
        let start = self.last_valid_scope(frame_offset);
        let end = self.stack_len() - frame_offset;
        &self.frames[start..end]
    }

    /// Get a reference to the all valid scopes
    /// Will act as if the stack is frame_offset frames up from the current frame
    fn get_valid_scopes_mut(&mut self, frame_offset: usize) -> &mut [(String, Option<Value>)] {
        let start = self.last_valid_scope(frame_offset);
        let end = self.stack_len() - frame_offset;
        &mut self.frames[start..end]
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
    /// Will act as if the stack is frame_offset frames up from the current frame
    pub fn get(&self, name: &str, frame_offset: usize) -> Option<&Value> {
        for (k, v) in self.get_valid_scopes(frame_offset).iter().rev() {
            if name == k {
                return v.as_ref();
            }
        }
        self.get_global(name)
    }

    /// Get a value from the stack
    pub fn get_mut(&mut self, name: &str, frame_offset: usize) -> Option<&mut Value> {
        let _self: *mut Self = self;
        for (k, v) in unsafe { &mut *_self }
            .get_valid_scopes_mut(frame_offset)
            .iter_mut()
            .rev()
        {
            if name == k {
                return v.as_mut();
            }
        }

        unsafe { &mut *_self }.globals.get_mut(name)
    }

    /// Write a value to the stack
    pub fn set(&mut self, name: &str, frame_offset: usize, value: Value) {
        if let Some(v) = self.get_mut(name, frame_offset) {
            *v = value;
        } else {
            self.set_top(name, value);
        }
    }

    /// Write a value to the top of the stack
    /// Will prioritize writing to existing empty slots over expanding the stack
    fn set_top(&mut self, name: &str, value: Value) {
        let current_framestart = self.frame_starts.last().map(|s| *s).unwrap_or_default();
        let total_entries = self.frames.len();
        for (k, v) in self
            .frames
            .iter_mut()
            .rev()
            .take(total_entries - current_framestart)
        {
            if v.is_none() {
                *k = name.to_string();
                *v = Some(value);
                return;
            }
        }
        self.frames.push((name.to_string(), Some(value)));
    }

    /// Deletes a value from the stack
    pub fn delete(&mut self, name: &str, frame_offset: usize) -> Option<Value> {
        self.prune_frames();
        for (k, v) in self.get_valid_scopes_mut(frame_offset).iter_mut().rev() {
            if name == k {
                return v.take();
            }
        }
        self.globals.remove(name)
    }

    /// Returns all variables in the state that are valid for reading
    pub fn all_variables_in_scope(&self, frame_offset: usize) -> HashMap<&str, &Value> {
        let mut variables = HashMap::new();
        variables.extend(self.globals.iter().map(|(k, v)| (k.as_str(), v)));
        for (k, v) in self.get_valid_scopes(frame_offset) {
            if let Some(v) = v {
                variables.insert(k.as_str(), v);
            }
        }
        variables
    }

    /// Returns all variables in the state, regardless of locks
    pub fn all_variables(&self) -> HashMap<&str, &Value> {
        let mut variables = HashMap::new();
        variables.extend(self.globals.iter().map(|(k, v)| (k.as_str(), v)));
        for (k, v) in &self.frames {
            if let Some(v) = v {
                variables.insert(k.as_str(), v);
            }
        }
        variables
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_samescope() {
        let mut mm = MemoryManager::new();

        // {a:0} [1]
        mm.set("a", 0, Value::from(1));
        mm.set_global("a", Value::from(0));
        assert_eq!(mm.get("a", 0), Some(&Value::from(1)));

        // {a:0} [1]
        mm.scope_into().unwrap();
        assert_eq!(mm.get("a", 0), Some(&Value::from(1)));

        // {a:0} [1, 2]
        mm.set("a", 0, Value::from(2));
        assert_eq!(mm.get("a", 0), Some(&Value::from(2)));

        // {a:0} [1]
        mm.scope_out();
        assert_eq!(mm.get("a", 0), Some(&Value::from(1)));

        // {a:0} []
        mm.scope_out();
        assert_eq!(mm.get("a", 0), Some(&Value::from(0)));
    }

    #[test]
    fn prune_frames() {}

    #[test]
    fn stack_size() {}

    #[test]
    fn test_unsafely_sort_compress_stack() {}

    #[test]
    fn test_reset() {}

    #[test]
    fn test_stack_len() {}

    #[test]
    fn test_scope_into() {}

    #[test]
    fn test_scope_out() {}

    #[test]
    fn test_last_valid_scope() {}

    #[test]
    fn test_get_valid_scopes() {}

    #[test]
    fn test_get_valid_scopes_mut() {}

    #[test]
    fn test_set_global() {}

    #[test]
    fn test_get_global() {}

    #[test]
    fn test_get() {}

    #[test]
    fn test_get_mut() {}

    #[test]
    fn test_set() {}

    #[test]
    fn test_set_top() {}

    #[test]
    fn test_delete() {
        let mut mm = MemoryManager::new();
        mm.set("a", 0, Value::from(1));
        mm.scope_into().unwrap();
        mm.lock_scope();
        mm.set("b", 0, Value::from(2));
        mm.scope_into().unwrap();
        mm.set_global("c", Value::from(3));
        mm.set_global("a", Value::from(3));

        assert_eq!(mm.delete("a", 2), Some(Value::from(1)));
        assert_eq!(mm.delete("b", 0), Some(Value::from(2)));
        assert_eq!(mm.delete("c", 0), Some(Value::from(3)));
        assert_eq!(mm.delete("a", 0), Some(Value::from(3)));
    }

    #[test]
    fn test_all_variables_in_scope() {
        let mut mm = MemoryManager::new();
        mm.set("a", 0, Value::from(1));
        mm.scope_into().unwrap();
        mm.lock_scope();
        mm.set("b", 0, Value::from(2));
        mm.scope_into().unwrap();
        mm.set_global("c", Value::from(3));
        mm.set_global("a", Value::from(3));

        let vars = mm.all_variables_in_scope(0);
        assert_eq!(vars.len(), 3);
        assert_eq!(vars.get("a"), Some(&&Value::from(3)));
        assert_eq!(vars.get("b"), Some(&&Value::from(2)));
        assert_eq!(vars.get("c"), Some(&&Value::from(3)));
    }

    #[test]
    fn test_all_variables() {
        let mut mm = MemoryManager::new();
        mm.set("a", 0, Value::from(1));
        mm.scope_into().unwrap();
        mm.lock_scope();
        mm.set("b", 0, Value::from(2));
        mm.scope_into().unwrap();
        mm.set_global("c", Value::from(3));
        mm.set_global("a", Value::from(3));

        let vars = mm.all_variables();
        assert_eq!(vars.len(), 3);
        assert_eq!(vars.get("a"), Some(&&Value::from(1)));
        assert_eq!(vars.get("b"), Some(&&Value::from(2)));
        assert_eq!(vars.get("c"), Some(&&Value::from(3)));
    }
}
