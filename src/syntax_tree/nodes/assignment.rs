use super::Node;
use crate::{
    error::{ErrorDetails, WrapExternalError, WrapOption},
    syntax_tree::{
        assignment_target::AssignmentTarget,
        traits::{IntoNode, NodeExt, SyntaxNodeBuilderExt},
    },
    Error, Rule, State,
};
use polyvalue::{
    operations::{
        ArithmeticOperation, ArithmeticOperationExt, BitwiseOperation, BitwiseOperationExt,
        BooleanOperation, BooleanOperationExt,
    },
    Value,
};

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

    pub fn apply_to(
        &self,
        state: &mut State,
        target: &AssignmentTarget,
        rhs: Value,
    ) -> Result<Value, Error> {
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

    pub fn apply(
        &self,
        state: &mut State,
        target: &AssignmentTarget,
        rhs: Value,
    ) -> Result<Value, Error> {
        match &target {
            // Assign a single value to multiple targets
            AssignmentTarget::Destructure(targets) if rhs.len() == 1 => {
                for target in targets {
                    self.apply_to(state, target, rhs.clone())?;
                }
                target.get_value(state)
            }

            // Assign multiple values to multiple targets
            AssignmentTarget::Destructure(targets) if rhs.len() == targets.len() => {
                let rhs = rhs.as_a::<Vec<Value>>()?;
                for (target, value) in targets.into_iter().zip(rhs) {
                    self.apply_to(state, target, value)?;
                }
                target.get_value(state)
            }

            // Target count mismatch
            AssignmentTarget::Destructure(targets) => oops!(DestructuringAssignment {
                expected_length: targets.len(),
                actual_length: rhs.len()
            }),

            // Assign a single value to a single target
            _ => self.apply_to(state, &target, rhs),
        }
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
                let op = unwrap_next!(pairs, token);
                let is_decorator = op.as_str().ends_with('@');
                let target = unwrap_next!(pairs, token);

                let target = target.into_node(state).with_context(&token)?;
                let mut target = as_assignment_target!(target).or_error(ErrorDetails::ConstantValue).with_context(&token)?;

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
                    a = 2; del a        // Deletes the variable a
                    a = [1]; del a[]    // Deletes the last value in the array

                    // Deletes the key 'test' from an object
                    a = {'test': 1}
                    del a['test']   

                    a=1;b=2; del [a,b] // Deletes both a and b
        
                    @dec(x) = 2
                    del @dec    // Deletes the decorator
                ",
            }
        },

        AssignmentExpression(target: AssignmentTarget<'i>, op: AssignmentOperation, rhs: Node<'i>) {
            build = (pairs, token, state) {

                let lhs = unwrap_next!(pairs, token);
                let lhs = lhs.into_node(state).with_context(&token)?;
                let op = AssignmentOperation::from(unwrap_next!(pairs, token).as_rule());
                let rhs = unwrap_node!(pairs, state, token)?;

                let target = as_assignment_target!(lhs).or_error(ErrorDetails::ConstantValue).with_context(&token)?;
                Ok(Self { target, op, rhs, token }.into())
            },
            eval = (this, state) {
                let rhs = this.rhs.evaluate(state).with_context(this.token())?;
                this.op.apply(state, &this.target, rhs).with_context(this.token())
            },
            owned = (this) {
                Self::Owned {
                    target: this.target.into_owned(),
                    op: this.op,
                    rhs: this.rhs.into_owned(),
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

                    Operators:
                    - Arithmetic: `+=, -=, *=, /=, %=, **=`
                    - Bitwise: `&=, |=, ^=, <<=, >>=`
                    - Boolean: `&&=, ||=`

                    Note: Operators are not supported for destructuring assignments
                ",
                examples: "
                    [a, b] = [1, 2]     // Destructuring assignment
                    a = 1; a += 1       // Arithmetic assignment
                    a = [1]; a[] = 2    // Array index assignment (appends to array)
                ",
            }
        }
    }
);

#[cfg(test)]
mod test {
    use crate::lav;

    lav!(test_del_ident r#"
        a=1; del a
    "#);

    lav!(test_del_const(Error) r#"
        del 1
    "#);

    lav!(test_del_const_arr(Error) r#"
        a=1; del [a,1]
    "#);

    lav!(test_del_const_idx(Error) r#"
        a=1; del a[1]
    "#);

    lav!(test_assign_ops r#"
        a=1; a+=1; assert_eq(a, 2)
        b=1; b-=1; assert_eq(b, 0)
        c=1; c*=2; assert_eq(c, 2)
        d=4; d/=2; assert_eq(d, 2)
        ee=4; ee%=2; assert_eq(ee, 0)
        f=2; f**=3; assert_eq(f, 8)
        g=2; g&=3; assert_eq(g, 2)
        h=2; h|=3; assert_eq(h, 3)
        i=2; i^=3; assert_eq(i, 1)
        j=2; j<<=3; assert_eq(j, 16)
        k=2; k>>=3; assert_eq(k, 0)
        l=true ; l&&=false; assert_eq(l, false)
        m=true ; m||=false; assert_eq(m, true)
    "#);

    lav!(test_assign_destructure r#"
        [a, b] = [1, [1,2]]
        assert_eq(a, 1)
        assert_eq(b, [1,2])

        a = 1; b = 2;
        [a, b] = [1, 1]
        assert_eq(a, 1)
        assert_eq(b, 1)
        
        [a, b] = 1
        assert_eq(a, 1)
        assert_eq(b, 1)
    "#);

    lav!(test_assign_destructure_error(Error) r#"
        [a, b] = [1, 2, 3]
    "#);

    lav!(test_buggy_push r#"
        save = {'choices':[]}; choice = 5
        save['choices'].push(choice)
        assert_eq(save['choices'], [5])
    "#);

    lav!(test_assign_idx r#"
        a = [1, 2, 3]
        a[0] = 2
        assert_eq(a, [2, 2, 3])

        assert_eq(a[], 3)
        assert_eq(a[-1], 3)

        a[] = [[[3]]]
        assert_eq(a[][][0][], 3)
    "#);

    lav!(test_assign_idx_error(Error) r#"
        a = [1, 2, 3]
        a[4] = 2
    "#);

    lav!(test_assign_idx_error2(Error) r#"
        a = [1, 2, 3]
        a[0][1] = [1, 2]
    "#);
}
