use crate::pest::{parse_input, Rule};
use crate::std_functions::Function;
use crate::{Error, State, Value};
use polyvalue::types::Array;
use polyvalue::ValueTrait;

/// Available options for the parser
/// timeout - The timeout in seconds for the parser
/// stack_size - The stack size in bytes for the parsing thread
#[derive(Debug, Clone)]
pub struct ParserOptions {
    pub timeout: u64,
    pub stack_size: usize,
}
impl Default for ParserOptions {
    fn default() -> Self {
        Self {
            timeout: 0,
            stack_size: 1024 * 1024 * 8,
        }
    }
}

pub struct Lavendeux {
    state: State,
    options: ParserOptions,
}
impl Lavendeux {
    /// Create a new Lavendeux instance
    /// The instance will have a new state
    pub fn new(options: ParserOptions) -> Self {
        Self::with_state(options.clone(), State::with_timeout(options.timeout))
    }

    /// Create a new Lavendeux instance with a given state
    pub fn with_state(options: ParserOptions, state: State) -> Self {
        Self { state, options }
    }

    pub fn register_function(&mut self, function: Function) {
        self.state.register_function(function);
    }

    /// Evaluate input against a given state, bypassing the normal checks for
    /// threading, timeout, and without sanitizing scope depth
    pub fn eval(input: &str, state: &mut State) -> Result<Value, Error> {
        let script = parse_input(input, Rule::SCRIPT)?;
        script.get_value(state)
    }

    /// Parses the given input
    pub fn parse(&mut self, input: &str) -> Result<Vec<Value>, Error> {
        self.state.sanitize_scopes();
        let value = std::thread::scope(|s| -> Result<Value, Error> {
            let handle = std::thread::Builder::new()
                .stack_size(self.options.stack_size)
                .name(format!("lavendeux-parser"))
                .spawn_scoped(s, || {
                    self.state.start_timer();
                    Self::eval(input, &mut self.state)
                })?;
            match handle.join() {
                Ok(value) => value,
                Err(e) => Err(Error::Fatal(e.downcast_ref::<&str>().unwrap().to_string())),
            }
        })?;

        let lines = value.as_a::<Array>()?.inner().clone();
        Ok(lines)
    }

    /// Load extension from a loaded module
    /// # Arguments
    /// * `module` - The extension source
    ///
    /// # Returns
    /// An error if the extension could not be loaded
    #[cfg(feature = "extensions")]
    pub fn load_extension_module(
        &mut self,
        module: rustyscript::Module,
    ) -> Result<crate::extensions::ExtensionDetails, Error> {
        crate::extensions::ExtensionController::with(|controller| controller.add_extension(module))
    }

    /// Load an extension from a file and register it
    /// # Arguments
    /// * `filename` - The filename of the extension to load
    ///
    /// # Returns
    /// An error if the extension could not be loaded
    #[cfg(feature = "extensions")]
    pub fn load_extension(
        &mut self,
        filename: &str,
    ) -> Result<crate::extensions::ExtensionDetails, Error> {
        crate::extensions::ExtensionController::with(|controller| controller.register(filename))
    }

    /// Unload an extension, stopping the thread and unregistering all functions
    /// # Arguments
    /// * `name` - The filename of the extension to unload
    #[cfg(feature = "extensions")]
    pub fn unload_extension(&mut self, filename: &str) {
        crate::extensions::ExtensionController::with(|controller| controller.unregister(filename));
    }
}
