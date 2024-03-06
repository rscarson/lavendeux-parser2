use std::{borrow::Cow, collections::HashSet};

use pest::error::ErrorVariant;

use crate::{error::ErrorDetails, pest::Rule, Error, Token};

/// Wraps a syntax error into an Error.
pub trait WrapSyntaxError<T, R> {
    /// Turns a pest error into an Error.
    fn wrap_syntax_error(self, input: &str) -> Result<T, Error>;
}
impl<T> WrapSyntaxError<T, Rule> for Result<T, pest::error::Error<Rule>> {
    fn wrap_syntax_error(self, input: &str) -> Result<T, Error> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => {
                let span = match e.location {
                    pest::error::InputLocation::Pos(pos) => pos..(input.len()),
                    pest::error::InputLocation::Span(span) => span.0..span.1,
                };
                let span = &input[span];

                let line = match e.line_col {
                    pest::error::LineColLocation::Pos((line, _)) => line,
                    pest::error::LineColLocation::Span((line, _), _) => line,
                };

                let token = crate::Token {
                    line,
                    rule: crate::Rule::SCRIPT,
                    input: Cow::Borrowed(span.split('\n').next().unwrap_or_default()),
                }
                .into_owned();

                let expected = if let ErrorVariant::ParsingError { positives, .. } = e.variant {
                    RuleCategory::collect(&positives)
                } else {
                    Vec::new()
                };

                oops!(Syntax { expected: expected }, token)
            }
        }
    }
}

/// Wrap a 3rd party error into an Error.
pub trait WrapExternalError<'i, T> {
    /// Adds a context [Token]
    fn with_context(self, context: &Token<'i>) -> Result<T, Error>;

    /// Adds a source [Error]
    fn with_source(self, source: Error) -> Result<T, Error>;

    /// Wraps the error without context or a source
    fn without_context(self) -> Result<T, Error>;
}

impl<'i, T, E> WrapExternalError<'i, T> for Result<T, E>
where
    E: Into<Error>,
{
    fn with_context(self, context: &Token<'i>) -> Result<T, Error> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(e.into().with_context(context.clone())),
        }
    }

    fn with_source(self, source: Error) -> Result<T, Error> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(e.into().with_source(source)),
        }
    }

    fn without_context(self) -> Result<T, Error> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(e.into().without_context()),
        }
    }
}

/// Wrap an `Option<T>` into a `Result<T, Error>`
pub trait WrapOption<'i, T> {
    /// Turns an `Option<T>` into a `Result<T, Error>`
    fn or_error(self, error: ErrorDetails) -> Result<T, Error>;
}
impl<'i, T> WrapOption<'i, T> for Option<T> {
    fn or_error(self, error: ErrorDetails) -> Result<T, Error> {
        match self {
            Some(v) => Ok(v),
            None => Err(Error {
                details: error,
                context: None,
                source: None,
            }),
        }
    }
}

/// Describes the category of a rule.
/// Used for error messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum RuleCategory {
    Block,
    FunctionAssignment,
    Expression,
    Literal,
    Identifier,
    Operator,
    Array,
    Object,
    Symbol(&'static str),

    IntSizeSuffix,
    CurrencySymbol,

    Hidden,
}

impl RuleCategory {
    /// Collects all rule categories from a list of rules.
    pub fn collect(rules: &[Rule]) -> Vec<Self> {
        let set = rules
            .iter()
            .map(|r| RuleCategory::from(*r))
            .filter(|e| e != &RuleCategory::Hidden)
            .collect::<HashSet<_>>();
        set.into_iter().collect()
    }

    /// Formats a set of rule categories into a string.
    pub fn fmt(cats: &[Self]) -> String {
        let mut cats = cats.iter().map(|c| c.to_string()).collect::<Vec<_>>();
        cats.sort();
        cats.join(", ")
    }
}

impl std::fmt::Display for RuleCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Block => write!(f, "block"),
            Self::FunctionAssignment => write!(f, "function assignment"),
            Self::Expression => write!(f, "expression"),
            Self::Literal => write!(f, "value"),
            Self::Identifier => write!(f, "identifier"),
            Self::Operator => write!(f, "operator"),
            Self::Array => write!(f, "array"),
            Self::Object => write!(f, "object"),
            Self::Symbol(s) => write!(f, "`{}`", s),

            Self::IntSizeSuffix => write!(f, "integer suffix"),
            Self::CurrencySymbol => write!(f, "currency symbol"),

            Self::Hidden => write!(f, ""),
        }
    }
}

impl From<Rule> for RuleCategory {
    fn from(rule: Rule) -> Self {
        match rule {
            Rule::BLOCK => Self::Block,

            Rule::FUNCTION_ASSIGNMENT_STATEMENT => Self::FunctionAssignment,

            Rule::SKIP_KEYWORD
            | Rule::BREAK_KEYWORD
            | Rule::RETURN_EXPRESSION
            | Rule::SWITCH_EXPRESSION
            | Rule::FOR_LOOP_EXPRESSION
            | Rule::IF_EXPRESSION
            | Rule::EXPR => Self::Expression,

            Rule::symbol_questionmark => Self::Symbol("?"),
            Rule::symbol_colon => Self::Symbol(":"),
            Rule::symbol_comma => Self::Symbol(","),

            Rule::symbol_opencurly => Self::Symbol("{"),
            Rule::symbol_closecurly => Self::Symbol("}"),
            Rule::array_symbol_opensquare => Self::Symbol("["),
            Rule::symbol_opensquare => Self::Symbol("["),
            Rule::symbol_closesquare => Self::Symbol("]"),
            Rule::symbol_openround => Self::Symbol("("),
            Rule::symbol_closeround => Self::Symbol(")"),
            Rule::symbol_arrow => Self::Symbol("=>"),
            Rule::symbol_at => Self::Symbol("@"),
            Rule::symbol_eq => Self::Symbol("="),

            Rule::POSTFIX_EMPTYINDEX
            | Rule::POSTFIX_DECORATE
            | Rule::POSTFIX_INDEX
            | Rule::POSTFIX_CALL
            | Rule::POSTFIX_INC
            | Rule::POSTFIX_DEC
            | Rule::PREFIX_DEL
            | Rule::PREFIX_BOOL_NOT
            | Rule::PREFIX_BIT_NOT
            | Rule::PREFIX_NEG
            | Rule::PREFIX_INC
            | Rule::PREFIX_DEC
            | Rule::OP_ASSIGN_ADD
            | Rule::OP_ASSIGN_SUB
            | Rule::OP_ASSIGN_POW
            | Rule::OP_ASSIGN_MUL
            | Rule::OP_ASSIGN_DIV
            | Rule::OP_ASSIGN_MOD
            | Rule::OP_BASSIGN_AND
            | Rule::OP_BASSIGN_OR
            | Rule::OP_ASSIGN_OR
            | Rule::OP_ASSIGN_AND
            | Rule::OP_ASSIGN_XOR
            | Rule::OP_ASSIGN_SL
            | Rule::OP_ASSIGN_SR
            | Rule::OP_ASSIGN
            | Rule::OP_BOOL_OR
            | Rule::OP_BOOL_AND
            | Rule::OP_BOOL_EQ
            | Rule::OP_BOOL_NE
            | Rule::OP_BOOL_LE
            | Rule::OP_BOOL_GE
            | Rule::OP_BOOL_LT
            | Rule::OP_BOOL_GT
            | Rule::OP_BIT_OR
            | Rule::OP_BIT_XOR
            | Rule::OP_BIT_AND
            | Rule::OP_BIT_SL
            | Rule::OP_BIT_SR
            | Rule::OP_ADD
            | Rule::OP_SUB
            | Rule::OP_MUL
            | Rule::OP_DIV
            | Rule::OP_MOD
            | Rule::OP_POW
            | Rule::match_infix_op
            | Rule::OP_MATCH_CONTAINS
            | Rule::OP_MATCH_MATCHES
            | Rule::OP_MATCH_IS
            | Rule::OP_MATCH_STARTSWITH
            | Rule::OP_MATCH_ENDSWITH
            | Rule::OP_RANGE
            | Rule::OP_CAST
            | Rule::OP_TERNARY
            | Rule::bool_infix_op
            | Rule::bitwise_infix_op
            | Rule::arithmetic_infix_op
            | Rule::assignment_infix_op
            | Rule::prefix_op
            | Rule::prefix_arith
            | Rule::KEYWORD_EXPRESSION
            | Rule::del_keyword
            | Rule::postfix_operation
            | Rule::postfixcall_args
            | Rule::postfix_arith
            | Rule::POSTFIX_NORMALMODE
            | Rule::POSTFIX_OBJECTMODE
            | Rule::infix_op => Self::Operator,

            Rule::ARRAY_TERM => Self::Array,
            Rule::OBJECT_TERM => Self::Object,

            Rule::TERM
            | Rule::ATOMIC_VALUE
            | Rule::dec_literal
            | Rule::hex_literal
            | Rule::bin_literal
            | Rule::oct_literal
            | Rule::fixed_literal
            | Rule::currency_literal
            | Rule::sci_literal
            | Rule::float_literal
            | Rule::const_literal
            | Rule::int_literal
            | Rule::bool_literal
            | Rule::string_literal
            | Rule::regex_literal => Self::Literal,

            Rule::identifier => Self::Identifier,

            Rule::int_sep
            | Rule::sized_literal_suffix
            | Rule::intsize_u8
            | Rule::intsize_i8
            | Rule::intsize_u16
            | Rule::intsize_i16
            | Rule::intsize_u32
            | Rule::intsize_i32
            | Rule::intsize_u64
            | Rule::intsize_i64 => Self::IntSizeSuffix,

            Rule::currency_suffix | Rule::currency_symbol => Self::CurrencySymbol,

            Rule::object_keyvalue_pair
            | Rule::for_conditional
            | Rule::switch_case
            | Rule::if_block
            | Rule::function_typespec
            | Rule::function_argument
            | Rule::function_name
            | Rule::SCRIPT
            | Rule::reserved_words
            | Rule::ERROR
            | Rule::UNTERMINATED_BLOCK_COMMENT
            | Rule::UNTERMINATED_STRING_LITERAL
            | Rule::UNCLOSED_BRACKET
            | Rule::UNCLOSED_BRACE
            | Rule::UNCLOSED_PAREN
            | Rule::MISSING_LINEBREAK
            | Rule::EOI
            | Rule::LINE
            | Rule::INLINE_COMMENT
            | Rule::BLOCK_COMMENT
            | Rule::COMMENT
            | Rule::WHITESPACE
            | Rule::EOL
            | Rule::STATEMENT => Self::Hidden,
        }
    }
}
