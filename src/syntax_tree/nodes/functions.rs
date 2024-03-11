use super::Node;
use crate::{
    error::{ErrorDetails, WrapExternalError},
    functions::{ParserFunction, UserDefinedFunction},
    syntax_tree::traits::IntoNode,
    Error, Rule, Token,
};
use polyvalue::{Value, ValueType};

define_ast!(
    Functions {
        KeywordReturn(value: Box<Node<'i>>) {
            build = (pairs, token, state) {
                let value = Box::new(unwrap_node!(pairs, state, token)?);
                Ok(Self { value, token }.into())
            },
            eval = (this, state) {
                let value = this.value.evaluate(state).with_context(this.token())?;
                oops!(Return { value }, this.token().clone())
            },
            owned = (this) {
                Self::Owned {
                    value: Box::new(this.value.into_owned()),
                    token: this.token.into_owned(),
                }
            },
            docs = {
                name: "Return",
                symbols = ["return <value>"],
                description: "
                    Returns a value from a function.
                    By default, the last expression is returned, unless a return statement is used.
                ",
                examples: "
                    a() = { return 5 ; 2 }
                    b() = { 5 ; 2 }
                    
                    assert_eq( a(), 5 )
                    assert_eq( b(), 2 )
                ",
            }
        },

        FunctionCall(name: String, arguments: Vec<Node<'i>>) {
            build = (pairs, token, state) {
                let lhs = unwrap_next!(pairs, token); // Function name, or the first argument of an object mode call
                let mut rhs = unwrap_next!(
                    unwrap_next!(pairs, token),
                    token
                ); // Arguments, or function name and arguments in object mode

                let mut node = match rhs.as_rule() {
                    Rule::POSTFIX_NORMALMODE => {
                        let name = lhs.as_str().to_string();
                        let arguments = if &name == "help" {
                            match rhs.next() {
                                Some(arg) => vec![Node::Literal(Value::from(arg.as_str().to_string()), token.clone())],
                                None => Vec::new(),
                            }
                        } else {
                            rhs.map(|p| p.into_node(state)).collect::<Result<Vec<_>, _>>().with_context(&token)?
                        };

                        Self { name, arguments, token }
                    }

                    // Rule::POSTFIX_OBJECTMODE
                    _ => {
                        let mut rhs = rhs;
                        let name = unwrap_next!(rhs, token).as_str().to_string();

                        let rhs = unwrap_next!(rhs, token);
                        let arguments = vec![lhs.into_node(state)] // First argument
                            .into_iter()
                            .chain(rhs.map(|p| p.into_node(state)))
                            .collect::<Result<Vec<_>, _>>().with_context(&token)?;

                        Self { name, arguments, token }
                    }
                };

                Ok(node.into())
            },
            eval = (this, state) {
                if &this.name == "help" {
                    let filter = match this.arguments.first() {
                        Some(n) => Some(n.evaluate(state).with_context(this.token())?.to_string()),
                        None => None
                    };

                    let help_text = state.help(filter);
                    return Ok(Value::from(help_text));
                }

                // Collect arguments
                let mut arguments = Vec::new();

                for argument in this.arguments.iter() {
                    arguments.push(argument.evaluate(state).with_context(this.token())?);
                }

                let value = match state.call_function(&this.name, arguments) {
                    Ok(value) => value,
                    Err(e) => {
                        if let ErrorDetails::Return { value, .. } = e.details {
                            value
                        } else {
                            return Err(ErrorDetails::FunctionCall { name: this.name.clone() })
                            .with_context(this.token())
                            .with_source(e)
                        }
                    },
                };

                Ok(value)
            },
            owned = (this) {
                Self::Owned {
                    name: this.name,
                    arguments: this.arguments
                        .into_iter()
                        .map(|s| s.into_owned())
                        .collect(),
                    token: this.token.into_owned(),
                }
            },

            docs = {
                name: "Function Call",
                symbols = ["name(arg1, arg2, ...)", "arg1.func(arg2, arg3, ...)"],
                description: "
                    Calls a function with the given arguments.
                    The help() will list all available functions, and can filter by category or function name.
                ",
                examples: "
                    arr = []
                    push(arr, 3)
                    arr.push(3)
                    help(push)
                    help(collections)
                ",
            }
        }
    }
);

define_handler!(
    FunctionDefinition(pairs, token, state) {
        let name = unwrap_next!(pairs, token).as_str().to_string();
        let src = pairs.last_child().unwrap().as_str().to_string();

        let mut returns = match pairs.peek_last() {
            Some(p) if p.as_rule() == Rule::function_typespec => {
                let t = pairs.last_child().unwrap().last_child().unwrap();
                let t = t.as_str();
                ValueType::try_from(t).with_context(&token)?
            },
            _ => ValueType::Any
        };

        let arguments = pairs.map(|arg| {
            let mut arg = arg;
            let name = unwrap_next!(arg, token).as_str().to_string();
            let t = match arg.next() {
                Some(t) => {
                    let t = t.as_str();
                    ValueType::try_from(t).with_context(&token)?
                }
                None => ValueType::Any
            };
            Ok((name, t))
        }).collect::<Result<Vec<_>, Error>>().with_context(&token)?;

        // Make sure decorators follow the `@name(in): string` signature
        if name.starts_with('@') {
            if arguments.len() != 1 {
                return Err(ErrorDetails::DecoratorSignatureArgs { name: name.clone() })
                .with_context(&token);
            } else if returns != ValueType::Any {
                return Err(ErrorDetails::DecoratorSignatureReturn { name: name.clone() })
                .with_context(&token);
            }

            returns = ValueType::String;
        }

        let mut function = UserDefinedFunction::new(&name, src.clone(), state).with_context(&token)?;
        function.set_returns(returns);
      //  function.set_src_line_offset(token.line);

        for (name, t) in arguments.iter() {
            function.add_arg(name, *t);
        }

        let sig = function.signature();
        state.register_function(function).with_context(&token)?;
        Ok(Node::Literal(Value::from(sig), token))
    }
);
document_operator!(
    name = "Function Assignment",
    rules = [],
    symbols = ["name([arg1:type, arg2, ...]) = { ... }"],
    description = "
        Assigns a block of code to a function name.
        The function can be called later in the code.
        If the function name begins with `@`, it is a decorator and must take in one argument and return a string

        Function body can be a block of code or a single expression. The last expression is returned, unless a return statement is used.
        Return type or argued types can be specified with `: type`, but are optional.

        Arguments will be cooerced to the specified type if provided, as will the return value.
        Valid type names are: `u[8-64]`, `i[8-64]`, `float`, `int`, `numeric`, `string`, `array`, `object`, `bool`, `any`.
    ",
    examples = "
        // Decorator taking in a number and returning a string
        @double(x:numeric) = 2*x
        5 @double

        // Takes in any 2 numeric values, and returns an integer (i64 by default)
        add(a:numeric, b:numeric): int = {
            a + b
        }
        add(3, 4.5)
    ",
);
