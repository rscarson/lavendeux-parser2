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
    /// Current depth of the parser
    /// This is used to prevent infinite recursion
    /// while parsing user_functions
    depth: usize,
    locked: usize,

    /// The time that the current parse started
    /// This is used to prevent infinite loops
    /// and implement a timeout
    parse_starttime: Instant,
    timeout: Duration,

    /// Registered variables
    /// Used as a stack for scoping
    variables: Vec<HashMap<String, Value>>,

    /// Registered functions
    functions: HashMap<String, Box<dyn ParserFunction>>,
}

impl Default for State {
    fn default() -> Self {
        let stdlib_fns = stdlib::all();
        let mut instance = Self {
            depth: 0,
            locked: 0,
            parse_starttime: std::time::Instant::now(),
            timeout: Duration::from_secs(0),
            variables: vec![HashMap::new()],

            functions: stdlib_fns,
        };

        ApiRegistry::populate_defaults(&mut instance);

        instance
    }
}

impl State {
    const MAX_DEPTH: usize = 999;

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
     * Scope handling functions
     *
     */

    /// Returns the current depth of the parser
    pub fn current_depth(&self) -> usize {
        self.depth
    }

    /// Sets the depth to 0, and destroys all scopes but the root scope
    pub fn sanitize_scopes(&mut self) {
        self.depth = 0;
        self.locked = 0;
        self.variables.truncate(1);
    }

    /// Creates a new scope from this state
    /// A limit is placed on the depth of scopes that can be created
    /// This is to prevent infinite recursion
    pub fn scope_into(&mut self) -> Result<(), Error> {
        if self.depth >= Self::MAX_DEPTH {
            return Err(ErrorDetails::StackOverflow.into());
        }

        self.depth += 1;
        self.variables.push(HashMap::new());
        Ok(())
    }

    /// Locks the current scope, preventing access to variables in higher scopes
    pub fn lock_scope(&mut self) {
        self.locked = self.depth;
    }

    /// Unlocks the current scope, granting access to variables in higher scopes
    pub fn unlock_scope(&mut self) {
        self.locked = 0;
    }

    /// Removes the current scope from this state
    pub fn scope_out(&mut self) {
        if self.depth == 0 {
            return;
        }
        self.depth -= 1;
        self.variables.pop();

        if self.depth < self.locked {
            self.unlock_scope();
        }
    }

    fn get_valid_scopes(
        &self,
    ) -> std::iter::Take<std::iter::Rev<std::slice::Iter<'_, HashMap<String, polyvalue::Value>>>>
    {
        self.variables
            .iter()
            .rev()
            .take(self.depth - self.locked + 1)
    }

    fn get_valid_scopes_mut(
        &mut self,
    ) -> std::iter::Take<std::iter::Rev<std::slice::IterMut<'_, HashMap<String, polyvalue::Value>>>>
    {
        self.variables
            .iter_mut()
            .rev()
            .take(self.depth - self.locked + 1)
    }

    /**
     *
     * Variable handling functions
     *
     */

    /// Assigns a variable in the state, in the root scope
    pub fn global_assign_variable(&mut self, name: &str, value: Value) {
        self.variables[0].insert(name.to_string(), value);
    }

    /// Gets a variable from the root scope
    pub fn global_get_variable(&self, name: &str) -> Option<Value> {
        self.variables[0].get(name).cloned()
    }

    /// Sets a variable in the a scope offset levels from the current scope
    /// If that scope does not exist, the variable is not set
    pub fn set_variable_in_offset(&mut self, offset: usize, name: &str, value: Value) {
        if let Some(scope) = self.variables.iter_mut().rev().nth(offset) {
            scope.insert(name.to_string(), value);
        }
    }

    /// Sets a variable as if it were in the parent scope
    /// Bypasses the scope lock
    pub fn set_variable_as_parent(&mut self, name: &str, value: Value) {
        let mut scopes = self.get_valid_scopes_mut();
        scopes.next();
        for scope in scopes {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return;
            }
        }

        // If the variable is not found, assign it in the current scope
        self.set_variable_in_offset(1, name, value)
    }

    /// Sets a variable in the state
    pub fn set_variable(&mut self, name: &str, value: Value) {
        for scope in self.get_valid_scopes_mut() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return;
            }
        }

        // If the variable is not found, assign it in the root scope
        self.set_variable_in_scope(name, value)
    }

    /// Sets a variable in the current scope
    pub fn set_variable_in_scope(&mut self, name: &str, value: Value) {
        self.variables
            .last_mut()
            .unwrap()
            .insert(name.to_string(), value);
    }

    /// Returns the value of a variable
    pub fn get_variable(&self, name: &str) -> Option<Value> {
        for scope in self.get_valid_scopes() {
            if let Some(value) = scope.get(name) {
                return Some(value.clone());
            }
        }
        None
    }

    /// Deletes a variable from the state
    pub fn delete_variable(&mut self, name: &str) -> Option<Value> {
        for scope in self.get_valid_scopes_mut() {
            if let Some(value) = scope.remove(name) {
                return Some(value);
            }
        }
        None
    }

    /// Returns all variables in the state
    pub fn all_variables(&self) -> HashMap<String, Value> {
        let mut variables = HashMap::new();
        for scope in self.get_valid_scopes() {
            variables.extend(scope.iter().map(|(k, v)| (k.clone(), v.clone())));
        }

        variables
    }

    /// Returns all variables in the state
    /// Ignores the scope lock
    pub fn all_variables_unscoped(&self) -> HashMap<String, Value> {
        let mut variables = HashMap::new();
        for scope in self.variables.iter().rev() {
            variables.extend(scope.iter().map(|(k, v)| (k.clone(), v.clone())));
        }

        variables
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
            return oops!(ReadOnlyFunction {
                name: name.to_string()
            });
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
    pub fn get_function(&self, name: &str) -> Option<&Box<dyn ParserFunction>> {
        self.functions.get(name)
    }

    /// List all functions in the state
    pub fn all_functions(&self) -> &HashMap<String, Box<dyn ParserFunction>> {
        &self.functions
    }

    /// Calls a function in the state
    /// arg1_references maps to the references field of the source [crate::Token]
    pub fn call_function(
        &mut self,
        name: &str,
        args: Vec<Value>,
        arg1_references: Option<&str>,
    ) -> Result<Value, Error> {
        let function = self
            .get_function(name)
            .ok_or(ErrorDetails::FunctionName {
                name: name.to_string(),
            })?
            .clone_self();

        function.exec(&args, self, arg1_references)
    }

    /// Calls a decorator function
    pub fn decorate(&mut self, name: &str, value: Value) -> Result<String, Error> {
        let name = format!("@{name}");
        match self.call_function(&name, vec![value], None) {
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

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_scope() {
        let mut state = State::new();
        state.set_variable("a", Value::from(2.0));
        state.scope_into().ok();
        assert_eq!(state.delete_variable("a"), Some(Value::from(2.0)));
        assert_eq!(state.delete_variable("a"), None);

        state.global_assign_variable("b", Value::from(2.0));

        assert_eq!(state.current_depth(), 1);
        state.scope_out();
        assert_eq!(state.current_depth(), 0);

        state.scope_out();
        assert_eq!(state.current_depth(), 0);

        assert_eq!(state.get_variable("a"), None);
        assert_eq!(state.get_variable("b"), Some(Value::from(2.0)));

        state.depth = State::MAX_DEPTH;
        assert!(matches!(
            state.scope_into().unwrap_err().details,
            ErrorDetails::StackOverflow { .. }
        ));
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
        state.set_variable("a", Value::from(2.0));
        state.scope_into().ok();
        state.set_variable("b", Value::from(3.0));

        let variables = state.all_variables();
        assert!(variables.contains_key("a"));
        assert!(variables.contains_key("b"));
    }
}
