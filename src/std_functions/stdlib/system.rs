use crate::{
    error::WrapError, get_argument, required_argument, static_function, std_functions::Function,
    Error, Lavendeux, State,
};
use polyvalue::{types::Object, Value, ValueType};
use std::collections::HashMap;

pub fn register_all(map: &mut HashMap<String, Function>) {
    static_function!(
        registry = map,
        name = "error",
        description = "Throws an error with the given message",
        category = "system",
        arguments = [required_argument!("message", ValueType::String)],
        returns = ValueType::Any,
        handler = |_state: &mut State, arguments, token, _| {
            let message = get_argument!("message", arguments).to_string();
            Err(Error::Custom {
                message: message,
                token: token.clone(),
            })
        }
    );

    static_function!(
        registry = map,
        name = "throw",
        description = "Throws an error with the given message",
        category = "system",
        arguments = [required_argument!("message", ValueType::String)],
        returns = ValueType::Any,
        handler = |_state: &mut State, arguments, token, _| {
            let message = get_argument!("message", arguments).to_string();
            Err(Error::Custom {
                message: message,
                token: token.clone(),
            })
        }
    );

    static_function!(
        registry = map,
        name = "debug",
        description = "Prints a debug message to the console",
        category = "system",
        arguments = [required_argument!("message", ValueType::String)],
        returns = ValueType::Any,
        handler = |_state: &mut State, arguments, _token, _| {
            let message = get_argument!("message", arguments).to_string();
            println!("{message}");
            Ok(Value::from(message))
        }
    );

    static_function!(
        registry = map,
        name = "assert",
        description = "Throws an error if the condition is false",
        category = "system",
        arguments = [required_argument!("condition", ValueType::Any)],
        returns = ValueType::Any,
        handler = |_state: &mut State, arguments, token, _| {
            let cond = get_argument!("condition", arguments);
            if cond.is_truthy() {
                return Ok(cond);
            } else {
                let message = "Assertion failed".to_string();
                return Err(Error::Custom {
                    message: message,
                    token: token.clone(),
                });
            }
        }
    );

    static_function!(
        registry = map,
        name = "assert_eq",
        description = "Throws an error if the two values are not equal",
        category = "system",
        arguments = [
            required_argument!("condition", ValueType::Any),
            required_argument!("expected", ValueType::Any)
        ],
        returns = ValueType::Any,
        handler = |_state: &mut State, arguments, token, _| {
            let cond = get_argument!("condition", arguments);
            let expected = get_argument!("expected", arguments);
            if cond == expected {
                return Ok(Value::from(vec![cond, expected]));
            } else {
                let message = format!("Assertion failed: {:?} != {:?}", cond, expected);
                return Err(Error::Custom {
                    message: message,
                    token: token.clone(),
                });
            }
        }
    );

    static_function!(
        registry = map,
        name = "assign",
        description = "Assigns a variable in the current scope",
        category = "system",
        arguments = [
            required_argument!("name", ValueType::String),
            required_argument!("value", ValueType::Any)
        ],
        returns = ValueType::Any,
        handler = |state: &mut State, arguments, _token, _| {
            let input = get_argument!("name", arguments).to_string();
            let value = get_argument!("value", arguments);

            state.set_variable(&input, value.clone());
            Ok(value)
        }
    );

    static_function!(
        registry = map,
        name = "assign_global",
        description = "Assigns a variable in the top-level scope",
        category = "system",
        arguments = [
            required_argument!("name", ValueType::String),
            required_argument!("value", ValueType::Any)
        ],
        returns = ValueType::Any,
        handler = |state: &mut State, arguments, _token, _| {
            let input = get_argument!("name", arguments).to_string();
            let value = get_argument!("value", arguments);

            state.global_assign_variable(&input, value.clone());
            Ok(value)
        }
    );

    #[cfg(feature = "extensions")]
    static_function!(
        registry = map,
        name = "js",
        description = "Executes a JavaScript expression and returns the result",
        category = "system",
        arguments = [required_argument!("expression", ValueType::String)],
        returns = ValueType::Any,
        handler = |_: &mut State, arguments, token, _| {
            let expression = get_argument!("expression", arguments).to_string();
            crate::extensions::ExtensionController::exec(&expression).to_error(token)
        }
    );

    #[cfg(feature = "extensions")]
    static_function!(
        registry = map,
        name = "add_extension",
        description = "Adds a JavaScript extension to the interpreter",
        category = "system",
        arguments = [required_argument!("filename", ValueType::String)],
        returns = ValueType::String,
        handler = |_: &mut State, arguments, token, _| {
            let filename = get_argument!("filename", arguments).to_string();
            crate::extensions::ExtensionController::with(|controller| {
                let extension = controller.register(&filename).to_error(token)?;
                Ok(Value::from(extension.signature()))
            })
        }
    );

    #[cfg(feature = "extensions")]
    static_function!(
        registry = map,
        name = "remove_extension",
        description = "Removes a JavaScript extension from the interpreter",
        category = "system",
        arguments = [required_argument!("name", ValueType::String)],
        returns = ValueType::String,
        handler = |_: &mut State, arguments, _token, _| {
            let name = get_argument!("name", arguments).to_string();
            crate::extensions::ExtensionController::with(|controller| {
                controller.unregister(&name);
                Ok(Value::from(name.clone()))
            })
        }
    );

    static_function!(
        registry = map,
        name = "eval",
        description = "Evaluates a string as a Lavendeux expression and returns the result",
        category = "system",
        arguments = [required_argument!("expression", ValueType::String)],
        returns = ValueType::Any,
        handler = |state: &mut State, arguments, token, _| {
            let expression = get_argument!("expression", arguments).to_string();

            state.scope_into(&token)?;
            let res = Lavendeux::eval(&expression, state);
            state.scope_out();
            res
        }
    );

    static_function!(
        registry = map,
        name = "include",
        description = "Evaluates a file as a Lavendeux expression and returns the result",
        category = "system",
        arguments = [required_argument!("filename", ValueType::String)],
        returns = ValueType::Any,
        handler = |state: &mut State, arguments, token, _| {
            let filename = get_argument!("filename", arguments).to_string();
            let script = std::fs::read_to_string(filename).to_error(token)?;

            state.scope_into(&token)?;
            state.lock_scope();
            let res = Lavendeux::eval(&script, state);
            state.scope_out();
            res
        }
    );

    static_function!(
        registry = map,
        name = "state",
        description = "Returns the currently defined variables",
        category = "system",
        arguments = [],
        returns = ValueType::Object,
        handler = |state: &mut State, _arguments, token, _| {
            Ok(Object::try_from(
                state
                    .all_variables()
                    .iter()
                    .map(|(k, v)| (Value::from(k.to_string()), v.clone()))
                    .collect::<Vec<(Value, Value)>>(),
            )
            .to_error(token)?
            .into())
        }
    );

    static_function!(
        registry = map,
        name = "typeof",
        description = "Returns the type of its input",
        category = "system",
        arguments = [required_argument!("input", ValueType::Any)],
        returns = ValueType::String,
        handler = |_state: &mut State, arguments, _token, _| {
            let input = get_argument!("input", arguments);
            Ok(Value::from(input.own_type().to_string()))
        }
    );
}
