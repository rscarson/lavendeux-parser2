use super::Node;
use crate::{
    error::WrapExternalError,
    syntax_tree::{
        assignment_target::Target,
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

    fn apply_to(&self, state: &mut State, target: &Target, rhs: Value) -> Result<Value, Error> {
        let value = if self.is_none() {
            rhs
        } else {
            let lhs = target.get(state)?;
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

        target.write(state, value.clone())?;
        Ok(value)
    }

    pub fn apply(&self, state: &mut State, targets: &[Target], rhs: Value) -> Result<Value, Error> {
        if targets.len() > 1 {
            if rhs.len() == 1 {
                let rhs = rhs.clone();
                let values = targets
                    .iter()
                    .map(|t| self.apply_to(state, t, rhs.clone()))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(values.into())
            } else if targets.len() != rhs.len() {
                oops!(DestructuringAssignment {
                    expected_length: targets.len(),
                    actual_length: rhs.len()
                })
            } else {
                for (target, value) in targets.iter().zip(rhs.clone().as_a::<Vec<Value>>()?.iter())
                {
                    self.apply_to(state, target, value.clone())?;
                }
                Ok(rhs)
            }
        } else {
            let target = &targets[0];
            self.apply_to(state, target, rhs)
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
        DeleteExpression(targets: Vec<Target<'i>>) {
            build = (pairs, token, state) {
                pairs.next(); // Skip the 'del' keyword
                let target = unwrap_next!(pairs, token);

                let target = target.into_node(state).with_context(&token)?;
                if let node_type!(Values::Reference(reference)) = target {
                    // Identifier or Index reference
                    Ok(Self { targets: vec![reference.target], token }.into())

                } else if let node_type!(Collections::Array(target)) = target {
                    // Destructuring assignment
                    let targets = target.elements.into_iter().map(|e| {
                        if let node_type!(Values::Reference(target)) = e {
                            Ok(target.target)
                        } else {
                            oops!(ConstantValue, e.token().clone())
                        }
                    }).collect::<Result<Vec<_>, _>>().with_context(&token)?;
                    Ok(Self { targets, token }.into())

                } else {
                    // Invalid target
                    oops!(ConstantValue, token)
                }
            },
            eval = (this, state) {
                let values = this.targets.iter().map(|t| t.delete(state)).collect::<Result<Vec<_>, _>>().with_context(this.token())?;
                Ok(values.into())
            },
            owned = (this) {
                Self::Owned {
                    targets: this.targets.into_iter().map(|t| t.into_owned()).collect(),
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

        AssignmentExpression(targets: Vec<Target<'i>>, op: AssignmentOperation, rhs: Box<Node<'i>>) {
            build = (pairs, token, state) {

                let lhs = unwrap_next!(pairs, token);
                let lhs = lhs.into_node(state).with_context(&token)?;
                let op = AssignmentOperation::from(unwrap_next!(pairs, token).as_rule());
                let rhs = Box::new(unwrap_node!(pairs, state, token)?);

                if let node_type!(Values::Reference(reference)) = lhs {
                    Ok(Self { targets: vec![reference.target], op, rhs, token }.into())
                } else if let node_type!(Collections::Array(target)) = lhs {
                    let targets = target.elements.into_iter().map(|e| {
                        if let node_type!(Values::Reference(target)) = e {
                            Ok(target.target)
                        } else {
                            oops!(ConstantValue, e.token().clone())
                        }
                    }).collect::<Result<Vec<_>, _>>().with_context(&token)?;

                    Ok(Self { targets, op, rhs, token }.into())
                } else {
                    return oops!(ConstantValue, token);
                }
            },
            eval = (this, state) {
                let rhs = this.rhs.evaluate(state).with_context(this.token())?;
                this.op.apply(state, &this.targets, rhs).with_context(this.token())
            },
            owned = (this) {
                Self::Owned {
                    targets: this.targets.into_iter().map(|t| t.into_owned()).collect(),
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

                    Operators:
                    - Arithmetic: `+=, -=, *=, /=, %=, **=`
                    - Bitwise: `&=, |=, ^=, <<=, >>=`
                    - Boolean: `&&=, ||=`
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
    use crate::{assert_expr, error::ErrorDetails, lav, match_expr_err};

    #[test]
    fn test_del() {
        // Identifiers
        assert_expr!("a = 2; del a", 2i64);
        match_expr_err!("del a", ErrorDetails::VariableName { .. });
        assert_expr!("a = 2; del a; would_err('a')", true);
        match_expr_err!("del e", ErrorDetails::ConstantValue { .. });

        // Destructuring
        assert_expr!("a = 1; b = 2; del [a, b]", vec![1i64, 2i64]);
        match_expr_err!("del [a, b]", ErrorDetails::VariableName { .. });
        match_expr_err!("a = 1; del [a, b]", ErrorDetails::VariableName { .. });
        assert_expr!("a = 1; b = 2; del [a, b]; would_err('a')", true);
        match_expr_err!("a = 1; del [a, 1]", ErrorDetails::ConstantValue { .. });

        // Indices
        assert_expr!("a = [1, 2]; del a[0]", 1i64);
        assert_expr!("a = [1, 2]; del a[]", 2i64);
        assert_expr!("a = {'test': 1}; del a['test']", 1i64);
        assert_expr!("a = {'test': 1}; del a['test']; would_err('test')", true);
        assert_expr!("a = {'test': [[[1]]]}; del a['test'][0][0][0]", 1i64);
        assert_expr!(
            "a = {'test': [[[1]]]}; del a['test'][0][0]; len(a['test'][0])",
            0i64
        );
    }

    #[test]
    fn test_assignment_ops() {
        assert_expr!("a=1; a+=1; a", 2i64);
        assert_expr!("b=1; b-=1; b", 0i64);
        assert_expr!("c=1; c*=2; c", 2i64);
        assert_expr!("d=4; d/=2; d", 2i64);
        assert_expr!("ee=4; ee%=2; ee", 0i64);
        assert_expr!("f=2; f**=3; f", 8i64);
        assert_expr!("g=2; g&=3; g", 2i64);
        assert_expr!("h=2; h|=3; h", 3i64);
        assert_expr!("i=2; i^=3; i", 1i64);
        assert_expr!("j=2; j<<=3; j", 16i64);
        assert_expr!("k=2; k>>=3; k", 0i64);
        assert_expr!("l=true; l&&=false; l", false);
        assert_expr!("m=true; m||=false; m", true);

        // destructuring
        assert_expr!("a = 1; b = 2; [a, b] += 1; [a, b]", vec![2i64, 3i64]);

        // indices are fine
        assert_expr!("a = [1, 2]; a[0] += 1; a", vec![2i64, 2i64.into()]);
    }

    lav!(test_assign_destructure r#"
        [a, b] = [1, [1,2]]
        assert_eq(a, 1)
        assert_eq(b, [1,2])

        a = 1; b = 2;
        [a, b] = [1, 1]
        assert_eq(a, 1)
        assert_eq(b, 1)
    "#);

    lav!(test_assign_destructure_error_toomany(Error) r#"
        [a, b] = [1, 2, 3]
    "#);

    lav!(test_assign_destructure_error_toofew(Error) r#"
        [a, b, c] = [1, 2]
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
