use super::{extension::ExtensionDetails, worker::ExtensionWorker};
use crate::{state::State, std_functions::Function, token, Error, Value};
use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

// This should allow the following to be enforced:
// - Runtime is only initialized once
// - Runtime is never accessed concurrently
static RUNTIME_CELL: OnceLock<Mutex<ExtensionController>> = OnceLock::new();

pub struct ExtensionController {
    /// Stores the actual extension worker threads
    /// indexed by filename
    extensions: HashMap<String, ExtensionWorker>,

    /// Maps function names to their respective extensions
    /// for faster lookup
    function_map: HashMap<String, String>,
}

impl ExtensionController {
    /// Create a new extension controller
    pub fn new() -> Self {
        Self {
            extensions: HashMap::new(),
            function_map: HashMap::new(),
        }
    }

    /// Execute some code on the runtime instance
    pub fn exec(code: &str) -> Result<Value, Error> {
        let result: serde_json::Value = rustyscript::evaluate(code)?;
        Ok(Value::try_from(result)?)
    }

    /// Perform an operation on the runtime instance
    /// Will return T if we can get access to the runtime
    /// or panic went wrong
    pub fn with<T, F: FnOnce(&mut ExtensionController) -> T>(callback: F) -> T {
        let mutex = RUNTIME_CELL.get_or_init(|| Mutex::new(ExtensionController::new()));
        let mut guard = mutex.lock().unwrap();
        callback(&mut *guard)
    }

    /// Register an extension
    pub fn register(&mut self, filename: &str) -> Result<ExtensionDetails, Error> {
        let worker = ExtensionWorker::new(filename)?;

        // Update the function map
        for name in &worker.extension().function_names() {
            self.function_map.insert(name.clone(), filename.to_string());
        }

        let extension = worker.extension().clone();
        self.extensions.insert(filename.to_string(), worker);
        Ok(extension)
    }

    /// Unregister an extension
    pub fn unregister(&mut self, filename: &str) {
        if let Some(extension) = self.extensions.remove(filename) {
            for name in &extension.extension().function_names() {
                self.function_map.remove(name);
            }

            extension.stop();
        }
    }

    pub fn call_function(
        &self,
        name: &str,
        args: &[Value],
        state: &mut State,
        token: &token::Token,
    ) -> Result<Value, Error> {
        self.extensions
            .get(self.function_map.get(name).unwrap())
            .unwrap()
            .call_function(name, args, state, token)
    }

    /// Return the function with the given name
    pub fn get_function(&self, name: &str) -> Option<Function> {
        self.function_map
            .get(name)
            .and_then(|extension_name| self.extensions.get(extension_name))
            .and_then(|extension| extension.to_std_function(name))
    }

    /// Returns all functions from all extensions
    pub fn functions(&self) -> Vec<Function> {
        let mut functions: Vec<Function> = Vec::new();
        for (function_name, extension_name) in self.function_map.iter() {
            let extension = self.extensions.get(extension_name).unwrap();
            let function = extension.to_std_function(function_name).unwrap();
            functions.push(function);
        }
        functions
    }
}
