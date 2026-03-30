use crate::syntax::error::{Span, SyntaxError};
use crate::syntax::token::{Token, TokenKind};

pub struct Lexer<'a> {
    source: &'a str,
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source, bytes: source.as_bytes(), pos: 0 }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, SyntaxError> {
        let mut tokens = Vec::new();
        loop {
            self.skip_ws();
            if self.pos >= self.bytes.len() {
                tokens.push(Token { kind: TokenKind::Eof, span: Span::new(self.pos, self.pos) });
                break;
            }
            tokens.push(self.next_token()?);
        }
        Ok(tokens)
    }

    fn skip_ws(&mut self) {
        loop {
            while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_whitespace() { self.pos += 1; }
            if self.pos + 1 < self.bytes.len() && self.bytes[self.pos] == b'-' && self.bytes[self.pos + 1] == b'-' {
                while self.pos < self.bytes.len() && self.bytes[self.pos] != b'\n' { self.pos += 1; }
                continue;
            }
            break;
        }
    }

    fn peek(&self) -> u8 { if self.pos < self.bytes.len() { self.bytes[self.pos] } else { 0 } }
    fn peek_at(&self, off: usize) -> u8 { let i = self.pos + off; if i < self.bytes.len() { self.bytes[i] } else { 0 } }
    fn advance(&mut self) -> u8 { let b = self.bytes[self.pos]; self.pos += 1; b }

    fn next_token(&mut self) -> Result<Token, SyntaxError> {
        let start = self.pos;
        let b = self.peek();
        if b.is_ascii_digit() { return self.lex_number(start); }
        if b == b'"' { return self.lex_string(start); }
        if b.is_ascii_alphabetic() || b == b'_' { return self.lex_ident(start); }
        self.advance();
        let kind = match b {
            b'\\' => TokenKind::Backslash,
            b'(' => TokenKind::LParen, b')' => TokenKind::RParen,
            b'{' => TokenKind::LBrace, b'}' => TokenKind::RBrace,
            b'[' => TokenKind::LBracket, b']' => TokenKind::RBracket,
            b',' => TokenKind::Comma, b':' => TokenKind::Colon, b'.' => TokenKind::Dot,
            b'+' => TokenKind::Plus, b'*' => TokenKind::Star,
            b'/' => TokenKind::Slash, b'%' => TokenKind::Percent,
            b'-' => if self.peek() == b'>' { self.advance(); TokenKind::Arrow } else { TokenKind::Minus },
            b'=' => if self.peek() == b'=' { self.advance(); TokenKind::EqEq } else { TokenKind::Eq },
            b'<' => if self.peek() == b'=' { self.advance(); TokenKind::LtEq } else { TokenKind::LAngle },
            b'>' => if self.peek() == b'=' { self.advance(); TokenKind::GtEq } else { TokenKind::RAngle },
            b'!' => if self.peek() == b'=' { self.advance(); TokenKind::BangEq } else { TokenKind::Bang },
            b'&' => if self.peek() == b'&' { self.advance(); TokenKind::AmpAmp }
                     else { return Err(SyntaxError::new("expected '&&'", Span::new(start, self.pos))); },
            b'|' => if self.peek() == b'|' { self.advance(); TokenKind::PipePipe }
                     else if self.peek() == b'>' { self.advance(); TokenKind::PipeGt }
                     else { TokenKind::Pipe },
            _ => return Err(SyntaxError::new(format!("unexpected character: '{}'", b as char), Span::new(start, self.pos))),
        };
        Ok(Token { kind, span: Span::new(start, self.pos) })
    }

    fn lex_number(&mut self, start: usize) -> Result<Token, SyntaxError> {
        if self.peek() == b'0' && self.peek_at(1) == b'x' {
            self.advance(); self.advance();
            let hex_start = self.pos;
            while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_hexdigit() { self.pos += 1; }
            let hex = &self.source[hex_start..self.pos];
            if hex.len() >= 8 {
                return Ok(Token { kind: TokenKind::HexHash(hex.to_string()), span: Span::new(start, self.pos) });
            }
            let val = i64::from_str_radix(hex, 16).map_err(|e| SyntaxError::new(format!("invalid hex: {e}"), Span::new(start, self.pos)))?;
            return Ok(Token { kind: TokenKind::IntLit(val), span: Span::new(start, self.pos) });
        }
        while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_digit() { self.pos += 1; }
        if self.pos < self.bytes.len() && self.bytes[self.pos] == b'.' && self.pos + 1 < self.bytes.len() && self.bytes[self.pos + 1].is_ascii_digit() {
            self.pos += 1;
            while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_digit() { self.pos += 1; }
            let val: f64 = self.source[start..self.pos].parse().map_err(|e| SyntaxError::new(format!("invalid float: {e}"), Span::new(start, self.pos)))?;
            return Ok(Token { kind: TokenKind::FloatLit(val), span: Span::new(start, self.pos) });
        }
        let val: i64 = self.source[start..self.pos].parse().map_err(|e| SyntaxError::new(format!("invalid int: {e}"), Span::new(start, self.pos)))?;
        Ok(Token { kind: TokenKind::IntLit(val), span: Span::new(start, self.pos) })
    }

    fn lex_string(&mut self, start: usize) -> Result<Token, SyntaxError> {
        self.advance();
        let mut s = String::new();
        loop {
            if self.pos >= self.bytes.len() { return Err(SyntaxError::new("unterminated string", Span::new(start, self.pos))); }
            let b = self.advance();
            if b == b'"' { break; }
            if b == b'\\' {
                if self.pos >= self.bytes.len() { return Err(SyntaxError::new("unterminated escape", Span::new(start, self.pos))); }
                match self.advance() { b'n'=>s.push('\n'), b't'=>s.push('\t'), b'\\'=>s.push('\\'), b'"'=>s.push('"'), b'0'=>s.push('\0'),
                    esc => return Err(SyntaxError::new(format!("unknown escape: \\{}", esc as char), Span::new(self.pos-2, self.pos))), }
            } else { s.push(b as char); }
        }
        Ok(Token { kind: TokenKind::StringLit(s), span: Span::new(start, self.pos) })
    }

    fn lex_ident(&mut self, start: usize) -> Result<Token, SyntaxError> {
        while self.pos < self.bytes.len() && (self.bytes[self.pos].is_ascii_alphanumeric() || self.bytes[self.pos] == b'_') { self.pos += 1; }
        let text = &self.source[start..self.pos];
        let kind = match text {
            "let" => TokenKind::Let, "rec" => TokenKind::Rec, "in" => TokenKind::In,
            "val" => TokenKind::Val, "type" => TokenKind::Type,
            "import" => TokenKind::Import, "as" => TokenKind::As, "and" => TokenKind::And,
            "match" => TokenKind::Match, "with" => TokenKind::With, "when" => TokenKind::When,
            "if" => TokenKind::If, "then" => TokenKind::Then, "else" => TokenKind::Else,
            "forall" => TokenKind::Forall, "true" => TokenKind::True, "false" => TokenKind::False,
            "requires" => TokenKind::Requires, "ensures" => TokenKind::Ensures,
            "allow" => TokenKind::Allow, "deny" => TokenKind::Deny,
            "class" => TokenKind::Class, "instance" => TokenKind::Instance, "where" => TokenKind::Where,
            "_" => TokenKind::Underscore,
            _ => TokenKind::Ident(text.to_string()),
        };
        Ok(Token { kind, span: Span::new(start, self.pos) })
    }
}
