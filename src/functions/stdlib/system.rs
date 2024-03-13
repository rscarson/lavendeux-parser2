use crate::{
    define_stdfunction,
    documentation::{DocumentationTemplate, MarkdownFormatter},
    error::{ErrorDetails, WrapOption},
    syntax_tree::traits::NodeExt,
    Lavendeux,
};
use polyvalue::{types::Object, Value};

/**********************************************
 *
 * Code and Evaluation
 *
 *********************************************/

define_stdfunction!(
   call_function {
       name: Standard::String,
       args: Standard::Array
   },
   returns = Any,

   docs = {
       category: "System",
       description: "Calls a function or @decorator by name with the given arguments",
       ext_description: "
            If the name begins with '@', it will be treated as a decorator.
            Maps the given object to the function's arguments and calls the function.
            Important note: Functions that take in a _reference, such as pop/push etc, will act by-value and not modify the original object.
        ",
       examples: "
            @test(x) = x
            assert_eq('5', call_function('@test', {'x': 5}))
        ",
   },

    handler = (state, _reference) {
         let name = required_arg!(state::name).to_string();
         let args = required_arg!(state::args).as_a::<Vec<Value>>()?;

         state.call_function(&name, args, None)
    },
);

define_stdfunction!(
    eval {
        expression: Standard::String
    },
    returns = Any,

    docs = {
        category: "System",
        description: "Evaluates a string as a Lavendeux expression and returns the result",
        ext_description: "
            The string will be interpreted as a script and evaluated in it's own scope.
            If there are multiple lines, an array of values will be returned.
        ",
        examples: "
            assert_eq(5, eval('2 + 3'))
            assert_eq([6, 6], eval('x = 6; x'))
            assert_eq([1, 2, 3], eval('1\\n2\\n3'))
        ",
    },
    handler = (state, _reference) {
        let expression = required_arg!(state::expression).to_string();

        state.scope_into()?;
        state.lock_scope();
        let res = Lavendeux::eval(&expression, state).map(|n| n.evaluate(state));

        let mut values = match res {
            Ok(r) => {
                match r {
                    Ok(v) => v,
                    Err(e) => {
                        state.scope_out();
                        return Err(e);
                    }
                }
            },
            Err(e) => {
                state.scope_out();
                return Err(e);
            }
        };

        state.scope_out();
        if values.len() == 1 {
            values = values.as_a::<Vec<Value>>()?.into_iter().next().unwrap();
        }
        Ok(values)
    },
);

define_stdfunction!(
    include {
        filename: Standard::String
    },
    returns = Any,

    docs = {
        category: "System",
        description: "Evaluates a file as a Lavendeux expression and returns the result",
        ext_description: "
            The file will be interpreted as a script and evaluated in it's own scope.
            Returns an empty string in all cases.
        ",
        examples: "
            include('example_scripts/stdlib.lav')
        ",
    },
    handler = (state, _reference) {
        let script = required_arg!(state::filename).to_string();
        let script = std::fs::read_to_string(script)?;

        state.scope_into()?;
        state.lock_scope();
        let res = Lavendeux::eval(&script, state).map(|n| n.evaluate(state));
        match res {
            Ok(r) => {
                match r {
                    Ok(v) => v,
                    Err(e) => {
                        state.scope_out();
                        return Err(e);
                    }
                }
            },
            Err(e) => {
                state.scope_out();
                return Err(e);
            }
        };

        state.scope_out();
        Ok(Value::from(""))
    },
);

define_stdfunction!(
    __exec_tests {
    },
    returns = Any,

    docs = {
        category: "System",
        description: "Evaluates all functions beginning with __test_, and reports a list of failed tests",
        ext_description: "
            Designed to be used mostly for internal testing, could be useful to testing scripts.
            Throws an error if a test fails, otherwise returns a string with the number of tests run and the number of tests failed.
        ",
        examples: "#skip
            __test_will_fail() = assert_eq(1, 2)
            __test_will_pass() = assert_eq(1, 1)
            __exec_tests()
            /* Output:
            Errors:

            In __test_will_fail: 
            Line 1: assert_eq (1, 2)
                Assertion failed: 1 != 2
                
            2 tests run, 1 failed
             */
        ",
    },
    handler = (state, _reference) {
        let matching_functions = state
            .all_functions()
            .iter()
            .filter(|(name, _)| name.starts_with("__test_"))
            .map(|(name, _)| name.clone())
            .collect::<Vec<_>>();

        let mut errors = vec![];
        for test_case in matching_functions.iter() {
            state.scope_into()?;
            state.lock_scope();
            let res = state.call_function(test_case, vec![], None);
            state.scope_out();

            if let Err(e) = res {
                errors.push((test_case, e));
            }
        }

        let mut output = String::new();
        if !errors.is_empty() {
            output.push_str("Errors:\n\n");
            for (name, e) in errors.iter() {
                output.push_str(&format!("In {}:\n{}\n\n", name, e));
            }
        }

        output.push_str(&format!(
            "{} tests run, {} failed",
            matching_functions.len(),
            errors.len()
        ));

        if errors.is_empty() {
            Ok(Value::from(format!("{} tests run, all passed", matching_functions.len())))
        } else {
            oops!(Custom { msg: output })
        }
    },
);

define_stdfunction!(
    generate_documentation {},
    returns = String,
    docs = {
        category: "System",
        description: "Generates documentation for all standard library functions",
        ext_description: "
            Returns a markdown-formatted string containing documentation for all standard library functions.
        ",
        examples: "
            generate_documentation()
        ",
    },
    handler = (state, _reference) {
        Ok(DocumentationTemplate::new(MarkdownFormatter).render(state).into())
    },
);

define_stdfunction!(
    document_function {
        name: Standard::String,
        docs: Standard::Object
    },
    returns = String,
    docs = {
        category: "System",
        description: "Adds documentation to a user-defined function",
        ext_description: "
            Adds documentation to a function, which will be displayed help()
            The documentation object should contain the keys 'category', 'description', 'ext_description', and 'examples'.
        ",
        examples: "
            a() = 5
            document_function('a', {
                'category': 'System',
                'description': 'Adds documentation to a function',
                'ext_description': 'Adds documentation to a function, which will be displayed in the documentation.',
                'examples': 'document_function(\"document_function\", {\"category\": \"System\", \"description\": \"Adds documentation to a function\", \"ext_description\": \"Adds documentation to a function, which will be displayed in the documentation.\"})'
            })
        ",
    },
    handler = (state, _reference) {
        let name = required_arg!(state::name).to_string();
        let docs = required_arg!(state::docs).as_a::<Object>()?;

        let function = state.get_function_mut(&name).or_error(ErrorDetails::FunctionName { name: name.clone() })?;
        if function.is_readonly() {
            return oops!(Custom {
                msg: "Cannot modify a readonly function".to_string()
            })
        }

        if let Some(category) = docs.get(&"category".into()) {
            function.documentation_mut().set_category(&category.to_string());
        }

        let ext_desc: Option<String> = docs.get(&"description".into()).map(|v| v.to_string());
        function.documentation_mut().set_description(ext_desc.as_deref());

        let ext_desc: Option<String> = docs.get(&"ext_description".into()).map(|v| v.to_string());
        function.documentation_mut().set_ext_description(ext_desc.as_deref());

        let ext_desc: Option<String> = docs.get(&"examples".into()).map(|v| v.to_string());
        function.documentation_mut().set_examples(ext_desc.as_deref());

        Ok(state.help(Some(name)).into())
    },
);

/**********************************************
 *
 * Assertions and Errors
 *
 *********************************************/

define_stdfunction!(
    assert {
        condition: Standard::Any
    },
    returns = Any,

    docs = {
        category: "System",
        description: "Throws an error if the condition is false",
        ext_description: "
            Does a weak-comparison to boolean, so 0, '', [], etc. are all considered false.
            Returns the value otherwise
        ",
        examples: "
            assert(true)
            assert( would_err('assert(false)') )
        ",
    },
    handler = (state, _reference) {
        let cond = required_arg!(state::condition);
        if cond.is_truthy() {
            Ok(cond.clone())
        } else {
            oops!(Custom {
                msg: "Assertion failed".to_string()
            })
        }
    },
);

define_stdfunction!(
    assert_eq {
        condition: Standard::Any,
        expected: Standard::Any
    },
    returns = Any,

    docs = {
        category: "System",
        description: "Asserts that 2 values are equal",
        ext_description: "
            Raises an error if the condition is not equal to the expected value.
            Also verifies type, as opposed to the `==` operator, which uses weak typing.
            use assert(a == b) if you want to compare values without checking their types.
        ",
        examples: "
            assert_eq(true, true)
            assert_eq( true, would_err('assert_eq(1, true)') )
        ",
    },
    handler = (state, _reference) {
        let cond = required_arg!(state::condition);
        let expected = required_arg!(state::expected);

        if cond == expected {
            Ok(cond.clone())
        } else {
            let message = format!("Assertion failed: {:?} != {:?}", cond, expected);
            oops!(Custom { msg: message })
        }
    },
);

define_stdfunction!(
    would_err {
        expression: Standard::String
    },
    returns = Bool,

    docs = {
        category: "System",
        description: "Returns true if the given expression would raise an error",
        ext_description: "
            Returns true if expression given by the string would raise an error, false otherwise.
            This is useful for testing error messages.
        ",
        examples: "
            assert_eq( false, would_err('1 + 1') )
            assert_eq( true, would_err('1 + asparagus') )
        ",
    },
    handler = (state, _reference) {
        let expression = required_arg!(state::expression).to_string();
        let res = crate::Lavendeux::eval(&expression, state).map(|n| n.evaluate(state));
        match res {
            Ok(r) if r.is_ok() => Ok(Value::from(false)),
            _ => Ok(Value::from(true))
        }
    },
);

define_stdfunction!(
    error {
        msg: Standard::String
    },
    returns = Any,

    docs = {
        category: "System",
        description: "Throws an error with the given message",
        ext_description: "
            Throws an exception with a custom message. The error's source will be the line where the error was thrown.
        ",
        examples: "
            would_err('error(\"This is an error\")')
        ",
    },
    handler = (state, _reference) {
        let message = required_arg!(state::msg).to_string();
        oops!(Custom { msg: message })
    },
);

define_stdfunction!(
    debug {
        msg: Standard::String
    },
    returns = Any,

    docs = {
        category: "System",
        description: "Prints a debug message to the console",
        ext_description: "
            The message will be both written to stdout, and returned as a string.
            If the parser is not attached to a console, it will not be visible.
        ",
        examples: "
            debug(\"This is a debug message\")
        ",
    },
    handler = (state, _reference) {
        let message = required_arg!(state::msg).to_string();
        println!("{message}");
        Ok(Value::string(message))
    },
);

/**********************************************
 *
 * Assignments and Variables
 *
 *********************************************/

define_stdfunction!(
    assign {
        name: Standard::String,
        value: Standard::Any
    },
    returns = Any,

    docs = {
        category: "System",
        description: "Assigns a variable in the current scope",
        ext_description: "
            Writes a value to the current scope, leaving other scopes unchanged.
        ",
        examples: "
            x = 5
            if true then {
                assign('x', 6)
                assert_eq(6, x)
            } else nil
            assert_eq(5, x)
        ",
    },
    handler = (state, _reference) {
        let name = required_arg!(state::name).to_string();
        let value = required_arg!(state::value);
        state.set_variable_in_offset(1, &name, value.clone());
        Ok(value)
    },
);

define_stdfunction!(
    assign_global {
        name: Standard::String,
        value: Standard::Any
    },
    returns = Any,

    docs = {
        category: "System",
        description: "Assigns a variable in the top-level scope",
        ext_description: "
            Writes a value to the top-level scope, leaving other scopes unchanged.
        ",
        examples: "
            x = 5
            if true then {
                assign_global('x', 6)
                assert_eq(6, x)
            } else { 0 }
            assert_eq(6, x)
        ",
    },
    handler = (state, _reference) {
        let name = required_arg!(state::name).to_string();
        let value = required_arg!(state::value);
        state.global_assign_variable(&name, value.clone());
        Ok(value.clone())
    },
);

define_stdfunction!(
    delete_global {
        name: Standard::String
    },
    returns = Any,

    docs = {
        category: "System",
        description: "Removes a variable from the top-level scope",
        ext_description: "
            Removes a value from the top-level scope, leaving other scopes unchanged.
        ",
        examples: "
            assign_global('x', 6)
            delete_global('x')
        ",
    },
    handler = (state, _reference) {
        let name = required_arg!(state::name).to_string();
        state.global_delete_variable(&name).or_error(ErrorDetails::VariableName {
            name
        })
    },
);

define_stdfunction!(
    global {
        name: Standard::String
    },
    returns = Any,

    docs = {
        category: "System",
        description: "Returns a variable from the top-level scope",
        ext_description: "
            Searches for the variable in the top-level scope only
        ",
        examples: "
            assign_global('x', 6)
            assert_eq(6, global('x'))
        ",
    },
    handler = (state, _reference) {
        let name = required_arg!(state::name).to_string();
        let value = state.global_get_variable(&name).or_error(ErrorDetails::VariableName {
            name
        })?;
        Ok(value.clone())
    },
);

define_stdfunction!(
    variables { },
    returns = Object,

    docs = {
        category: "System",
        description: "Returns the currently defined variables",
        ext_description: "
            Returns a map of all the variables currently defined in the current scope.
        ",
        examples: "
            x = 5; y = 6
            state = variables()
            assert_eq(5, state['x'])
            assert_eq(6, state['y'])
        ",
    },
    handler = (state, _reference) {
        let obj = Object::try_from(
            state.all_variables_unscoped()
                .iter()
                .map(|(k, v)| (Value::from(k.to_string()), (*v).clone()))
                .collect::<Vec<(Value, Value)>>(),
        )?;

        Ok(obj.into())
    },
);

define_stdfunction!(
    typeof {
        value: Standard::Any
    },
    returns = String,

    docs = {
        category: "System",
        description: "Returns the type of its input",
        ext_description: "
            Returns the type of the given value as a string.
        ",
        examples: "
            assert_eq('string', typeof('hello'))
            assert_eq('i64', typeof(5))
            assert_eq('object', typeof({}))
        ",
    },
    handler = (state, _reference) {
        let value = required_arg!(state::value);
        Ok(Value::string(value.own_type().to_string()))
    },
);

#[cfg(test)]
mod test {
    use crate::lav;

    lav!(test_exec_tests_bad(Error) r#"
        __test_will_fail() = assert_eq(1, 2)
        __test_will_pass() = assert_eq(1, 1)
        __exec_tests()
    "#);

    lav!(test_exec_tests_good r#"
        __test_will_pass() = assert_eq(1, 1)
        __exec_tests()
    "#);
}
