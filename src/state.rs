use crate::{
    documentation::{DocumentationFormatter, PlaintextFormatter},
    error::ErrorDetails,
    functions::{stdlib, ParserFunction},
    network::ApiRegistry,
    Error, Value,
};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

/// The main parser state
/// Stores variables, scoping data, functions, and metadata about the current parse
#[derive(Debug)]
pub struct State {
    /// The state's stackframes
    /// Stores variables in a stack of scopes
    /// without resorting to a recursive data structure
    stack: StateScopes,

    /// The time that the current parse started
    /// This is used to prevent infinite loops
    /// and implement a timeout
    parse_starttime: Instant,
    timeout: Duration,

    /// Registered functions
    functions: HashMap<String, Box<dyn ParserFunction>>,
}

impl Default for State {
    fn default() -> Self {
        let stdlib_fns = stdlib::all();
        let mut instance = Self {
            stack: StateScopes::new(),
            parse_starttime: std::time::Instant::now(),
            timeout: Duration::from_secs(0),

            functions: stdlib_fns,
        };

        ApiRegistry::populate_defaults(&mut instance);

        instance
    }
}

impl State {
    /// Creates a new parser state
    pub fn new() -> Self {
        Self::default()
    }

    /**
     *
     * Timeout handling functions
     *
     */

    /// Creates a new parser state with a timeout
    /// Parsing will fail with `ErrorDetails::Timeout` if the timeout is exceeded
    /// The timer does not begin until `start_timer` is called
    /// And is checked with `check_timer` (which is called internally by the parser)
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            timeout,
            ..Self::default()
        }
    }

    /// Sets the timeout of the parser
    /// Used on parse start
    pub fn start_timer(&mut self) {
        self.parse_starttime = Instant::now();
    }

    /// Checks the timeout of the parser
    pub fn check_timer(&self) -> Result<(), Error> {
        if !self.timeout.is_zero() && self.parse_starttime.elapsed() > self.timeout {
            Err(ErrorDetails::Timeout.into())
        } else {
            Ok(())
        }
    }

    /**
     *
     * Stack handling functions
     *
     */

    /// Read the state's variable stack
    pub fn stack(&self) -> &StateScopes {
        &self.stack
    }

    /// Mutate the state's variable stack
    pub fn stack_mut(&mut self) -> &mut StateScopes {
        &mut self.stack
    }

    /// Increase the depth of the stack
    pub fn scope_into(&mut self) -> Result<(), Error> {
        self.stack.scope_into()
    }

    /// Decrease the depth of the stack
    pub fn scope_out(&mut self) {
        self.stack.scope_out();
    }

    /// Lock the current scope
    pub fn lock_scope(&mut self) {
        self.stack.lock_scope();
    }

    /// Read a variable from the state
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.stack.get(name)
    }

    /// Write a value to the stack
    pub fn set(&mut self, name: &str, value: Value) {
        self.stack.set(name, value);
    }

    /**
     *
     * Function handling functions
     *
     */

    /// Returns true if the given function is a read-only system function
    pub fn is_system_function(&self, name: &str) -> bool {
        if let Some(function) = self.functions.get(name) {
            function.is_readonly()
        } else {
            false
        }
    }

    /// Registers a function in the state
    /// See [crate::define_stdfunction] for an example of how to define a function
    pub fn register_function(&mut self, function: impl ParserFunction) -> Result<(), Error> {
        let name = function.name();
        if self.is_system_function(name) {
            oops!(ReadOnlyFunction {
                name: name.to_string()
            })
        } else {
            self.functions
                .insert(name.to_string(), function.clone_self());
            Ok(())
        }
    }

    /// Unregisters a function from the state
    pub fn unregister_function(
        &mut self,
        name: &str,
    ) -> Result<Option<Box<dyn ParserFunction>>, Error> {
        if self.is_system_function(name) {
            oops!(ReadOnlyFunction {
                name: name.to_string()
            })
        } else {
            Ok(self.functions.remove(name))
        }
    }

    /// Returns a function from the state
    pub fn get_function(&self, name: &str) -> Option<&dyn ParserFunction> {
        self.functions.get(name).map(|f| f.as_ref())
    }

    /// Returns a function from the state
    pub fn get_function_mut(&mut self, name: &str) -> Option<&mut Box<dyn ParserFunction>> {
        self.functions.get_mut(name)
    }

    /// List all functions in the state
    pub fn all_functions(&self) -> &HashMap<String, Box<dyn ParserFunction>> {
        &self.functions
    }

    /// Calls a function in the state
    /// arg1_references maps to the references field of the source [crate::Token]
    pub fn call_function(&mut self, name: &str, args: Vec<Value>) -> Result<Value, Error> {
        let function = self.get_function(name).ok_or(ErrorDetails::FunctionName {
            name: name.to_string(),
        })?;
        let function = function.clone_self();
        function.exec(&args, self)
    }

    /// Calls a decorator function
    pub fn decorate(&mut self, name: &str, value: Value) -> Result<String, Error> {
        let name = format!("@{name}");
        match self.call_function(&name, vec![value]) {
            Ok(value) => Ok(value.to_string()),
            Err(e) if matches!(e.details, ErrorDetails::FunctionName { .. }) => {
                oops!(DecoratorName {
                    name: name.to_string()
                })
            }
            Err(e) => Err(e),
        }
    }

    /// Returns a string containing the help for all functions
    pub fn help(&self, filter: Option<String>) -> String {
        PlaintextFormatter.format_functions(self, filter.as_deref())
    }
}

/// Implementation of the stack of scopes for the parser state
#[derive(Debug, Clone, Default)]
pub struct StateScopes {
    globals: HashMap<String, Value>,
    frames: Vec<(String, Value)>,
    locks: Vec<usize>,
    frame_starts: Vec<usize>,
}
impl StateScopes {
    const MAX_DEPTH: usize = 999;

    /// Creates a blank stack
    pub fn new() -> Self {
        Self::default()
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
            while self.stack_len() < self.last_valid_scope() {
                self.unlock_scope();
            }
        }
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
    pub fn get_valid_scopes(&self) -> &[(String, Value)] {
        let start = self.last_valid_scope();
        &self.frames[start..]
    }

    /// Get a reference to the all valid scopes
    /// If ignore_lock is true, the last lock is ignored
    pub fn get_valid_scopes_mut(&mut self) -> &mut [(String, Value)] {
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
                return Some(v);
            }
        }
        None
    }

    /// Get a value from the stack
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Value> {
        for (k, v) in self.get_valid_scopes_mut().iter_mut().rev() {
            if name == k {
                return Some(v);
            }
        }
        None
    }

    /// Write a value to the stack
    pub fn set(&mut self, name: &str, value: Value) {
        if let Some(v) = self.get_mut(name) {
            *v = value;
        } else {
            self.set_top(name, value);
        }
    }

    /// Write a value to the top of the stack
    pub fn set_top(&mut self, name: &str, value: Value) {
        self.frames.push((name.to_string(), value));
    }

    /// Deletes a value from the stack
    pub fn delete(&mut self, name: &str) -> Option<Value> {
        let index = self
            .get_valid_scopes_mut()
            .iter()
            .rev()
            .position(|(k, _)| k == name);
        if let Some(index) = index {
            let index = self.last_valid_scope() + index;
            Some(self.frames.remove(index).1)
        } else {
            self.globals.remove(name)
        }
    }

    /// Returns all variables in the state that are valid for reading
    pub fn all_variables_in_scope(&self) -> HashMap<&str, &Value> {
        let mut variables = HashMap::new();
        for (k, v) in self.get_valid_scopes() {
            variables.insert(k.as_str(), v);
        }
        variables
    }

    /// Returns all variables in the state, regardless of locks
    pub fn all_variables(&self) -> HashMap<&str, &Value> {
        let mut variables = HashMap::new();
        for (k, v) in &self.frames {
            variables.insert(k.as_str(), v);
        }
        variables
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_scope() {
        let mut state = State::new();
        state.set("a", Value::from(2.0));
        state.scope_into().ok();
        assert_eq!(state.stack_mut().delete("a"), Some(Value::from(2.0)));
        assert_eq!(state.stack_mut().delete("a"), None);

        state.stack_mut().set_global("b", Value::from(2.0));

        state.scope_out();
        state.scope_out();

        assert_eq!(state.get("a"), None);
        assert_eq!(state.stack().get_global("b"), Some(&Value::from(2.0)));
    }

    #[test]
    fn test_timer() {
        let mut state = State::with_timeout(Duration::from_millis(100));
        state.start_timer();
        std::thread::sleep(std::time::Duration::from_millis(250));
        assert!(matches!(
            state.check_timer().unwrap_err().details,
            ErrorDetails::Timeout
        ));
    }

    #[test]
    fn test_all_variables() {
        let mut state = State::new();
        state.set("a", Value::from(2.0));
        state.scope_into().ok();
        state.set("b", Value::from(3.0));

        let variables = state.stack.all_variables();
        assert!(variables.contains_key("a"));
        assert!(variables.contains_key("b"));
    }
}
