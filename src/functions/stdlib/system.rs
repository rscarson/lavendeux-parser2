use crate::{
    define_stdfunction,
    documentation::{DocumentationTemplate, MarkdownFormatter},
    functions::std_function::ParserFunction,
    Lavendeux, State,
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
            Important note: Functions that take in a reference, such as pop/push etc, will act by-value and not modify the original object.
        ",
       examples: "
            @test(x) = x
            assert_eq('5', call_function('@test', {'x': 5}))
        ",
   },

    handler = |state: &mut State| {
         let name = state.get_variable("name").unwrap().to_string();
         let args = state.get_variable("args").unwrap().as_a::<Vec<Value>>()?;

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
    handler = |state: &mut State| {
        let expression = state.get_variable("expression").unwrap().to_string();

        state.scope_into()?;
        state.lock_scope();
        let res = Lavendeux::eval(&expression, state);
        state.scope_out();

        let res = res?;
        if res.len() == 1 {
            let res = res.as_a::<Vec<Value>>().unwrap();
            Ok(Value::from(res[0].clone()))
        } else {
            Ok(Value::from(res))
        }
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
            include('examples/stdlib_example.lav')
        ",
    },
    handler = |state: &mut State| {
        let script = state.get_variable("filename").unwrap().to_string();
        let script = std::fs::read_to_string(script)?;

        state.scope_into()?;
        state.lock_scope();
        let res = Lavendeux::eval(&script, state);
        state.scope_out();

        res?;
        Ok(Value::from(""))
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
    handler = |state: &mut State| {
        Ok(DocumentationTemplate::new(MarkdownFormatter).render(state).into())
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
        ",
        examples: "
            assert(true)
            assert( would_err('assert(false)') )
        ",
    },
    handler = |state: &mut State| {
        let cond = state.get_variable("condition").unwrap();
        if cond.is_truthy() {
            Ok(cond)
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
    handler = |state: &mut State| {
        let cond = state.get_variable("condition").unwrap();
        let expected = state.get_variable("expected").unwrap();

        if cond == expected {
            Ok(cond)
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
    handler = |state: &mut State| {
        let expression = state.get_variable("expression").unwrap().to_string();
        let res = crate::Lavendeux::eval(&expression, state);
        match res {
            Ok(_) => Ok(Value::from(false)),
            Err(_) => Ok(Value::from(true))
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
    handler = |state: &mut State| {
        let message = state.get_variable("message").unwrap().to_string();
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
    handler = |state: &mut State| {
        let message = state.get_variable("message").unwrap().to_string();
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
            } else { 0 }
            assert_eq(5, x)
        ",
    },
    handler = |state: &mut State| {
        let name = state.get_variable("name").unwrap().to_string();
        let value = state.get_variable("value").unwrap();
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
    handler = |state: &mut State| {
        let name = state.get_variable("name").unwrap().to_string();
        let value = state.get_variable("value").unwrap();
        state.global_assign_variable(&name, value.clone());
        Ok(value)
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
    handler = |state: &mut State| {
        let obj = Object::try_from(
            state
                .all_variables_unscoped()
                .iter()
                .map(|(k, v)| (Value::from(k.to_string()), v.clone()))
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
    handler = |state: &mut State| {
        let value = state.get_variable("value").unwrap();
        Ok(Value::string(value.own_type().to_string()))
    },
);
