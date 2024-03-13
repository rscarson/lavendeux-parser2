use crate::documentation::{DocumentationTemplate, MarkdownFormatter};
use crate::functions::ParserFunction;
use crate::pest::LavendeuxParser;
use crate::syntax_tree::traits::NodeExt;
use crate::syntax_tree::Node;
use crate::{Error, Rule, State, Value};
use std::num::NonZeroUsize;
use std::time::Duration;

/// Available options for the parser
/// timeout - The timeout for the parser
/// stack_size - The stack size in bytes for the parsing thread
#[derive(Debug, Clone)]
pub struct ParserOptions {
    /// Timeout value to use when building the [State]
    pub timeout: Duration,

    /// The maximum number of calls to the pest parser
    /// This is used to prevent stack overflows
    pub pest_call_limit: usize,
}
impl Default for ParserOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(0),
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
    pub(crate) fn eval<'i>(input: &'i str, state: &mut State) -> Result<Node<'i>, Error> {
        Self::eval_rule(input, state, Rule::SCRIPT)
    }

    /// Evaluate input against a given state, bypassing the normal checks for
    /// threading, timeout, and without sanitizing scope depth
    pub(crate) fn eval_rule<'i>(
        input: &'i str,
        state: &mut State,
        rule: Rule,
    ) -> Result<Node<'i>, Error> {
        let root = LavendeuxParser::parse2(input, rule)?;
        LavendeuxParser::compile_ast(root, state)
    }

    /// Parses the given input
    /// Returns an array of values, one for each line in the input
    pub fn parse(&mut self, input: &str) -> Result<Vec<Value>, Error> {
        self.state.sanitize_scopes();
        pest::set_call_limit(NonZeroUsize::new(self.options.pest_call_limit));
        self.state.start_timer();

        let value = Self::eval(input, &mut self.state)?.evaluate(&mut self.state)?;
        let lines = value.as_a::<Vec<Value>>()?;
        Ok(lines)
    }

    /// Run the parser on the given file
    /// Returns an array of values, one for each line in the input
    pub fn run(&mut self, filename: &str) -> Result<Vec<Value>, Error> {
        let input = std::fs::read_to_string(filename)?;
        self.parse(&input)
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
            "1$1666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666662.11",
        ).unwrap_err();
        parser.parse("e₿8**82asin").unwrap_err();
        parser.parse("e85**88d**e8**8").unwrap_err();
    }
}
