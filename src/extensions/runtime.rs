use std::{collections::HashMap, time::Duration};

use polyvalue::Value;
use rustyscript::{deno_core::extension, Module, ModuleHandle, Runtime, RuntimeOptions};

use crate::{Error, Token};

use super::extension::ExtensionDetails;

extension!(
    lavendeux,
    esm_entry_point = "ext:lavendeux/lavendeux.ts",
    esm = [
        dir "src/extensions/js", "lavendeux.ts",
    ],
);

pub struct ExtensionRuntime {
    runtime: Runtime,
    extension: ExtensionDetails,
    handle: ModuleHandle,
}

impl ExtensionRuntime {
    const SCRIPT_TIMEOUT: u64 = 1000;

    pub fn new(filename: &str) -> Result<Self, Error> {
        // Start the runtime
        let mut inner = Runtime::new(RuntimeOptions {
            timeout: Duration::from_millis(Self::SCRIPT_TIMEOUT),
            extensions: vec![lavendeux::init_ops_and_esm()],
            ..Default::default()
        })?;

        // Load the module
        let module = Module::load(filename)?;
        let handle = inner.load_module(&module)?;

        // Extract extension details
        let extension: ExtensionDetails = inner.call_function(&handle, "lavendeuxExport", &[])?;

        Ok(Self {
            runtime: inner,
            extension,
            handle,
        })
    }

    pub fn call_function(
        &mut self,
        name: &str,
        args: &[Value],
        variables: &mut HashMap<String, Value>,
        token: &Token,
    ) -> Result<Value, Error> {
        self.extension.call_function(
            &mut self.runtime,
            &self.handle,
            name,
            args,
            variables,
            token,
        )
    }

    pub fn extension_details(&self) -> &ExtensionDetails {
        &self.extension
    }
}
