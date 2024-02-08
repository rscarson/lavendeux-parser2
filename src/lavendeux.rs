use crate::pest::{parse_input, Rule};
use crate::preprocessor::{Preprocessor, PreprocessorDirectives};
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

#[derive(Debug, Clone)]
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

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }

    /// Designed to catch unmatched brackets that can slow down parsing
    pub fn preprocess_input(input: &str) -> Result<PreprocessorDirectives, Error> {
        Preprocessor::process(input)
    }

    /// Evaluate input against a given state, bypassing the normal checks for
    /// threading, timeout, and without sanitizing scope depth
    pub fn eval(input: &str, state: &mut State) -> Result<Value, Error> {
        let directives = Self::preprocess_input(input)?;
        for (name, value) in directives.consts {
            state.set_variable(&name, value.into());
        }

        let script = parse_input(&directives.script, Rule::SCRIPT)?;
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
                })
                .or(Err(Error::Fatal(
                    "Failed to spawn parser thread".to_string(),
                )))?;
            match handle.join() {
                Ok(value) => value,
                Err(e) => Err(Error::Fatal(e.downcast_ref::<&str>().unwrap().to_string())),
            }
        })?;

        let lines = value.as_a::<Array>().unwrap().inner().clone();
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
        module: rustyscript::Module,
    ) -> Result<crate::extensions::ExtensionDetails, Error> {
        Ok(crate::extensions::ExtensionController::with(
            |controller| controller.add_extension(module),
        )?)
    }

    /// Load an extension from a file and register it
    /// # Arguments
    /// * `filename` - The filename of the extension to load
    ///
    /// # Returns
    /// An error if the extension could not be loaded
    #[cfg(feature = "extensions")]
    pub fn load_extension(filename: &str) -> Result<crate::extensions::ExtensionDetails, Error> {
        Ok(crate::extensions::ExtensionController::with(
            |controller| controller.register(filename),
        )?)
    }

    /// Unload an extension, stopping the thread and unregistering all functions
    /// # Arguments
    /// * `name` - The filename of the extension to unload
    #[cfg(feature = "extensions")]
    pub fn unload_extension(filename: &str) {
        crate::extensions::ExtensionController::with(|controller| controller.unregister(filename));
    }

    /// Unload all extensions, stopping all threads and unregistering all functions
    #[cfg(feature = "extensions")]
    pub fn unload_all_extensions() {
        crate::extensions::ExtensionController::with(|controller| controller.unregister_all());
    }
}

// Tests mostly related to the fuzzer
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_slow_brackets() {
        let mut parser = Lavendeux::new(Default::default());
        parser
            .parse("X[[[]3[4[bri[z[eeg(e4?estarts_witheHoAs(tX[[[]3[4[bri[z[eee(e4?estarts_<a")
            .unwrap_err();
        parser
            .parse("X[[[]3[4[bri[z[eee(e4?estarts_witheHAso(tX[[[]3[4[bri[z[eee(e4?estarts_<a")
            .unwrap_err();
        parser
            .parse("eeeeeeeA(e5[[4^A(e5[[4^A^eA(e5[[4^A(e5[[4^A^A^")
            .unwrap_err();
        parser
            .parse("eeeeeeA(peeeeeA(eeeeA(peeeeeA(eeA(pA(peeA(pA(pi^A")
            .unwrap_err();
    }

    #[test]
    fn test_large_fixed_convert() {
        let mut parser = Lavendeux::new(Default::default());
        parser.parse(
            "1$16666666666666666666666666666666666666666666666666666666666666666666666666662.11",
        ).unwrap_err();
        parser.parse("e₿8**82asin").unwrap_err();
        parser.parse("e85**88d**e8**8").unwrap_err();
    }
}
