use super::*;
use crate::{
    error::WrapExternalError,
    error_matches,
    pest::{Rule, ToAstNode},
    token::ToToken,
    State,
};
use pest::iterators::Pair;
use polyvalue::{
    types::{Object, Range},
    Value, ValueTrait, ValueType,
};

use super::Node;

define_node!(
    BreakExpression,
    rules = [BREAK_KEYWORD],
    new = |input: Pair<Rule>| {
        let token = input.to_token();
        let mut children = input.into_inner();
        children.next(); // Break keyword

        Ok(Self { token }.boxed())
    },
    value = |this: &BreakExpression, _state: &mut State| { oops!(Break, this.token.clone()) }
);

define_node!(
    SkipExpression,
    rules = [SKIP_KEYWORD],
    new = |input: Pair<Rule>| {
        let token = input.to_token();
        let mut children = input.into_inner();
        children.next(); // skip keyword

        Ok(Self { token }.boxed())
    },
    value = |this: &SkipExpression, _state: &mut State| { oops!(Skip, this.token.clone()) }
);

define_node!(
    ReturnExpression { expression: Node },
    rules = [RETURN_EXPRESSION],
    new = |input: Pair<Rule>| {
        let token = input.to_token();
        let mut children = input.into_inner();

        let expression = children.next().unwrap().to_ast_node()?;

        Ok(Self { expression, token }.boxed())
    },
    value = |this: &ReturnExpression, state: &mut State| {
        let value = this.expression.get_value(state)?;
        oops!(Return { value: value }, this.token.clone())
    }
);

define_node!(
    IfExpression {
        condition: Node,
        then_branch: Node,
        else_branch: Node
    },
    rules = [IF_EXPRESSION],
    new = |input: Pair<Rule>| {
        let token = input.to_token();
        let children = input.into_inner().collect::<Vec<_>>();
        if children.len() % 2 == 0 {
            // We parse as a set of (if, then) pairs ending with an else
            // if the number of children is even, we have no else
            return oops!(NoElseBlock, token);
        }

        // We will begin at the end, the final right-side expression
        // Then we will work backwards, grabbing pairs of expressions
        // And turning them into ternary expressions with the previous
        // iteration as the false side until we run out of children
        let mut children = children.into_iter().rev().peekable();
        let mut else_branch = children.next().unwrap().to_ast_node()?;
        while children.peek().is_some() {
            let then_branch = children.next().unwrap().to_ast_node()?;
            let condition = children.next().unwrap().to_ast_node()?;

            else_branch = Self {
                condition,
                then_branch,
                else_branch,
                token: token.clone(),
            }
            .boxed();
        }

        Ok(else_branch)
    },
    value = |this: &IfExpression, state: &mut State| {
        let condition = this.condition.get_value(state)?;
        state.scope_into().with_context(this.token())?;
        let result = if condition.is_truthy() {
            this.then_branch.get_value(state)
        } else {
            this.else_branch.get_value(state)
        };

        state.scope_out();
        result
    },

    docs = {
        name: "If",
        symbols = ["if <condition> then <block> else <block>", "if <condition> {block} else {block}"],
        description: "
            A conditional expression that evaluates a condition and then one of two branches.
            body can be either a block or a single expression. The last expression is returned from a block.
            Since all expressions in lavendeux return a value, the if expression will return the value of the branch that is executed.
            As such, all if expressions must have both a then and an else branch.
        ",
        examples: "
            a = 6
            if a > 5 { a } else { 5 }
            if a == 4 {
                a
            } else if a == 5 {
                5
            } else {
                6
            }
        ",
    }
);

#[derive(Debug)]
pub enum SwitchCase {
    Default(Node),
    Case(Node, Node),
}

define_node!(
    SwitchExpression {
        match_on: Node,
        cases: Vec<SwitchCase>
    },
    rules = [SWITCH_EXPRESSION],

    new = |input: Pair<Rule>| {
        let token = input.to_token();
        let mut children = input.into_inner();

        let match_on = children.next().unwrap().to_ast_node()?;
        let mut cases = vec![];

        while let Some(case) = children.next()  {
            let body = children.next().unwrap().to_ast_node()?;

            if case.as_str() == "_" {
                cases.push(SwitchCase::Default(body));
                if children.next().is_some() {
                    return oops!(UnreachableSwitchCase, token);
                }

                break;
            } else {
                cases.push(SwitchCase::Case(case.to_ast_node()?, body));
            }
        }

        Ok(Self {
            match_on,
            cases,
            token,
        }
        .boxed())
    },

    value = |this: &SwitchExpression, state: &mut State| {
        let match_on = this.match_on.get_value(state)?;

        for case in &this.cases {
            match case {
                SwitchCase::Default(body) => {
                    state.scope_into().with_context(this.token())?;
                    let result = body.get_value(state);

                    state.scope_out();
                    return result;
                },

                SwitchCase::Case(value, body) => {
                    let value = value.get_value(state)?;

                    if value.own_type() != match_on.own_type() {
                        return oops!(SwitchCaseTypeMismatch {
                            case: value,
                            expected_type: match_on.own_type()
                        }, this.token.clone());
                    }

                    if value == match_on {
                        state.scope_into().with_context(this.token())?;
                        let result = body.get_value(state);

                        state.scope_out();
                        return result;
                    }
                }
            }
        }

        oops!(NonExhaustiveSwitch, this.token.clone())
    },

    docs = {
        name: "match",
        symbols = ["match <value> { <condition> => <block>, _ => <block> }"],
        description: "
            A conditional expression that evaluates a value and then one of several cases.
            match blocks must be exhaustive, and therefore must end in a default case
        ",
        examples: "
            a = 6
            match a {
                5 => { 'five' },
                6 => { 'six' },
                _ => { 'other' }
            }
        ",
    }
);

define_node!(
    ForExpression {
        iterable: Node,
        variable: Option<String>,
        body: Node
    },
    rules = [FOR_LOOP_EXPRESSION],

    new = |input: Pair<Rule>| {
        let token = input.to_token();
        let mut children = input.into_inner().rev();

        let body = children.next().unwrap().to_ast_node()?;
        let iterable = children.next().unwrap().to_ast_node()?;
        let variable = children.next().map(|c| c.as_str().to_string());

        Ok(Self {
            variable,
            iterable,
            body,
            token
        }.boxed())
    },

    value = |this: &ForExpression, state: &mut State| {
        let iterable = this.iterable.get_value(state)?;

        // An iterator over Into<Value>
        let iterable = match iterable.own_type() {
            ValueType::Range => {
                let iterable = iterable.as_a::<Range>().with_context(this.token())?.inner().clone().into_iter();
                let mut values = vec![];
                for i in iterable {
                    values.push(i.into());
                    state.check_timer()?; // Potentially long-running operation
                }
                values.into_iter()
            },

            ValueType::Object => {
                let iterable = iterable.as_a::<Object>().with_context(this.token())?;
                iterable.inner().keys().cloned().collect::<Vec<_>>().into_iter()
            },

            _ => {
                let iterable = iterable.as_a::<polyvalue::types::Array>().with_context(this.token())?;
                iterable.inner().clone().into_iter()
            }
        };

        let mut result = vec![];
        for v in iterable {
            state.check_timer()?; // Potentially long-running operation

            state.scope_into().with_context(this.token())?;
            if let Some(variable) = &this.variable {
                state.set_variable(variable, v);
            }

            let value = this.body.get_value(state);
            state.scope_out();
            match value {
                Ok(value) => result.push(value),
                Err(e) if error_matches!(e, Skip) => {},
                Err(e) if error_matches!(e, Break) => {
                    break;
                },

                Err(e) => {
                    return Err(e);
                }
            }
        }

        Ok(Value::array(result))
    },

    docs = {
        name: "For",
        symbols = ["for <variable> in <iterable> { <block> }", "for [<variable> in] <iterable> do <block>"],
        description: "
            A loop that iterates over a range, array, or object.
            The variable is optional, and if not provided, the loop will not bind a variable.
            The expression will return an array of the results of the block.
            Break and skip/continue can be used to exit the loop or skip the current iteration.
        ",
        examples: "
            for i in 0..10 { i }
            for i in [1, 2, 3] { i }
            for i in {'a': 1, 'b': 2} { i }

            for 0..10 do '!'
        ",
    }
);

define_prattnode!(
    TernaryExpression {
        condition: Node,
        then_branch: Node,
        else_branch: Node
    },
    rules = [OP_TERNARY],
    new = |input: PrattPair| {
        let token = input.as_token();
        let mut children = input.into_inner();

        let condition = children.next().unwrap().to_ast_node()?;
        let then_branch = children
            .next()
            .unwrap()
            .first_pair()
            .into_inner()
            .next()
            .unwrap()
            .to_ast_node()?;
        let else_branch = children.next().unwrap().to_ast_node()?;

        Ok(Self {
            condition,
            then_branch,
            else_branch,
            token,
        }
        .boxed())
    },
    value = |this: &TernaryExpression, state: &mut State| {
        let condition = this.condition.get_value(state)?;
        state.scope_into().with_context(this.token())?;
        let result = if condition.is_truthy() {
            this.then_branch.get_value(state)
        } else {
            this.else_branch.get_value(state)
        };

        state.scope_out();
        result
    },

    docs = {
        name: "Ternary",
        symbols = ["condition ? then : else"],
        description: "
            A right-associative ternary operator.
            The condition is evaluated first, then either the then or else branch is evaluated.
        ",
        examples: "true ? 1 : 2",
    }
);