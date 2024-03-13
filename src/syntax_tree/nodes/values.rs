use super::Node;
use crate::{
    error::WrapExternalError,
    syntax_tree::{assignment_target::AssignmentTarget, traits::IntoNode},
    Error, Rule, State, Token,
};
use polyvalue::{
    operations::{MatchingOperation, MatchingOperationExt},
    Value, ValueType,
};

define_handler!(
    Identifier(_pairs, token, _state) {
        let name = token.input.to_string();
        Ok(Reference::new(AssignmentTarget::Identifier(name), token).into())
    }
);
document_operator!(
    name = "Identifier",
    rules = [],
    symbols = ["a", "b", "c"],
    description = "
        A variable name.
        The value of the variable is looked up in the state.
    ",
    examples = "
        [a, b, c] = [1, 2, 3]
        a; b; c
    ",
);

define_ast!(
    Values {
        Reference(target: AssignmentTarget<'i>) {
            build = (_pairs, token, _state) {
                oops!(Internal {
                    msg: "Reference node should not be built directly".to_string()
                }, token)
            },

            eval = (this, state) {
                this.target.get_value(state).with_context(this.token())
            },

            owned = (this) {
                Self::Owned {
                    target: this.target.into_owned(),
                    token: this.token.into_owned(),
                }
            }
        },

        CastExpression(value: Node<'i>, target: Node<'i>) {
            build = (pairs, token, state) {
                let mut pairs = pairs;
                let value = unwrap_node!(pairs, state, token)?;
                pairs.next(); // skip the operator
                let target = unwrap_node!(pairs, state, token)?;

                Ok(Self {
                    value,
                    target,
                    token,
                }
                .into())
            },
            eval = (this, state) {
                let value = this.value.evaluate(state).with_context(this.token())?;
                let target = if this.target.token().rule == Rule::identifier {
                    this.target.token().input.to_string()
                } else {
                    this.target.evaluate(state).with_context(this.token())?.to_string()
                };

                let target = ValueType::try_from(target.as_str()).with_context(this.token())?;
                value.as_type(target).with_context(this.token())
            },
            owned = (this) {
                Self::Owned {
                    value: this.value.into_owned(),
                    target: this.target.into_owned(),
                    token: this.token.into_owned(),
                }
            },

            docs = {
                name: "Cast",
                symbols = ["as"],
                description: "
                    Casts a value to a different type.
                    The type can be a string or an identifier.
                    The operator is right-associative
                ",
                examples: "
                    5 as float
                    5 as 'float'
                    5 as i8
                ",
            }
        },

        DecoratorExpression(expression: Node<'i>, decorator: String) {
            build = (pairs, token, state) {
                let mut pairs = pairs;
                let expression = unwrap_node!(pairs, state, token)?;

                let mut decorator_pair = unwrap_next!(pairs, token);
                let decorator = unwrap_next!(decorator_pair, token).as_str().to_string();

                Ok(Self {
                    expression,
                    decorator,
                    token,
                }
                .into())
            },
            eval = (this, state) {
                let value = this.expression.evaluate(state).with_context(this.token())?;
                let result = state.decorate(&this.decorator, value).with_context(this.token())?;
                Ok(Value::from(result))
            },
            owned = (this) {
                Self::Owned {
                    expression: this.expression.into_owned(),
                    decorator: this.decorator,
                    token: this.token.into_owned(),
                }
            },

            docs = {
                name: "Decorator",
                symbols = ["@name"],
                description: "
                    Converts a value to a formatted string.
                    It calls a function named '@name' with the value as an argument.
                ",
                examples: "
                    assert_eq(
                        5 @float,
                        '5.0'
                    )
                ",
            }
        },

        MatchingExpression(
            left: Node<'i>,
            right: Node<'i>,
            operator: MatchingOperation
        ) {
            build = (pairs, token, state) {
                let mut pairs = pairs;
                let left = unwrap_node!(pairs, state, token)?;
                let operator = unwrap_next!(pairs, token).as_rule();
                let right = unwrap_node!(pairs, state, token)?;

                let operator = match operator {
                    Rule::OP_MATCH_CONTAINS => MatchingOperation::Contains,
                    Rule::OP_MATCH_MATCHES => MatchingOperation::Matches,
                    Rule::OP_MATCH_IS => MatchingOperation::Is,
                    Rule::OP_MATCH_STARTSWITH => MatchingOperation::StartsWith,
                    Rule::OP_MATCH_ENDSWITH => MatchingOperation::EndsWith,
                    _ => {
                        return oops!(
                            Internal {
                                msg: format!("Unrecognize matching operator {operator:?}")
                            },
                            token
                        )
                    }
                };

                Ok(Self {
                    left,
                    right,
                    operator,
                    token,
                }
                .into())
            },
            eval = (this, state) {
                let left = this.left.evaluate(state).with_context(this.token())?;
                let right = if this.operator == MatchingOperation::Is
                    && this.right.token().rule == Rule::identifier
                {
                    Value::from(&*this.right.token().input)
                } else {
                    this.right.evaluate(state).with_context(this.token())?
                };

                Value::matching_op(&left, &right, this.operator).with_context(this.token())
            },
            owned = (this) {
                Self::Owned {
                    left: this.left.into_owned(),
                    right: this.right.into_owned(),
                    operator: this.operator,
                    token: this.token.into_owned(),
                }
            },

            docs = {
                name: "Matching",
                symbols = ["contains", "matches", "is", "starts_with", "ends_with"],
                description: "
                    A set of left-associative boolean operators comparing a collection with a pattern
                    'is' is a special case that compares type (`value is string` is equivalent `typeof(value) == 'string'`)
                    starts/ends with are not applicable to objects, which are not ordered
                ",
                examples: "
                    {'name': 'test'} contains 'name'
                    'hello' matches 'ell'
                    'hello' is string
                    'hello' starts_with 'hel'
                    [1, 2] endswith 2
                ",
            }
        }
    }
);

impl<'i> Reference<'i> {
    pub(crate) fn new(target: AssignmentTarget<'i>, token: Token<'i>) -> Reference<'i> {
        Self { target, token }
    }

    /// Get the reference's value from the state
    pub fn get_value(&self, state: &mut State) -> Result<Value, Error> {
        self.target.get_value(state)
    }

    /// Get the reference's value from the state
    /// Uses the parent scope (if a function defines a value of the same name)
    pub fn get_value_in_parent(&self, state: &mut State) -> Result<Value, Error> {
        self.target.get_value_in_parent(state)
    }

    /// Update the reference's value in the state
    pub fn update_value(&self, state: &mut State, value: Value) -> Result<(), Error> {
        self.target.update_value(state, value)
    }

    /// Update the reference's value in the state
    /// Uses the parent scope (if a function defines a value of the same name)
    pub fn update_value_in_parent(&self, state: &mut State, value: Value) -> Result<(), Error> {
        self.target.update_value_in_parent(state, value)
    }

    /// Get the reference's value from the state as a mutable reference
    pub fn get_target_mut_in_parent<'s>(
        &self,
        state: &'s mut State,
    ) -> Result<Option<&'s mut Value>, Error> {
        self.target.get_target_mut_in_parent(state)
    }
}
