use super::{extension::Extension, worker::ExtensionWorker};
use crate::{state::State, std_functions::Function, Error, Value};
use std::{
    cell::{OnceCell, RefCell},
    collections::HashMap,
};

// Create a thread-local version of the runtime
// This should allow the following to be enforced:
// - Runtime is not sent between threads
// - Runtime is only initialized once
// - Runtime is never accessed concurrently
thread_local! {
    static RUNTIME_CELL: OnceCell<RefCell<ExtensionController>> = OnceCell::new();
}

pub struct ExtensionController {
    /// Stores the actual extension worker threads
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
    pub fn with<T, F: FnMut(&mut ExtensionController) -> T>(mut callback: F) -> T {
        RUNTIME_CELL.with(|once_lock| {
            let rt_mut = once_lock.get_or_init(|| RefCell::new(ExtensionController::new()));
            let mut runtime = rt_mut.borrow_mut();
            callback(&mut runtime)
        })
    }

    /// Register an extension
    pub fn register(&mut self, filename: &str) -> Result<Extension, Error> {
        let worker = ExtensionWorker::new(filename)?;
        for (name, _) in worker.extension().functions.iter() {
            self.function_map
                .insert(name.clone(), worker.extension().name.clone());
        }

        let extension = worker.extension().clone();
        self.extensions
            .insert(worker.extension().name.clone(), worker);
        Ok(extension)
    }

    /// Unregister an extension
    pub fn unregister(&mut self, name: &str) {
        if let Some(extension) = self.extensions.remove(name) {
            for (function_name, _) in extension.extension().functions.iter() {
                self.function_map.remove(function_name);
            }

            extension.stop();
        }
    }

    pub fn call_function(
        &self,
        name: &str,
        args: &[Value],
        state: &mut State,
    ) -> Result<Value, Error> {
        self.extensions
            .get(self.function_map.get(name).unwrap())
            .unwrap()
            .call_function(name, args, state)
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
