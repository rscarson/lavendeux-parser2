use once_cell::sync::OnceCell;
use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::rc::Rc;
use std::{collections::HashMap, fmt::Display};

use crate::Token;
use crate::{
    flatten_arguments, pest, required_argument, state::State, std_functions::Function, Error, Node,
    Value,
};
use polyvalue::ValueType;

// A cache of pre-compiled user function bodies
thread_local! {
    static USER_FUNCTION_CACHE: OnceCell<RefCell<HashMap<String, Rc<Node>>>> = OnceCell::new();
}

fn cached_fn_compile(src: &str) -> Result<Rc<Node>, Error> {
    USER_FUNCTION_CACHE.with(|once_lock| {
        let rt_mut = once_lock.get_or_init(|| RefCell::new(HashMap::new()));
        let mut cache = rt_mut.borrow_mut();

        match cache.entry(src.to_string()) {
            Entry::Occupied(o) => Ok(o.get().clone()),
            Entry::Vacant(v) => {
                let node = pest::parse_input(src, pest::Rule::TOPLEVEL_EXPRESSION)?;
                Ok(v.insert(Rc::new(node)).clone())
            }
        }
    })
}

#[derive(Debug, Clone)]
pub struct UserFunction {
    name: String,
    arguments: Vec<String>,
    src: String,
}

impl UserFunction {
    /// Creates a new user function
    pub fn new(name: &str, arguments: Vec<String>, src: String) -> Result<Self, Error> {
        // Check that the function is valid
        cached_fn_compile(&src)?;

        let inst = Self {
            name: name.to_string(),
            arguments,
            src,
        };

        Ok(inst)
    }

    /// Returns the name of this function
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the arguments of this function
    pub fn arguments(&self) -> &[String] {
        &self.arguments
    }

    /// Returns the source of this function
    pub fn src(&self) -> &str {
        &self.src
    }

    /// Returns the body of this function
    pub fn body(&self) -> Rc<Node> {
        cached_fn_compile(&self.src).unwrap()
    }

    /// Executes this function
    pub fn execute(
        &self,
        state: &mut State,
        arguments: Vec<Value>,
        token: &Token,
    ) -> Result<Value, Error> {
        // Create a new scope
        state.scope_into(token)?;

        // Set the arguments
        for (name, value) in self.arguments.iter().zip(arguments) {
            state.set_variable(name, value);
        }

        // Execute the body - this is checked in the constructor
        // so we can unwrap here
        let body_result = self.body().get_value(state);
        state.scope_out();

        body_result
    }

    /// Converts this function into a standard function
    pub fn to_std_function(&self) -> Function {
        Function::new(
            &self.name,
            &self.src(),
            "user-defined",
            self.arguments()
                .iter()
                .map(|s| required_argument!(s, ValueType::Any))
                .collect(),
            ValueType::Any,
            |state, args, token, func| {
                if let Some(function) = state.get_user_function(func) {
                    function.execute(state, flatten_arguments!(args, function.arguments), token)
                } else {
                    Err(Error::FunctionName {
                        name: func.clone(),
                        token: token.clone(),
                    })
                }
            },
            self.name.clone(),
        )
    }
}

impl Display for UserFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}({}) = {}",
            self.name(),
            self.arguments().join(", "),
            self.src()
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_precompilation() {
        let src = "fn test(a, b) = a + b";
        cached_fn_compile(src).unwrap();
        cached_fn_compile(src).unwrap();

        USER_FUNCTION_CACHE.with(|once_lock| {
            let rt_mut = once_lock.get_or_init(|| RefCell::new(HashMap::new()));
            let cache = rt_mut.borrow_mut();

            assert_eq!(1, cache.len());
        });
    }

    #[test]
    fn test_compilation() {
        let fn1 = UserFunction::new(
            "test",
            vec!["a".to_string(), "b".to_string()],
            "a + b".to_string(),
        )
        .unwrap();
        assert_eq!(fn1.name(), "test");
        assert_eq!(fn1.arguments(), &["a".to_string(), "b".to_string()]);
        assert_eq!(fn1.src(), "a + b");
        let res = fn1.execute(
            &mut State::new(),
            vec![Value::from(1.0), Value::from(2.0)],
            fn1.body().token(),
        );
        assert_eq!(res.unwrap(), Value::from(3.0));

        // no args now
        let fn2 = UserFunction::new("test2", vec![], "1 + 2".to_string()).unwrap();
        assert_eq!(fn2.name(), "test2");
        assert_eq!(fn2.arguments().len(), 0);
        assert_eq!(fn2.src(), "1 + 2");
        let res = fn2.execute(&mut State::new(), vec![], fn2.body().token());
        assert_eq!(res.unwrap(), Value::from(3));

        // 1 arg
        let fn3 = UserFunction::new("test3", vec!["a".to_string()], "a + 2".to_string()).unwrap();
        assert_eq!(fn3.name(), "test3");
        assert_eq!(fn3.arguments(), &["a".to_string()]);
        assert_eq!(fn3.src(), "a + 2");
        let res = fn3.execute(
            &mut State::new(),
            vec![Value::from(1.0)],
            fn3.body().token(),
        );
        assert_eq!(res.unwrap(), Value::from(3.0));
    }

    #[test]
    fn test_display() {
        let fn1 = UserFunction::new(
            "test",
            vec!["a".to_string(), "b".to_string()],
            "a + b".to_string(),
        )
        .unwrap();
        assert_eq!(format!("{}", fn1), "test(a, b) = a + b");

        let fn2 = UserFunction::new("test2", vec![], "1 + 2".to_string()).unwrap();
        assert_eq!(format!("{}", fn2), "test2() = 1 + 2");

        let fn3 = UserFunction::new("test3", vec!["a".to_string()], "a + 2".to_string()).unwrap();
        assert_eq!(format!("{}", fn3), "test3(a) = a + 2");
    }

    #[test]
    fn test_to_std() {
        let fn1 = UserFunction::new(
            "test",
            vec!["a".to_string(), "b".to_string()],
            "a + b".to_string(),
        )
        .unwrap();
        let std_fn = fn1.to_std_function();
        assert_eq!(std_fn.name(), "test");
        assert_eq!(std_fn.arguments().len(), 2);
        let mut state = &mut State::new();
        state.set_user_function(fn1.clone());
        let res = std_fn.execute(
            &mut state,
            vec![Value::from(1.0), Value::from(2.0)],
            fn1.body().token(),
        );
        assert_eq!(res.unwrap(), Value::from(3.0));
    }
}
