use crate::{
    get_argument, required_argument, static_function, std_functions::Function, Error, Lavendeux,
    State,
};
use polyvalue::{types::Object, Value, ValueType};
use std::collections::HashMap;

pub fn register_all(map: &mut HashMap<String, Function>) {
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
        handler = |_: &mut State, arguments, _token, _| {
            let expression = get_argument!("expression", arguments).to_string();
            crate::extensions::ExtensionController::exec(&expression)
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
        handler = |_: &mut State, arguments, _token, _| {
            let filename = get_argument!("filename", arguments).to_string();
            crate::extensions::ExtensionController::with(|controller| {
                let extension = controller.register(&filename)?;
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
        handler = |state: &mut State, arguments, _token, _| {
            let expression = get_argument!("expression", arguments).to_string();
            Lavendeux::eval(&expression, state)
        }
    );

    static_function!(
        registry = map,
        name = "eval_file",
        description = "Evaluates a file as a Lavendeux expression and returns the result",
        category = "system",
        arguments = [required_argument!("filename", ValueType::String)],
        returns = ValueType::Any,
        handler = |state: &mut State, arguments, _token, _| {
            let filename = get_argument!("filename", arguments).to_string();
            match std::fs::read_to_string(filename) {
                Ok(expression) => Lavendeux::eval(&expression, state),
                Err(e) => Err(Error::Io(e)),
            }
        }
    );

    static_function!(
        registry = map,
        name = "state",
        description = "Returns the currently defined variables",
        category = "system",
        arguments = [],
        returns = ValueType::Object,
        handler = |state: &mut State, _arguments, _token, _| {
            Ok(Object::try_from(
                state
                    .all_variables()
                    .iter()
                    .map(|(k, v)| (Value::from(k.to_string()), v.clone()))
                    .collect::<Vec<(Value, Value)>>(),
            )?
            .into())
        }
    )
}
