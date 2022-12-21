use std::fmt::{Debug, Display, Formatter};
use crate::source::Span;

#[derive(Clone, PartialEq)]
pub(crate) struct Token {
    pub(crate) tt: TokenType,
    pub(crate) loc: Span
}

impl Debug for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Token {{ tt: {:?}, loc: {}..{} }}", self.tt, self.loc.start, self.loc.end)
    }
}

#[allow(non_camel_case_types)]
/// `true` if the preceding character is also a particle with no spaces or any kind of separator.
/// Keep in mind that a lot of characters qualify as a particle so it is not always safe to assume a value.
pub(crate) type glued = bool;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TokenType {
    Particle(char, glued),
    Ident(String),
    Literal(Literal)
}

impl TokenType {
    pub(crate) fn at(self, loc: Span) -> Token{
        Token { tt: self, loc }
    }
}

#[derive(Clone, PartialEq)]
pub(crate) enum Literal {
    String(String),
    Char(char),
    Number(NumLit, Option<NumLitTy>),
    Bool(bool)
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum NumLit {
    Float(f64),
    Integer(u128)
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum NumLitTy {
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128,
    F32,
    F64,
}

impl Display for NumLitTy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            NumLitTy::U8 => "u8",
            NumLitTy::U16 => "u16",
            NumLitTy::U32 => "u32",
            NumLitTy::U64 => "u64",
            NumLitTy::U128 => "u128",
            NumLitTy::I8 => "i8",
            NumLitTy::I16 => "i16",
            NumLitTy::I32 => "i32",
            NumLitTy::I64 => "i64",
            NumLitTy::I128 => "i128",
            NumLitTy::F32 => "f32",
            NumLitTy::F64 => "f64",
        })
    }
}

impl Debug for Literal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Literal::String(s) => format!("String(\"{s}\")"),
            Literal::Char(c) => format!("Char('{c}')"),
            Literal::Number(NumLit::Integer(i), t) => format!("Integer({i}, {t:?})"),
            Literal::Number(NumLit::Float(f), t) => format!("Float({f}, {t:?})"),
            Literal::Bool(b) => format!("Bool({b})"),
        })
    }
}