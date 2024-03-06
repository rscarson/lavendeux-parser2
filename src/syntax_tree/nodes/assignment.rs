use polyvalue::{
    operations::{
        ArithmeticOperation, ArithmeticOperationExt, BitwiseOperation, BitwiseOperationExt,
        BooleanOperation, BooleanOperationExt,
    },
    Value,
};

use crate::{
    error::WrapExternalError,
    syntax_tree::{
        assignment_target::AssignmentTarget,
        traits::{IntoNode, NodeExt, SyntaxNodeBuilderExt},
    },
    Error, Rule, State,
};

use super::{Node, Reference};

#[derive(Debug, Clone, Copy)]
#[rustfmt::skip]
pub enum AssignmentOperation {
    Add, Sub, Mul, Div, Mod, Pow,
    BitAnd, BitOr, BitXor, BitSl, BitSr,
    And, Or, None
}
impl AssignmentOperation {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn is_some(&self) -> bool {
        !self.is_none()
    }

    pub fn apply(&self, state: &mut State, target: &Reference, rhs: Value) -> Result<Value, Error> {
        let value = if self.is_none() {
            rhs
        } else {
            let lhs = target.get_value(state)?.clone();
            let rhs = rhs.as_type(lhs.own_type())?;
            
            match self {
                Self::Add => lhs.arithmetic_op(rhs, ArithmeticOperation::Add)?,
                Self::Sub => lhs.arithmetic_op(rhs, ArithmeticOperation::Subtract)?,
                Self::Mul => lhs.arithmetic_op(rhs, ArithmeticOperation::Multiply)?,
                Self::Div => lhs.arithmetic_op(rhs, ArithmeticOperation::Divide)?,
                Self::Mod => lhs.arithmetic_op(rhs, ArithmeticOperation::Modulo)?,
                Self::Pow => lhs.arithmetic_op(rhs, ArithmeticOperation::Exponentiate)?,

                Self::BitAnd => lhs.bitwise_op(rhs, BitwiseOperation::And)?,
                Self::BitOr => lhs.bitwise_op(rhs, BitwiseOperation::Or)?,
                Self::BitXor => lhs.bitwise_op(rhs, BitwiseOperation::Xor)?,
                Self::BitSl => lhs.bitwise_op(rhs, BitwiseOperation::LeftShift)?,
                Self::BitSr => lhs.bitwise_op(rhs, BitwiseOperation::RightShift)?,

                Self::And => lhs.boolean_op(rhs, BooleanOperation::And)?,
                Self::Or => lhs.boolean_op(rhs, BooleanOperation::Or)?,

                Self::None => rhs,
            }
        };

        target.update_value(state, value.clone())?;
        Ok(value)
    }
}
impl From<Rule> for AssignmentOperation {
    fn from(value: Rule) -> Self {
        match value {
            Rule::OP_ASSIGN_ADD => Self::Add,
            Rule::OP_ASSIGN_SUB => Self::Sub,
            Rule::OP_ASSIGN_POW => Self::Pow,
            Rule::OP_ASSIGN_MUL => Self::Mul,
            Rule::OP_ASSIGN_DIV => Self::Div,
            Rule::OP_ASSIGN_MOD => Self::Mod,
            Rule::OP_BASSIGN_AND => Self::And,
            Rule::OP_BASSIGN_OR => Self::Or,
            Rule::OP_ASSIGN_AND => Self::BitAnd,
            Rule::OP_ASSIGN_XOR => Self::BitXor,
            Rule::OP_ASSIGN_OR => Self::BitOr,
            Rule::OP_ASSIGN_SL => Self::BitSl,
            Rule::OP_ASSIGN_SR => Self::BitSr,
            _ => Self::None,
        }
    }
}

define_ast!(
    Assignment {
        DeleteExpression(target: AssignmentTarget<'i>) {
            build = (pairs, token, state) {
                let op = pairs.next().unwrap();
                let is_decorator = op.as_str().ends_with('@');
                let target = pairs.next().unwrap();

                let target = target.into_node(state).with_context(&token)?;
                if let node_type!(Values::Reference(target)) = target {
                    let mut target = target.target;
                    match target {
                        AssignmentTarget::Identifier(ref mut id) => {
                            if is_decorator {
                                *id = format!("@{id}");
                            }
                        }
                        _ if is_decorator => {
                            return oops!(
                                DecoratorName {
                                    name: target.to_string()
                                },
                                token
                            );
                        }
                        _ => {}
                    }


                    Ok(Self { target, token }.into())
                } else if let node_type!(Collections::Array(target)) = target {
                    let target = target.elements.into_iter().map(|e| {
                        if let node_type!(Values::Reference(target)) = e {
                            Ok(target.target)
                        } else {
                            oops!(ConstantValue, e.token().clone())
                        }
                    }).collect::<Result<Vec<_>, _>>().with_context(&token)?;
                    let target = AssignmentTarget::Destructure(target);
                    Ok(Self { target, token }.into())
                } else {
                    oops!(ConstantValue, token)
                }
            },
            eval = (this, state) {
                this.target.delete(state).with_context(this.token())
            },
            owned = (this) {
                Self::Owned {
                    target: this.target.into_owned(),
                    token: this.token.into_owned()
                }
            },
            docs = {
                name: "Deletion Keyword",
                symbols = ["del", "delete", "unset"],
                description: "
                    Deletes a value, function, @decorator, or index
                    Will return the value deleted (or the function signature if a function was deleted)
                    Index can be blank to delete the last value in an array, or negative to count from the end
                    Indices can also be a collection to delete multiple values at once
                ",
                examples: "
                    a = 2; del a
                    a = [1]; del a[]
                    a = {'test': 1}; del a['test']
                    a=1;b=2; del [a,b]
        
                    @dec(x) = 2
                    del @dec
                ",
            }
        },

        AssignmentExpression(target: Reference<'i>, op: AssignmentOperation, rhs: Box<Node<'i>>) {
            build = (pairs, token, state) {

                let lhs = pairs.next().unwrap();
                let lhs = lhs.into_node(state).with_context(&token)?;
                let op = AssignmentOperation::from(pairs.next().unwrap().as_rule());
                let rhs = Box::new(pairs.next().unwrap().into_node(state).with_context(&token)?);

                if let node_type!(Values::Reference(target)) = lhs {
                    Ok(Self { target, op, rhs, token }.into())
                } else if let node_type!(Collections::Array(target)) = lhs {
                    let lhs_token = target.token().clone();
                    let t = target.elements.into_iter().map(|e| {
                        if let node_type!(Values::Reference(target)) = e {
                            Ok(target.target)
                        } else {
                            oops!(ConstantValue, e.token().clone())
                        }
                    }).collect::<Result<Vec<_>, _>>().with_context(&token)?;
                    let t = AssignmentTarget::Destructure(t);
                    let target = Reference::new(t, lhs_token);
                    Ok(Self { target, op, rhs, token }.into())
                } else {
                    return oops!(ConstantValue, token);
                }
            },
            eval = (this, state) {
                let rhs = this.rhs.evaluate(state).with_context(this.token())?;
                this.op.apply(state, &this.target, rhs)
            },
            owned = (this) {
                Self::Owned {
                    target: this.target.into_owned(),
                    op: this.op,
                    rhs: Box::new(this.rhs.into_owned()),
                    token: this.token.into_owned(),
                }
            },
            docs = {
                name: "Assignment Operator",
                symbols = ["=", "+=", "-=", "*=", "/=", "%=", "**=", "&=", "|=", "^=", "<<=", ">>="],
                description: "
                    Assigns a value to a variable, index, or destructuring assignment
                    Target is either a literal with optional indices, or a destructuring assignment
                    If an index is empty, a new value will be appended to the array
                    If the target is a destructuring assignment, the value must be a collection of the same length
                    If the operator is present, the value will be transformed before assignment
                ",
                examples: "
                    [a, b] = [1, 2]
                    a = 1; a += 1
                    a = [1]; a[] = 2
                ",
            }
        }
    }
);
