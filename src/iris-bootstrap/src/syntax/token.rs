use crate::syntax::error::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    IntLit(i64),
    FloatLit(f64),
    StringLit(String),
    HexHash(String),
    Ident(String),

    // Keywords
    Let, Rec, In, Val, Type, Import, As, And,
    Match, With, If, Then, Else, When, Where,
    Forall, True, False,
    Requires, Ensures,
    Allow, Deny, Class, Instance,

    // Punctuation
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,
    LAngle, RAngle, Comma, Colon, Dot,
    Arrow, Pipe, Eq, Underscore, Backslash,

    // Operators
    Plus, Minus, Star, Slash, Percent,
    EqEq, BangEq, LtEq, GtEq,
    AmpAmp, PipePipe, Bang, PipeGt,

    Eof,
}

impl TokenKind {
    pub fn is_eof(&self) -> bool { matches!(self, TokenKind::Eof) }
}
