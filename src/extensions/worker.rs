use rustyscript::Module;

use super::{
    controller::ExtensionController, extension::ExtensionDetails, runtime::ExtensionRuntime,
};
use crate::{
    error::ExternalError,
    flatten_arguments,
    state::State,
    std_functions::{Argument, Function},
    Error, Token, Value,
};
use std::{
    sync::mpsc::{channel, Receiver, Sender},
    thread,
};

type VariableState = std::collections::HashMap<String, Value>;

fn runtime_thread(
    extension_module: Module,
    request_rx: Receiver<ExtensionWorkerMessage>,
    response_tx: Sender<ExtensionWorkerResponse>,
) {
    let runtime = ExtensionRuntime::new(extension_module);
    match runtime {
        Ok(mut runtime) => {
            let meta = runtime.extension_details();
            response_tx
                .send(ExtensionWorkerResponse::Start(meta.clone()))
                .unwrap();

            loop {
                let message = request_rx.recv().unwrap();
                match message {
                    ExtensionWorkerMessage::Shutdown => return,
                    ExtensionWorkerMessage::CallFunction {
                        function,
                        args,
                        mut state,
                        token,
                    } => {
                        let result = runtime.call_function(&function, &args, &mut state, &token);
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
    extension: ExtensionDetails,
}

enum ExtensionWorkerMessage {
    CallFunction {
        function: String,
        args: Vec<Value>,
        state: VariableState,
        token: Token,
    },
    Shutdown,
}

enum ExtensionWorkerResponse {
    CallFunction {
        result: Result<Value, Error>,
        state: VariableState,
    },
    Start(ExtensionDetails),
    Error(ExternalError),
}

impl ExtensionWorker {
    /// Create a new worker thread
    ///
    /// # Arguments
    /// * `extension_filename` - Path to the extension file
    ///
    /// # Returns
    /// * `Result<ExtensionWorker, Error>` - The worker thread
    pub fn new(extension_module: Module) -> Result<Self, ExternalError> {
        let (req_tx, req_rx) = channel::<ExtensionWorkerMessage>();
        let (res_tx, res_rx) = channel::<ExtensionWorkerResponse>();

        let join_handle = thread::spawn(move || {
            runtime_thread(extension_module, req_rx, res_tx);
        });

        let response = res_rx.recv().unwrap();
        let extension = match response {
            ExtensionWorkerResponse::Start(extension) => extension,
            ExtensionWorkerResponse::Error(err) => return Err(err),
            _ => {
                let e = Error::Internal(format!("JSRuntime worker responded incorrectly"));
                return Err(Box::new(e).into());
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
        token: &Token,
    ) -> Result<Value, Error> {
        self.request
            .send(ExtensionWorkerMessage::CallFunction {
                function: function.to_string(),
                args: args.to_vec(),
                state: cur_state.all_variables(),
                token: token.clone(),
            })
            .unwrap();

        match self.response.recv().unwrap() {
            ExtensionWorkerResponse::CallFunction { result, state } => {
                for (key, value) in state {
                    cur_state.set_variable(&key, value);
                }
                result
            }
            ExtensionWorkerResponse::Error(err) => Err(err.to_error(token)),
            _ => Err(Error::Internal(format!(
                "JSRuntime worker responded incorrectly"
            ))),
        }
    }

    /// Returns the extension
    pub fn extension(&self) -> &ExtensionDetails {
        &self.extension
    }

    pub fn to_std_function(&self, function: &str) -> Option<Function> {
        if let Some(function) = self.extension().all_functions().get(function) {
            Some(Function::new(
                &function.name(),
                function.description(),
                &self.extension().signature(),
                function
                    .arguments()
                    .iter()
                    .enumerate()
                    .map(|(i, arg)| Argument {
                        name: format!("{}", i + 1),
                        optional: false,
                        plural: false,
                        expects: *arg,
                    })
                    .collect(),
                *function.returns(),
                |state, args, token, name| {
                    // get a vec of the strings 1 to function.arguments().len()
                    let arg_order = (1..=args.len())
                        .map(|i| format!("{}", i))
                        .collect::<Vec<String>>();
                    ExtensionController::with(|controller| {
                        controller.call_function(
                            name,
                            &flatten_arguments!(args, arg_order),
                            state,
                            token,
                        )
                    })
                },
                function.name().to_string(),
            ))
        } else {
            None
        }
    }
}
