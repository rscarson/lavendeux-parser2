#[macro_export]
macro_rules! get_argument {
    ($name:expr, $arguments:expr) => {
        $crate::get_arguments!($name, $arguments)[0].clone()
    };
}

#[macro_export]
macro_rules! get_optional_argument {
    ($name:expr, $arguments:expr) => {
        $arguments.get($name).and_then(|v| Some(v[0].clone()))
    };
}

#[macro_export]
macro_rules! get_arguments {
    ($name:expr, $arguments:expr) => {
        $arguments
            .get($name)
            .ok_or(crate::error::ErrorDetails::Internal {
                msg: format!("No such argument `{}`", $name),
            })?
    };
}

/// Will ignore potential plural arguments, and return a flat vector of arguments
/// will panic if an argument is not found
#[macro_export]
macro_rules! flatten_arguments {
    ($arguments:expr, $keys:expr) => {
        $keys
            .iter()
            .map(|k| $arguments.get(k).cloned().unwrap()[0].clone())
            .collect::<Vec<Value>>()
    };
}

/// Macro to generate a static function definition
/// # Arguments
/// * `registry` - The registry to register the function to
/// * `name` - The name of the function
/// * `description` - The description of the function
/// * `category` - The category of the function
/// * `arguments` - The arguments of the function
/// * `returns` - The return type of the function
/// * `handler` - The handler of the function
#[macro_export]
macro_rules! static_function {
    (
        registry = $map:expr,
        name = $name:expr,
        description = $description:expr,
        category = $category:literal,
        arguments = [$($arguments:expr),*],
        returns = $returns:expr,
        handler = $handler:expr
    ) => {
        Function::new(
            $name,
            $description,
            $category,
            vec![$($arguments),*],
            $returns,
            $handler,
            String::new(),
        ).register($map)
    };

    (
        registry = $map:expr,
        name = $name:literal,
        description = $description:literal,
        arguments = [$($arguments:expr),*],
        returns = $returns:expr,
        handler = $handler:expr
    ) => {
        Function::new(
            $name,
            $description,
            "misc",
            vec![$($arguments),*],
            $returns,
            $handler,
            String::new(),
        ).register($map)
    };
}

#[macro_export]
macro_rules! required_argument {
    ($name:expr, $expects:expr) => {
        $crate::std_functions::Argument {
            name: $name.to_string(),
            expects: $expects,
            optional: false,
            plural: false,
        }
    };

    ($name:expr, $expects:expr, plural) => {
        $crate::std_functions::Argument {
            name: $name.to_string(),
            expects: $expects,
            optional: false,
            plural: true,
        }
    };
}

#[macro_export]
macro_rules! optional_argument {
    ($name:expr, $expects:expr) => {
        $crate::std_functions::Argument {
            name: $name.to_string(),
            expects: $expects,
            optional: true,
            plural: false,
        }
    };

    ($name:expr, $expects:expr, plural) => {
        $crate::std_functions::Argument {
            name: $name.to_string(),
            expects: $expects,
            optional: true,
            plural: true,
        }
    };
}
