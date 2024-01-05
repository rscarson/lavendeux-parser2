use super::{
    extension::Extension,
    runtime::{ExtensionRuntime, VariableState},
    ExtensionController,
};
use crate::{
    flatten_arguments,
    state::State,
    std_functions::{Argument, Function},
    Error, Value,
};
use std::{
    sync::mpsc::{channel, Receiver, Sender},
    thread,
};

fn runtime_thread(
    extension_filename: String,
    request_rx: Receiver<ExtensionWorkerMessage>,
    response_tx: Sender<ExtensionWorkerResponse>,
) {
    let runtime = ExtensionRuntime::new(&extension_filename);
    match runtime {
        Ok(mut runtime) => {
            let extension: Extension = runtime.extension().clone().into();
            response_tx
                .send(ExtensionWorkerResponse::Start(extension))
                .unwrap();

            loop {
                let message = request_rx.recv().unwrap();
                match message {
                    ExtensionWorkerMessage::Shutdown => return,
                    ExtensionWorkerMessage::CallFunction {
                        function,
                        args,
                        mut state,
                    } => {
                        let result = runtime.call_function(&mut state, &function, &args);
                        response_tx
                            .send(ExtensionWorkerResponse::CallFunction { result, state })
                            .unwrap();
                    }
                }
            }
        }
        Err(err) => {
            response_tx
                .send(ExtensionWorkerResponse::Error(err.into()))
                .unwrap();
            return;
        }
    };
}

/// This structure will store a threaded instance of a JS runtime
/// which will listen for messages from a channel in order to call
/// functions and return values
pub struct ExtensionWorker {
    thread: thread::JoinHandle<()>,
    request: Sender<ExtensionWorkerMessage>,
    response: Receiver<ExtensionWorkerResponse>,
    extension: Extension,
}

enum ExtensionWorkerMessage {
    CallFunction {
        function: String,
        args: Vec<Value>,
        state: VariableState,
    },
    Shutdown,
}

enum ExtensionWorkerResponse {
    CallFunction {
        result: Result<Value, Error>,
        state: VariableState,
    },
    Start(Extension),
    Error(Error),
}

impl ExtensionWorker {
    /// Create a new worker thread
    ///
    /// # Arguments
    /// * `extension_filename` - Path to the extension file
    ///
    /// # Returns
    /// * `Result<ExtensionWorker, Error>` - The worker thread
    pub fn new(extension_filename: &str) -> Result<Self, Error> {
        let (req_tx, req_rx) = channel::<ExtensionWorkerMessage>();
        let (res_tx, res_rx) = channel::<ExtensionWorkerResponse>();

        let filename = extension_filename.to_string();
        let join_handle = thread::spawn(move || {
            runtime_thread(filename, req_rx, res_tx);
        });

        let response = res_rx.recv().unwrap();
        let extension = match response {
            ExtensionWorkerResponse::Start(extension) => extension,
            ExtensionWorkerResponse::Error(err) => return Err(err),
            _ => {
                return Err(Error::Internal(format!(
                    "JSRuntime worker responded incorrectly"
                )))
            }
        };

        Ok(Self {
            response: res_rx,
            request: req_tx,
            thread: join_handle,
            extension,
        })
    }

    /// Stop the worker thread
    pub fn stop(self) {
        self.request.send(ExtensionWorkerMessage::Shutdown).unwrap();
        self.thread.join().unwrap();
    }

    /// Call a function from the extension
    ///
    /// # Arguments
    /// * `function` - Function name
    /// * `args` - Values to pass in
    /// * `state` - State to pass in
    ///
    /// # Returns
    /// * `Result<Value, Error>` - The result of the function call
    pub fn call_function(
        &self,
        function: &str,
        args: &[Value],
        cur_state: &mut State,
    ) -> Result<Value, Error> {
        self.request
            .send(ExtensionWorkerMessage::CallFunction {
                function: function.to_string(),
                args: args.to_vec(),
                state: cur_state.all_variables(),
            })
            .unwrap();

        match self.response.recv().unwrap() {
            ExtensionWorkerResponse::CallFunction { result, state } => {
                for (key, value) in state {
                    cur_state.set_variable(&key, value).ok();
                }
                result
            }
            ExtensionWorkerResponse::Error(err) => Err(err),
            _ => Err(Error::Internal(format!(
                "JSRuntime worker responded incorrectly"
            ))),
        }
    }

    /// Returns the extension
    pub fn extension(&self) -> &Extension {
        &self.extension
    }

    pub fn to_std_function(&self, function: &str) -> Option<Function> {
        if let Some(function) = self.extension.functions.get(function) {
            Some(Function::new(
                &function.name,
                "",
                &format!("Extension: {}", self.extension.name),
                function
                    .arguments
                    .iter()
                    .enumerate()
                    .map(|(i, arg)| Argument {
                        name: format!("arg{}", i),
                        optional: false,
                        plural: false,
                        expects: *arg,
                    })
                    .collect(),
                function.returns,
                |state, args, _token, name| {
                    ExtensionController::with(|controller| {
                        controller.call_function(name, &flatten_arguments!(args), state)
                    })
                },
                function.name.clone(),
            ))
        } else {
            None
        }
    }
}
