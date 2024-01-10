use crate::{
    network_utils::ApiManager,
    std_functions::{self, Function},
    user_function::UserFunction,
    Error, Token, Value,
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct State {
    /// Current depth of the parser
    /// This is used to prevent infinite recursion
    /// while parsing user_functions
    depth: usize,

    /// The time that the current parse started
    /// This is used to prevent infinite loops
    /// and implement a timeout
    parse_starttime: std::time::Instant,
    timeout: u64,

    /// Registered variables
    /// Used as a stack for scoping
    variables: Vec<HashMap<String, Value>>,

    /// Registered user functions
    /// These are functions that are set with
    /// `name(arguments) = body`
    user_functions: HashMap<String, UserFunction>,

    /// Registered std functions
    /// These are functions that are built into the parser
    /// and are not user defined
    std_functions: HashMap<String, Function>,
}

impl State {
    const MAX_DEPTH: usize = 999;

    pub fn with_timeout(seconds: u64) -> Self {
        let mut instance = Self {
            depth: 0,
            parse_starttime: std::time::Instant::now(),
            timeout: seconds,
            variables: vec![HashMap::new()],
            user_functions: HashMap::new(),
            std_functions: HashMap::new(),
        };

        ApiManager::default_apis(&mut instance);
        std_functions::register_all(&mut instance.std_functions);

        instance
    }

    /// Creates a new parser state
    pub fn new() -> Self {
        Self::with_timeout(0)
    }

    /// Returns the current depth of the parser
    pub fn current_depth(&self) -> usize {
        self.depth
    }

    /// Sets the timeout of the parser
    /// Used on parse start
    pub fn start_timer(&mut self) {
        self.parse_starttime = std::time::Instant::now();
    }

    /// Checks the timeout of the parser
    pub fn check_timer(&self) -> Result<(), Error> {
        if self.timeout > 0 && self.parse_starttime.elapsed().as_secs() > self.timeout {
            Err(Error::Timeout)
        } else {
            Ok(())
        }
    }

    /// Sets the depth to 0, and destroys all scopes but the root scope
    pub fn sanitize_scopes(&mut self) {
        self.depth = 0;
        self.variables.truncate(1);
    }

    /// Creates a new scope from this state
    /// A limit is placed on the depth of scopes that can be created
    /// This is to prevent infinite recursion
    pub fn scope_into(&mut self, token: &Token) -> Result<(), Error> {
        if self.depth >= Self::MAX_DEPTH {
            return Err(Error::StackOverflow {
                token: token.clone(),
            });
        }

        self.depth += 1;
        //println!("Depth: {}", self.depth);
        self.variables.push(HashMap::new());
        Ok(())
    }

    /// Removes the current scope from this state
    pub fn scope_out(&mut self) {
        if self.depth == 0 {
            return;
        }
        self.depth -= 1;
        self.variables.pop();
    }

    /// Assigns a variable in the state, in the root scope
    pub fn global_assign_variable(&mut self, name: &str, value: Value) {
        self.variables[0].insert(name.to_string(), value);
    }

    /// Sets a variable in the state
    pub fn set_variable(&mut self, name: &str, value: Value) {
        self.variables
            .last_mut()
            .unwrap()
            .insert(name.to_string(), value);
    }

    /// Returns the value of a variable
    pub fn get_variable(&self, name: &str) -> Option<Value> {
        for scope in self.variables.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Some(value.clone());
            }
        }
        None
    }

    pub fn delete_variable(&mut self, name: &str) -> Option<Value> {
        for scope in self.variables.iter_mut().rev() {
            if let Some(value) = scope.remove(name) {
                return Some(value);
            }
        }
        None
    }

    /// Returns all variables in the state
    pub fn all_variables(&self) -> HashMap<String, Value> {
        let mut variables = HashMap::new();
        for scope in self.variables.iter().rev() {
            variables.extend(scope.iter().map(|(k, v)| (k.clone(), v.clone())));
        }

        variables
    }

    /// Sets a user function in the state
    pub fn set_user_function(&mut self, function: UserFunction) {
        self.user_functions
            .insert(function.name().to_string(), function);
    }

    /// Deletes a user function from the state
    pub fn delete_user_function(&mut self, name: &str) -> Option<UserFunction> {
        self.user_functions.remove(name)
    }

    /// Returns the user function with the given name
    pub fn get_user_function(&self, name: &str) -> Option<UserFunction> {
        self.user_functions.get(name).cloned()
    }

    /// Returns the std function with the given name
    pub fn get_std_function(&self, name: &str) -> Option<Function> {
        self.std_functions.get(name).cloned()
    }

    /// Returns the extension function with the given name
    /// If the `extensions` feature is not enabled, this function will always return `None`
    #[allow(unused_variables)]
    pub fn get_ext_function(&self, name: &str) -> Option<Function> {
        #[cfg(not(feature = "extensions"))]
        return None;

        #[cfg(feature = "extensions")]
        crate::extensions::ExtensionController::with(|controller| {
            controller.get_function(name).clone()
        })
    }

    /// Registers a function in the state
    /// This function will overwrite any existing function with the same name
    pub fn register_function(&mut self, function: Function) {
        self.std_functions
            .insert(function.name().to_string(), function);
    }

    // We will have a function that searches the whole state for a matching function
    // And builds a temporary Function object that calls the correct code
    // this is where we should handle the special case of the `help` function

    // We will search for functions in the following order:
    // 1. The function named 'help' which is a special case
    // 2. Standard library functions
    // 3. User defined functions
    // 4. JS Extension-provided functions
    pub fn get_function(&self, name: &str) -> Option<Function> {
        if name == "help" {
            todo!();
        } else if let Some(f) = self.get_std_function(name) {
            Some(f)
        } else if let Some(f) = self.get_user_function(name) {
            Some(f.to_std_function())
        } else if let Some(f) = self.get_ext_function(name) {
            Some(f)
        } else {
            None
        }
    }

    /// Decorates the given value with the given decorator
    pub fn decorate(&mut self, name: &str, token: &Token, value: Value) -> Result<String, Error> {
        let mut value = value;

        if let Some(f) = self.get_function(&format!("@{name}")) {
            value = f.execute(self, vec![value.clone()], token)?;
            Ok(value.to_string())
        } else {
            // Check for extension decorators
            #[cfg(feature = "extensions")]
            if let Some(decorator) = self.get_ext_function(&format!("@{name}")) {
                value = decorator.execute(self, vec![value.clone()], token)?;
                return Ok(value.to_string());
            }

            Err(Error::DecoratorName {
                name: name.to_string(),
                token: token.clone(),
            })
        }
    }

    /// Returns a string containing the help for all functions
    pub fn help(&self, filter: Option<String>) -> String {
        let mut map = self.std_functions.clone();
        map.extend(
            self.user_functions
                .iter()
                .map(|(n, f)| (n.clone(), f.to_std_function())),
        );

        let mut help = Vec::new();

        let strings = std_functions::collect_help(map, filter.clone());
        help.push(std_functions::help_to_string(strings));

        #[cfg(feature = "extensions")]
        help.push(crate::extensions::ExtensionController::with(|controller| {
            let ext_fn_map = controller
                .functions()
                .iter()
                .map(|f| (f.name().to_string(), f.clone()))
                .collect::<HashMap<String, Function>>();
            let ext_fn_strings = std_functions::collect_help(ext_fn_map, filter.clone());
            std_functions::help_to_string(ext_fn_strings)
        }));

        let user_fn_map = self
            .user_functions
            .iter()
            .map(|(n, f)| (n.clone(), f.to_std_function()))
            .collect::<HashMap<String, Function>>();
        let user_fn_strings = std_functions::collect_help(user_fn_map, filter.clone());
        help.push(std_functions::help_to_string(user_fn_strings));

        help.join("\n")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_scoped_delete() {
        let mut state = State::new();
        state.set_variable("a", Value::from(2.0));
        state
            .scope_into(&Token {
                rule: crate::pest::Rule::ATOMIC_VALUE,
                input: "".to_string(),
                references: None,
            })
            .ok();
        state.delete_variable("a");
        state.scope_out();

        assert_eq!(state.get_variable("a"), None);
    }
}