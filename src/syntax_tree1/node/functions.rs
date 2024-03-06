use super::*;
use crate::{
    error::{ErrorDetails, WrapExternalError},
    functions::{ParserFunction, UserDefinedFunction},
    Rule, ToToken,
};
use polyvalue::{Value, ValueType};

// DECORATOR_SYMBOL? ~ identifier ~ "(" ~ (identifier ~ ("," ~ identifier)* ~ ","?)? ~ ")" ~ "=" ~ BLOCK
define_node!(
    FunctionAssignment {
        name: String,
        returns: ValueType,
        arguments: Vec<(String, ValueType)>,
        src: Vec<String>
    },
    rules = [FUNCTION_ASSIGNMENT_STATEMENT],

    new = (input) {
        let token = input.to_token();
        let mut children = input.into_inner();

        let name = children.next().unwrap().as_str().to_string();
        let mut children = children.rev().peekable();
        let src = children.next().unwrap().into_inner().map(|p| p.as_str().to_string()).collect::<Vec<_>>();

        let mut returns = match children.peek() {
            Some(p) if p.as_rule() == Rule::function_typespec => {
                let t = children.next().unwrap().into_inner().next().unwrap().as_str();
                ValueType::try_from(t)?
            },
            _ => ValueType::Any
        };

        let arguments = children.rev().map(|arg| {
            let mut arg = arg.into_inner();
            let name = arg.next().unwrap().as_str().to_string();
            let t = match arg.next() {
                Some(t) => {
                    let t = t.as_str();
                    ValueType::try_from(t)?
                }
                None => ValueType::Any
            };
            Ok((name, t))
        }).collect::<Result<Vec<_>, Error>>()?;

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

        Ok(Self {
            name,
            arguments,
            returns,
            src,
            token,
        }.boxed())
    },

    value = (this, state) {
        let mut function = UserDefinedFunction::new(&this.name, this.src.clone())?;
        function.set_returns(this.returns);
        function.set_src_line_offset(this.token().line);

        for (name, t) in this.arguments.iter() {
            function.add_arg(name, *t);
        }

        let sig = function.signature();
        state.register_function(function)?;
        Ok(Value::from(sig))
    },

    into_owned = (this) {
        Self {
            name: this.name.clone(),
            returns: this.returns,
            arguments: this.arguments.clone(),
            src: this.src.clone(),
            token: this.token.clone(),
        }
        .boxed()
    },

    docs = {
        name: "Function Assignment",
        symbols = ["name([arg1:type, arg2, ...]) = { ... }"],
        description: "
            Assigns a block of code to a function name.
            The function can be called later in the code.
            If the function name begins with `@`, it is a decorator and must take in one argument and return a string

            Function body can be a block of code or a single expression. The last expression is returned, unless a return statement is used.
            Return type or argued types can be specified with `: type`, but are optional.

            Arguments will be cooerced to the specified type if provided, as will the return value.
            Valid type names are: `u[8-64]`, `i[8-64]`, `float`, `int`, `numeric`, `string`, `array`, `object`, `bool`, `any`.
        ",
        examples: "
            // Decorator taking in a number and returning a string
            @double(x:numeric) = 2*x
            5 @double

            // Takes in any 2 numeric values, and returns an integer (i64 by default)
            add(a:numeric, b:numeric): int = {
                a + b
            }
            add(3, 4.5)
        ",
    }
);

// ("." ~ identifier)? ~ "(" ~ ((EXPR ~ ",")* ~ EXPR ~ ","?)? ~ ")"
define_prattnode!(
    FunctionCall {
        name: String,
        arguments: Vec<Node<'i>>
    },
    rules = [POSTFIX_CALL],

    new = (input) {
        let mut token = input.as_token();
        let mut children = input.into_inner();

        let lhs = children.next().unwrap();
        let rhs = children.next().unwrap();
        let mut rhs = rhs.first_pair().into_inner();

        let mut arguments = Vec::new();

        let is_object_mode = rhs.peek().map_or(false, |p| p.as_rule() == Rule::POSTFIX_OBJECTMODE);

        let name = if is_object_mode {
            let name = rhs.next().unwrap().into_inner().next().unwrap().as_str().to_string();
            let _token = lhs.as_token();
            if &name == "help" && lhs.as_rule() == Rule::identifier {
                let _token = lhs.as_token();
                arguments.push(
                    literals::ConstantValue::new(
                        Value::from(lhs.first_pair().as_str().to_string()),
                        _token
                    )
                );
            } else {
                arguments.push(lhs.to_ast_node()?);
            }
            name
        } else {
            let name = lhs.first_pair().as_str().to_string();
            if &name == "help" {
                match rhs.next() {
                    Some(a) if a.as_rule() == Rule::identifier => {
                        let _token = a.to_token();
                        arguments.push(
                            literals::ConstantValue::new(
                                Value::from(a.as_str().to_string()),
                                _token
                            )
                        );
                    },
                    Some(a) => arguments.push(a.to_ast_node()?),
                    None => {}
                }
            }
            name
        };

        // Collect arguments
        for node in rhs {
            arguments.push(node.to_ast_node()?);
        }

        // Function arguments can have variable references to the first argument in order to
        // update values. This is especially useful for array functions like push, pop, etc.
        token.references = arguments.first().and_then(|c| c.token().references.clone());

        Ok(Self {
            name,
            arguments,
            token,
        }.boxed())
    },

    value = (this, state) {
        if &this.name == "help" {
            let filter = match this.arguments.get(0) {
                Some(n) => Some(n.get_value(state)?.to_string()),
                None => None
            };

            let help_text = state.help(filter);
            return Ok(Value::from(help_text));
        }

        // Collect arguments
        let mut arguments = Vec::new();

        for argument in this.arguments.iter() {
            arguments.push(argument.get_value(state)?);
        }

        let reference = this.token().references.as_deref();
        match state.call_function(&this.name, arguments, reference) {
            Ok(value) => Ok(value),
            Err(e) => {
                if let ErrorDetails::Return { value, .. } = e.details {
                    Ok(value)
                } else {
                    Err(ErrorDetails::FunctionCall { name: this.name.clone() })
                    .with_context(this.token())
                    .with_source(e)
                }
            },
        }
    },

    into_owned = (this) {
        Self {
            name: this.name.clone(),
            arguments: this.arguments.into_iter().map(|a| a.into_owned()).collect(),
            token: this.token.clone(),
        }
        .boxed()
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
);