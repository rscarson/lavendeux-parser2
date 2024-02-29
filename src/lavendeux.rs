use crate::documentation::{DocumentationTemplate, MarkdownFormatter};
use crate::functions::ParserFunction;
use crate::pest::{parse_input, Rule};
use crate::syntax_tree::Node;
use crate::{Error, State, Value};
use polyvalue::types::Array;
use polyvalue::ValueTrait;
use std::num::NonZeroUsize;
use std::time::Duration;

/// Available options for the parser
/// timeout - The timeout for the parser
/// stack_size - The stack size in bytes for the parsing thread
#[derive(Debug, Clone)]
pub struct ParserOptions {
    /// Timeout value to use when building the [State]
    pub timeout: Duration,

    /// Stack size for the thread running the parser
    pub stack_size: usize,

    /// The maximum number of calls to the pest parser
    /// This is used to prevent stack overflows
    pub pest_call_limit: usize,
}
impl Default for ParserOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(0),
            stack_size: 1024 * 1024 * 8,
            pest_call_limit: 0,
        }
    }
}

/// The main parser, and the entrypoint for the library
#[derive(Debug)]
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

    /// Register a function with the parser
    pub fn register_function(&mut self, function: impl ParserFunction) -> Result<(), Error> {
        self.state.register_function(function)
    }

    /// Get a reference to the state
    pub fn state(&self) -> &State {
        &self.state
    }

    /// Get a mutable reference to the state
    pub fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }

    /// Evaluate input against a given state, bypassing the normal checks for
    /// threading, timeout, and without sanitizing scope depth
    pub fn eval<'i>(input: &'i str, state: &mut State) -> Result<Node<'i>, Error<'i>> {
        parse_input(input, Rule::SCRIPT)
    }

    /// Parses the given input
    /// Returns an array of values, one for each line in the input
    pub fn parse<'i>(&mut self, input: &'i str) -> Result<Vec<Value>, Error<'i>> {
        self.state.sanitize_scopes();
        pest::set_call_limit(NonZeroUsize::new(self.options.pest_call_limit));

        let value = std::thread::scope(|s| -> Result<Value, Error> {
            let handle = std::thread::Builder::new()
                .stack_size(self.options.stack_size)
                .name("lavendeux-parser".to_string())
                .spawn_scoped(s, || {
                    self.state.start_timer();
                    Self::eval(input, &mut self.state)?.get_value(&mut self.state)
                })
                .or(oops!(Fatal {
                    msg: "Failed to spawn parser thread".to_string()
                }))?;
            match handle.join() {
                Ok(value) => value,
                Err(e) => {
                    if let Some(s) = e.downcast_ref::<&str>() {
                        let s = s.to_string();
                        oops!(Fatal { msg: s })
                    } else {
                        oops!(Fatal {
                            msg: format!("Parser thread panicked: {:?}", e)
                        })
                    }
                }
            }
        })?;

        let lines = value.as_a::<Array>().unwrap().inner().clone();
        Ok(lines)
    }

    /// Generates markdown formatted documentation for the parser
    /// Returns it as a string
    pub fn generate_documentation(&self) -> String {
        DocumentationTemplate::new(MarkdownFormatter).render(&self.state)
    }
}

// Tests mostly related to the fuzzer
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_slow_brackets() {
        let mut parser = Lavendeux::new(ParserOptions {
            timeout: Duration::from_millis(500),
            pest_call_limit: 25000000,
            ..Default::default()
        });
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
        parser
                  .parse("forirPP[forPorP[f&r[forPorP[ffororPP[forororPP[forPorP[f&r[forPorP[ffororPP[forororPP[forPorP[f&r[forPorP[f&Br[PP]/b@]][f&r[P;;P]]]^d]f&[]P[f&r[forPorP[f&r[PP]-b@]]]^d]PP]][PorP[f&Br[PP]/b@]][f&r[P;;P]]]^d]f&[]P[f&r[forPorP[f&r[PP]-b@]]]^d]PP]][f&r[P;;P]]]^d]f&[]")
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
