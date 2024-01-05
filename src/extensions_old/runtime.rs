use super::js_extension::{JsExtension, JsExtensionFunction};
use crate::{Error, Value};
use rustyscript::{deno_core::extension, json_args, Module, ModuleHandle, Runtime, RuntimeOptions};
use std::{collections::HashMap, time::Duration};

pub type VariableState = HashMap<String, Value>;

extension!(
    lavendeux,
    esm_entry_point = "ext:lavendeux/entrypoint.ts",
    esm = [
        dir "src/extensions/js", "entrypoint.ts", "runtime.ts", "extension.ts", "value.js",
    ],
);

pub struct ExtensionRuntime {
    runtime: Runtime,
    handle: ModuleHandle,
    extension: JsExtension,
}

impl ExtensionRuntime {
    const SCRIPT_TIMEOUT: u64 = 1000;

    pub fn new(filename: &str) -> Result<Self, rustyscript::Error> {
        // Start the runtime
        let mut inner = Runtime::new(RuntimeOptions {
            timeout: Duration::from_millis(Self::SCRIPT_TIMEOUT),
            default_entrypoint: Some("extension".to_string()),
            extensions: vec![lavendeux::init_ops_and_esm()],
        })?;

        // Load the module
        let module = Module::load(filename)?;
        let handle = inner.load_module(&module)?;

        // Extract extension details
        let extension: JsExtension = inner.call_entrypoint(&handle, json_args!())?;
        println!("Loaded extension: {:?}", extension);

        Ok(Self {
            runtime: inner,
            handle,
            extension,
        })
    }

    pub fn extension(&self) -> &JsExtension {
        &self.extension
    }

    pub fn call_function(
        &mut self,
        state: &mut VariableState,
        name: &str,
        args: &[Value],
    ) -> Result<Value, Error> {
        let function = self
            .extension
            .functions
            .get(name)
            .ok_or(Error::FunctionName {
                name: name.to_string(),
            })?
            .clone();

        match function {
            JsExtensionFunction::Legacy(f) => Ok(self.call_legacy_function(&f, args)?),
            JsExtensionFunction::Standard(_) => Ok(self.call_standard_function(name, state, args)?),
        }
    }

    fn call_legacy_function(
        &mut self,
        function: &str,
        args: &[Value],
    ) -> Result<Value, rustyscript::Error> {
        let mut _args = serde_json::to_value(args)?;
        self.runtime
            .call_function::<Value>(&self.handle, function, &[_args])
    }

    fn call_standard_function(
        &mut self,
        function: &str,
        state: &mut VariableState,
        args: &[Value],
    ) -> Result<Value, rustyscript::Error> {
        // Inject parser state
        let json_variables = serde_json::to_value(state.clone())?;
        self.runtime.call_function(
            &self.handle,
            "setLavendeuxState",
            json_args!(json_variables),
        )?;

        // Decode arguments
        let mut _args: Vec<serde_json::Value> = vec![];
        _args.push(serde_json::to_value(function)?);
        for arg in args {
            _args.push(serde_json::to_value(arg)?);
        }

        // Call the function
        let result: Value =
            self.runtime
                .call_function(&self.handle, "callLavendeuxFunction", _args.as_slice())?;

        // Pull out modified state
        let new_state: HashMap<String, Value> =
            self.runtime
                .call_function(&self.handle, "getLavendeuxState", json_args!())?;
        for k in new_state.keys() {
            state.insert(k.to_string(), new_state.get(k).unwrap().clone());
        }

        Ok(result)
    }
}
