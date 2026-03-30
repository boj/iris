use crate::syntax::ast::*;
use crate::syntax::error::{Span, SyntaxError};
use crate::syntax::token::{Token, TokenKind};

/// Maximum recursive descent depth. Prevents stack-overflow on deeply-nested
/// or adversarially crafted input.
const MAX_PARSE_DEPTH: usize = 256;

pub struct Parser { tokens: Vec<Token>, pos: usize, depth: usize }

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self { Self { tokens, pos: 0, depth: 0 } }
    fn peek(&self) -> &TokenKind { &self.tokens[self.pos].kind }
    fn span(&self) -> Span { self.tokens[self.pos].span }
    fn advance(&mut self) -> &Token { let t = &self.tokens[self.pos]; if !t.kind.is_eof() { self.pos += 1; } t }
    fn expect(&mut self, expected: &TokenKind) -> Result<Span, SyntaxError> {
        if std::mem::discriminant(self.peek()) == std::mem::discriminant(expected) { let s = self.span(); self.advance(); Ok(s) }
        else { Err(SyntaxError::new(format!("expected {:?}, found {:?}", expected, self.peek()), self.span())) }
    }
    fn expect_ident(&mut self) -> Result<(String, Span), SyntaxError> {
        match self.peek().clone() { TokenKind::Ident(s) => { let sp = self.span(); self.advance(); Ok((s, sp)) }
            _ => Err(SyntaxError::new(format!("expected identifier, found {:?}", self.peek()), self.span())) }
    }
    fn at(&self, kind: &TokenKind) -> bool { std::mem::discriminant(self.peek()) == std::mem::discriminant(kind) }
    fn eat(&mut self, kind: &TokenKind) -> bool { if self.at(kind) { self.advance(); true } else { false } }

    pub fn parse_module(&mut self) -> Result<Module, SyntaxError> {
        // Parse optional capability declarations before items.
        let capabilities = self.parse_capability_decls()?;
        let mut items = Vec::new();
        while !self.peek().is_eof() { items.push(self.parse_item()?); }
        Ok(Module { items, capabilities })
    }
    fn parse_item(&mut self) -> Result<Item, SyntaxError> {
        match self.peek() {
            TokenKind::Let => self.parse_let_or_mutual_rec(),
            TokenKind::Type => self.parse_type_decl().map(Item::TypeDecl),
            TokenKind::Import => self.parse_import().map(Item::Import),
            TokenKind::Class => self.parse_class_decl().map(Item::ClassDecl),
            TokenKind::Instance => self.parse_instance_decl().map(Item::InstanceDecl),
            _ => Err(SyntaxError::new(format!("expected 'let', 'type', 'class', 'instance', or 'import', found {:?}", self.peek()), self.span())),
        }
    }

    /// Parse a `let` declaration. If this is `let rec ... and ...`, collect all
    /// mutually recursive declarations into a `MutualRecGroup`.
    fn parse_let_or_mutual_rec(&mut self) -> Result<Item, SyntaxError> {
        let first = self.parse_let_decl()?;
        if first.recursive && self.at(&TokenKind::And) {
            let mut group = vec![first];
            while self.eat(&TokenKind::And) {
                let start = self.span();
                let (name, _) = self.expect_ident()?;
                let mut params = Vec::new();
                while let TokenKind::Ident(_) = self.peek() {
                    let (p, _) = self.expect_ident()?;
                    params.push(p);
                }
                let ret_type = if self.eat(&TokenKind::Colon) { Some(self.parse_type()?) } else { None };
                let cost = if self.eat(&TokenKind::LBracket) {
                    let (kw, _) = self.expect_ident()?;
                    if kw != "cost" {
                        return Err(SyntaxError::new(format!("expected 'cost', found '{kw}'"), self.span()));
                    }
                    self.expect(&TokenKind::Colon)?;
                    let c = self.parse_cost()?;
                    self.expect(&TokenKind::RBracket)?;
                    Some(c)
                } else {
                    None
                };
                let mut requires = Vec::new();
                while self.eat(&TokenKind::Requires) { requires.push(self.parse_expr()?); }
                let mut ensures = Vec::new();
                while self.eat(&TokenKind::Ensures) { ensures.push(self.parse_expr()?); }
                self.expect(&TokenKind::Eq)?;
                let body = self.parse_expr()?;
                let span = start.merge(self.tokens[self.pos.saturating_sub(1)].span);
                group.push(LetDecl {
                    name, params, ret_type, cost, requires, ensures, body,
                    recursive: true, span,
                });
            }
            Ok(Item::MutualRecGroup(group))
        } else {
            Ok(Item::LetDecl(first))
        }
    }

    /// Parse optional `allow [...]` and `deny [...]` declarations.
    ///
    /// ```iris
    /// allow [FileRead, FileWrite "/tmp/*"]
    /// deny [TcpConnect, ThreadSpawn, MmapExec]
    /// ```
    fn parse_capability_decls(&mut self) -> Result<Option<CapabilityDecl>, SyntaxError> {
        let mut allow = Vec::new();
        let mut deny = Vec::new();
        let start = self.span();
        let mut found = false;

        while matches!(self.peek(), TokenKind::Allow | TokenKind::Deny) {
            found = true;
            let is_allow = matches!(self.peek(), TokenKind::Allow);
            self.advance();
            self.expect(&TokenKind::LBracket)?;

            let mut entries = Vec::new();
            loop {
                if self.at(&TokenKind::RBracket) { break; }
                let entry_span = self.span();
                let (name, _) = self.expect_ident()?;
                let argument = if let TokenKind::StringLit(_) = self.peek() {
                    match self.peek().clone() {
                        TokenKind::StringLit(s) => { self.advance(); Some(s) }
                        _ => None,
                    }
                } else {
                    None
                };
                entries.push(CapEntry { effect_name: name, argument, span: entry_span });
                if !self.eat(&TokenKind::Comma) { break; }
            }
            self.expect(&TokenKind::RBracket)?;

            if is_allow { allow.extend(entries); }
            else { deny.extend(entries); }
        }

        if found {
            let end = self.tokens[self.pos.saturating_sub(1)].span;
            Ok(Some(CapabilityDecl { allow, deny, span: start.merge(end) }))
        } else {
            Ok(None)
        }
    }

    fn parse_let_decl(&mut self) -> Result<LetDecl, SyntaxError> {
        let start = self.span(); self.expect(&TokenKind::Let)?;
        // Consume optional `rec` keyword — makes the binding recursive (fixpoint).
        let recursive = self.eat(&TokenKind::Rec);
        let (name, _) = self.expect_ident()?;
        let mut params = Vec::new();
        while let TokenKind::Ident(_) = self.peek() { let (p, _) = self.expect_ident()?; params.push(p); }
        let ret_type = if self.eat(&TokenKind::Colon) { Some(self.parse_type()?) } else { None };
        let cost = if self.eat(&TokenKind::LBracket) {
            let (kw, _) = self.expect_ident()?;
            if kw != "cost" { return Err(SyntaxError::new(format!("expected 'cost', found '{kw}'"), self.span())); }
            self.expect(&TokenKind::Colon)?; let c = self.parse_cost()?; self.expect(&TokenKind::RBracket)?; Some(c)
        } else { None };
        let mut requires = Vec::new();
        while self.eat(&TokenKind::Requires) { requires.push(self.parse_expr()?); }
        let mut ensures = Vec::new();
        while self.eat(&TokenKind::Ensures) { ensures.push(self.parse_expr()?); }
        self.expect(&TokenKind::Eq)?;
        let body = self.parse_expr()?;
        Ok(LetDecl { name, params, ret_type, cost, requires, ensures, body, recursive, span: start.merge(self.tokens[self.pos.saturating_sub(1)].span) })
    }
    fn parse_type_decl(&mut self) -> Result<TypeDecl, SyntaxError> {
        let start = self.span(); self.expect(&TokenKind::Type)?;
        let (name, _) = self.expect_ident()?;
        let type_params = if self.eat(&TokenKind::LAngle) {
            let mut v = Vec::new(); loop { let (t,_) = self.expect_ident()?; v.push(t); if !self.eat(&TokenKind::Comma) { break; } }
            self.expect(&TokenKind::RAngle)?; v } else { vec![] };
        self.expect(&TokenKind::Eq)?;
        // Check for record type: `= { field: Type, ... }`
        let mut def = if self.at(&TokenKind::LBrace) {
            self.parse_record_type(start)?
        // Check for sum type: `= Variant(T) | Variant | ...`
        } else if self.is_variant_start() {
            self.parse_sum_type(start)?
        } else {
            self.parse_type()?
        };
        // Record composition: `Type / Type / ...`
        while self.eat(&TokenKind::Slash) {
            let rhs = if self.at(&TokenKind::LBrace) {
                self.parse_record_type(start)?
            } else {
                self.parse_type()?
            };
            let sp = def.span().merge(rhs.span());
            def = TypeExpr::RecordMerge(Box::new(def), Box::new(rhs), sp);
        }
        Ok(TypeDecl { name, type_params, def, span: start.merge(self.tokens[self.pos.saturating_sub(1)].span) })
    }

    /// Check if current position looks like the start of a sum-type variant list.
    /// Returns true if we see `Ident` followed by `|` or `(` or at EOF/newline,
    /// AND the identifier starts with an uppercase letter (constructor convention).
    fn is_variant_start(&self) -> bool {
        if let TokenKind::Ident(name) = self.peek() {
            if name.chars().next().map_or(false, |c| c.is_uppercase()) {
                // Peek ahead: after ident, should see `|`, `(`, or end of declaration
                if self.pos + 1 < self.tokens.len() {
                    let next = &self.tokens[self.pos + 1].kind;
                    return matches!(next,
                        TokenKind::Pipe | TokenKind::LParen
                        | TokenKind::Let | TokenKind::Type | TokenKind::Eof
                    );
                }
                return true;
            }
        }
        false
    }

    /// Parse a sum type: `Variant(Type) | Variant | Variant(Type)`
    fn parse_sum_type(&mut self, start: Span) -> Result<TypeExpr, SyntaxError> {
        let mut variants = Vec::new();
        loop {
            let (vname, _) = self.expect_ident()?;
            let payload = if self.eat(&TokenKind::LParen) {
                let pstart = self.tokens[self.pos.saturating_sub(1)].span;
                let first = self.parse_type()?;
                if self.eat(&TokenKind::Comma) {
                    // Multi-arity: Variant(T1, T2, ...) -> wrap as Tuple type
                    let mut types = vec![first];
                    types.push(self.parse_type()?);
                    while self.eat(&TokenKind::Comma) {
                        types.push(self.parse_type()?);
                    }
                    let pend = self.expect(&TokenKind::RParen)?;
                    Some(Box::new(TypeExpr::Tuple(types, pstart.merge(pend))))
                } else {
                    self.expect(&TokenKind::RParen)?;
                    Some(Box::new(first))
                }
            } else {
                None
            };
            variants.push((vname, payload));
            if !self.eat(&TokenKind::Pipe) { break; }
        }
        let end = self.tokens[self.pos.saturating_sub(1)].span;
        Ok(TypeExpr::Sum(variants, start.merge(end)))
    }

    /// Parse a record type: `{ field: Type, field: Type }`
    fn parse_record_type(&mut self, start: Span) -> Result<TypeExpr, SyntaxError> {
        self.expect(&TokenKind::LBrace)?;
        let mut fields = Vec::new();
        if !self.at(&TokenKind::RBrace) {
            loop {
                let (fname, _) = self.expect_ident()?;
                self.expect(&TokenKind::Colon)?;
                let ftype = self.parse_type()?;
                fields.push((fname, Box::new(ftype)));
                if !self.eat(&TokenKind::Comma) { break; }
                if self.at(&TokenKind::RBrace) { break; } // trailing comma
            }
        }
        let end = self.expect(&TokenKind::RBrace)?;
        Ok(TypeExpr::Record(fields, start.merge(end)))
    }

    fn parse_import(&mut self) -> Result<ImportDecl, SyntaxError> {
        let start = self.span(); self.expect(&TokenKind::Import)?;
        let source = match self.peek().clone() {
            TokenKind::HexHash(h) => { self.advance(); ImportSource::Hash(h) }
            TokenKind::StringLit(p) => { self.advance(); ImportSource::Path(p) }
            _ => return Err(SyntaxError::new("expected hex hash or \"path\"", self.span()))
        };
        self.expect(&TokenKind::As)?; let (name, end) = self.expect_ident()?;
        Ok(ImportDecl { source, name, span: start.merge(end) })
    }

    /// Parse `class Eq<A> [requires Superclass<A>] where method : Type [= default] ...`
    fn parse_class_decl(&mut self) -> Result<ClassDecl, SyntaxError> {
        let start = self.span();
        self.expect(&TokenKind::Class)?;
        let (name, _) = self.expect_ident()?;
        self.expect(&TokenKind::LAngle)?;
        let (type_param, _) = self.expect_ident()?;
        self.expect(&TokenKind::RAngle)?;

        // Optional superclasses: `requires Eq<A>`
        let mut superclasses = Vec::new();
        while self.eat(&TokenKind::Requires) {
            let (sc_name, _) = self.expect_ident()?;
            // Skip the <A> part if present
            if self.eat(&TokenKind::LAngle) {
                let _ = self.expect_ident()?;
                self.expect(&TokenKind::RAngle)?;
            }
            superclasses.push(sc_name);
        }

        self.expect(&TokenKind::Where)?;

        // Parse method declarations: name : Type [= default_impl]
        let mut methods = Vec::new();
        while !self.at_top_level_start() && !self.peek().is_eof() {
            let ms = self.span();
            let (mname, _) = self.expect_ident()?;
            self.expect(&TokenKind::Colon)?;
            let type_sig = self.parse_type()?;
            let default_impl = if self.eat(&TokenKind::Eq) {
                Some(self.parse_binding_expr()?)
            } else {
                None
            };
            let mend = self.tokens[self.pos.saturating_sub(1)].span;
            methods.push(MethodDecl { name: mname, type_sig, default_impl, span: ms.merge(mend) });
        }

        let end = self.tokens[self.pos.saturating_sub(1)].span;
        Ok(ClassDecl { name, type_param, superclasses, methods, span: start.merge(end) })
    }

    /// Parse `instance Eq<Int> where method = impl ...`
    fn parse_instance_decl(&mut self) -> Result<InstanceDecl, SyntaxError> {
        let start = self.span();
        self.expect(&TokenKind::Instance)?;
        let (class_name, _) = self.expect_ident()?;
        self.expect(&TokenKind::LAngle)?;
        let type_arg = self.parse_type()?;
        self.expect(&TokenKind::RAngle)?;
        self.expect(&TokenKind::Where)?;

        // Parse method implementations: name = expr
        let mut methods = Vec::new();
        while !self.at_top_level_start() && !self.peek().is_eof() {
            let (mname, _) = self.expect_ident()?;
            self.expect(&TokenKind::Eq)?;
            let body = self.parse_binding_expr()?;
            methods.push((mname, body));
        }

        let end = self.tokens[self.pos.saturating_sub(1)].span;
        Ok(InstanceDecl { class_name, type_arg, methods, span: start.merge(end) })
    }

    /// Check if the current token starts a new top-level item.
    fn at_top_level_start(&self) -> bool {
        matches!(self.peek(),
            TokenKind::Let | TokenKind::Type | TokenKind::Import |
            TokenKind::Class | TokenKind::Instance)
    }

    fn parse_type(&mut self) -> Result<TypeExpr, SyntaxError> {
        if self.at(&TokenKind::Forall) {
            let s = self.span(); self.advance();
            let (v,_) = self.expect_ident()?;
            self.expect(&TokenKind::Dot)?;
            let i = self.parse_type()?;
            let sp = s.merge(i.span());
            return Ok(TypeExpr::ForAll(v, Box::new(i), sp));
        }
        let base = self.parse_type_atom()?;
        if self.eat(&TokenKind::Arrow) {
            let r = self.parse_type()?;
            let sp = base.span().merge(r.span());
            return Ok(TypeExpr::Arrow(Box::new(base), Box::new(r), sp));
        }
        Ok(base)
    }
    fn parse_type_atom(&mut self) -> Result<TypeExpr, SyntaxError> {
        match self.peek().clone() {
            TokenKind::LParen => { let s = self.span(); self.advance();
                if self.eat(&TokenKind::RParen) { return Ok(TypeExpr::Unit(s.merge(self.tokens[self.pos-1].span))); }
                let first = self.parse_type()?;
                if self.eat(&TokenKind::Comma) { let mut v = vec![first]; loop { v.push(self.parse_type()?); if !self.eat(&TokenKind::Comma) { break; } }
                    let end = self.expect(&TokenKind::RParen)?; Ok(TypeExpr::Tuple(v, s.merge(end)))
                } else { self.expect(&TokenKind::RParen)?; Ok(first) } }
            TokenKind::LBrace => { let s = self.span(); self.advance(); let (v,_) = self.expect_ident()?;
                self.expect(&TokenKind::Colon)?; let b = self.parse_type()?; self.expect(&TokenKind::Pipe)?;
                let p = self.parse_expr()?; let end = self.expect(&TokenKind::RBrace)?;
                Ok(TypeExpr::Refined(v, Box::new(b), Box::new(p), s.merge(end))) }
            TokenKind::Ident(name) => { let s = self.span(); self.advance();
                if self.eat(&TokenKind::LAngle) { let mut args = Vec::new();
                    loop { args.push(self.parse_type()?); if !self.eat(&TokenKind::Comma) { break; } }
                    let end = self.expect(&TokenKind::RAngle)?; Ok(TypeExpr::App(name, args, s.merge(end)))
                } else { Ok(TypeExpr::Named(name, s)) } }
            _ => Err(SyntaxError::new(format!("expected type, found {:?}", self.peek()), self.span())),
        }
    }
    fn parse_cost(&mut self) -> Result<CostExpr, SyntaxError> {
        let (n, _) = self.expect_ident()?;
        match n.as_str() {
            "Unknown" => Ok(CostExpr::Unknown), "Zero" => Ok(CostExpr::Zero),
            "Const" => { self.expect(&TokenKind::LParen)?; let v = self.expect_int()?; self.expect(&TokenKind::RParen)?; Ok(CostExpr::Constant(v as u64)) }
            "Linear" => { self.expect(&TokenKind::LParen)?; let (v,_) = self.expect_ident()?; self.expect(&TokenKind::RParen)?; Ok(CostExpr::Linear(v)) }
            "NLogN" => { self.expect(&TokenKind::LParen)?; let (v,_) = self.expect_ident()?; self.expect(&TokenKind::RParen)?; Ok(CostExpr::NLogN(v)) }
            _ => Err(SyntaxError::new(format!("unknown cost: '{n}'"), self.span())),
        }
    }
    fn expect_int(&mut self) -> Result<i64, SyntaxError> {
        match self.peek().clone() { TokenKind::IntLit(v) => { self.advance(); Ok(v) }
            _ => Err(SyntaxError::new(format!("expected integer, found {:?}", self.peek()), self.span())) }
    }

    // -- Expressions: precedence climbing --
    pub fn parse_expr(&mut self) -> Result<Expr, SyntaxError> {
        self.depth += 1;
        if self.depth > MAX_PARSE_DEPTH {
            self.depth -= 1;
            return Err(SyntaxError::new(
                format!("parse depth exceeded maximum of {}", MAX_PARSE_DEPTH),
                self.span(),
            ));
        }
        let result = self.parse_binding_expr();
        self.depth -= 1;
        result
    }

    fn parse_binding_expr(&mut self) -> Result<Expr, SyntaxError> {
        if self.at(&TokenKind::Let) {
            let s = self.span(); self.advance();
            // Consume optional `rec` — produces a recursive let-expression.
            let is_rec = self.eat(&TokenKind::Rec);
            let (n,_) = self.expect_ident()?; self.expect(&TokenKind::Eq)?;
            let v = self.parse_expr()?; self.expect(&TokenKind::In)?;
            let b = self.parse_expr()?; let sp = s.merge(b.span());
            if is_rec {
                return Ok(Expr::LetRec(n, Box::new(v), Box::new(b), sp));
            }
            return Ok(Expr::Let(n, Box::new(v), Box::new(b), sp));
        }
        if self.at(&TokenKind::If) {
            let s = self.span(); self.advance();
            let c = self.parse_expr()?; self.expect(&TokenKind::Then)?;
            let t = self.parse_expr()?; self.expect(&TokenKind::Else)?;
            let e = self.parse_expr()?; let sp = s.merge(e.span());
            return Ok(Expr::If(Box::new(c), Box::new(t), Box::new(e), sp));
        }
        if self.at(&TokenKind::Match) { return self.parse_match_expr(); }
        if self.at(&TokenKind::Backslash) { return self.parse_lambda(); }
        self.parse_pipe_expr()
    }

    fn parse_lambda(&mut self) -> Result<Expr, SyntaxError> {
        let s = self.span(); self.expect(&TokenKind::Backslash)?;
        let mut ps = Vec::new();
        while let TokenKind::Ident(_) = self.peek() { let (p,_) = self.expect_ident()?; ps.push(p); }
        if ps.is_empty() { return Err(SyntaxError::new("lambda needs parameters", self.span())); }
        self.expect(&TokenKind::Arrow)?;
        let body = self.parse_expr()?; let sp = s.merge(body.span());
        Ok(Expr::Lambda(ps, Box::new(body), sp))
    }

    fn parse_match_expr(&mut self) -> Result<Expr, SyntaxError> {
        let s = self.span(); self.expect(&TokenKind::Match)?;
        let scr = self.parse_pipe_expr()?; self.expect(&TokenKind::With)?;
        let mut arms = Vec::new();
        while self.eat(&TokenKind::Pipe) {
            let ps = self.span(); let pat = self.parse_pattern()?;
            // Optional guard clause: `when <expr>`
            let guard = if self.eat(&TokenKind::When) {
                Some(self.parse_pipe_expr()?)
            } else {
                None
            };
            self.expect(&TokenKind::Arrow)?; let body = self.parse_binding_expr()?;
            let arm_sp = ps.merge(self.tokens[self.pos.saturating_sub(1)].span);
            arms.push(MatchArm { pattern: pat, guard, body, span: arm_sp });
        }
        if arms.is_empty() { return Err(SyntaxError::new("match needs arms", self.span())); }
        let end_sp = arms.last().unwrap().span;
        let sp = s.merge(end_sp);
        Ok(Expr::Match(Box::new(scr), arms, sp))
    }
    fn parse_pattern(&mut self) -> Result<Pattern, SyntaxError> {
        let s = self.span();
        match self.peek().clone() {
            TokenKind::Underscore => { self.advance(); Ok(Pattern::Wildcard(s)) }
            TokenKind::True => { self.advance(); Ok(Pattern::BoolLit(true, s)) }
            TokenKind::False => { self.advance(); Ok(Pattern::BoolLit(false, s)) }
            TokenKind::IntLit(v) => { self.advance(); Ok(Pattern::IntLit(v, s)) }
            TokenKind::LParen => {
                // Tuple pattern: (a, b, c)
                self.advance();
                let mut pats = Vec::new();
                if !matches!(self.peek(), TokenKind::RParen) {
                    pats.push(self.parse_pattern()?);
                    while self.eat(&TokenKind::Comma) {
                        pats.push(self.parse_pattern()?);
                    }
                }
                let end = self.expect(&TokenKind::RParen)?;
                Ok(Pattern::Tuple(pats, s.merge(end)))
            }
            TokenKind::Ident(n) => {
                self.advance();
                // Uppercase ident with `(` -> constructor pattern: `Some(x)`
                // Uppercase ident without `(` -> bare constructor: `None`
                if n.chars().next().map_or(false, |c| c.is_uppercase()) {
                    if self.eat(&TokenKind::LParen) {
                        let inner = self.parse_pattern()?;
                        let end = self.expect(&TokenKind::RParen)?;
                        Ok(Pattern::Constructor(n, Some(Box::new(inner)), s.merge(end)))
                    } else {
                        Ok(Pattern::Constructor(n, None, s))
                    }
                } else {
                    Ok(Pattern::Ident(n, s))
                }
            }
            _ => Err(SyntaxError::new(format!("expected pattern, found {:?}", self.peek()), s)),
        }
    }

    fn parse_pipe_expr(&mut self) -> Result<Expr, SyntaxError> {
        let mut e = self.parse_or_expr()?;
        while self.eat(&TokenKind::PipeGt) {
            let r = self.parse_or_expr()?; let sp = e.span().merge(r.span());
            e = Expr::Pipe(Box::new(e), Box::new(r), sp);
        }
        Ok(e)
    }
    fn parse_or_expr(&mut self) -> Result<Expr, SyntaxError> {
        let mut e = self.parse_and_expr()?;
        while self.eat(&TokenKind::PipePipe) { let r = self.parse_and_expr()?; let s = e.span().merge(r.span()); e = Expr::BinOp(Box::new(e), BinOp::Or, Box::new(r), s); }
        Ok(e)
    }
    fn parse_and_expr(&mut self) -> Result<Expr, SyntaxError> {
        let mut e = self.parse_cmp_expr()?;
        while self.eat(&TokenKind::AmpAmp) { let r = self.parse_cmp_expr()?; let s = e.span().merge(r.span()); e = Expr::BinOp(Box::new(e), BinOp::And, Box::new(r), s); }
        Ok(e)
    }
    fn parse_cmp_expr(&mut self) -> Result<Expr, SyntaxError> {
        let l = self.parse_add_expr()?;
        let op = match self.peek() { TokenKind::EqEq=>Some(BinOp::Eq), TokenKind::BangEq=>Some(BinOp::Ne),
            TokenKind::LAngle=>Some(BinOp::Lt), TokenKind::RAngle=>Some(BinOp::Gt),
            TokenKind::LtEq=>Some(BinOp::Le), TokenKind::GtEq=>Some(BinOp::Ge), _=>None };
        if let Some(op) = op { self.advance(); let r = self.parse_add_expr()?; let sp = l.span().merge(r.span()); Ok(Expr::BinOp(Box::new(l), op, Box::new(r), sp)) }
        else { Ok(l) }
    }
    fn parse_add_expr(&mut self) -> Result<Expr, SyntaxError> {
        let mut e = self.parse_mul_expr()?;
        loop { match self.peek() {
            TokenKind::Plus => { self.advance(); let r = self.parse_mul_expr()?; let s = e.span().merge(r.span()); e = Expr::BinOp(Box::new(e), BinOp::Add, Box::new(r), s); }
            TokenKind::Minus => { self.advance(); let r = self.parse_mul_expr()?; let s = e.span().merge(r.span()); e = Expr::BinOp(Box::new(e), BinOp::Sub, Box::new(r), s); }
            _ => break } } Ok(e)
    }
    fn parse_mul_expr(&mut self) -> Result<Expr, SyntaxError> {
        let mut e = self.parse_unary_expr()?;
        loop { match self.peek() {
            TokenKind::Star => { self.advance(); let r = self.parse_unary_expr()?; let s = e.span().merge(r.span()); e = Expr::BinOp(Box::new(e), BinOp::Mul, Box::new(r), s); }
            TokenKind::Slash => { self.advance(); let r = self.parse_unary_expr()?; let s = e.span().merge(r.span()); e = Expr::BinOp(Box::new(e), BinOp::Div, Box::new(r), s); }
            TokenKind::Percent => { self.advance(); let r = self.parse_unary_expr()?; let s = e.span().merge(r.span()); e = Expr::BinOp(Box::new(e), BinOp::Mod, Box::new(r), s); }
            _ => break } } Ok(e)
    }
    fn parse_unary_expr(&mut self) -> Result<Expr, SyntaxError> {
        match self.peek() {
            TokenKind::Minus => { let s = self.span(); self.advance(); let o = self.parse_unary_expr()?; let sp = s.merge(o.span()); Ok(Expr::UnaryOp(UnaryOp::Neg, Box::new(o), sp)) }
            TokenKind::Bang => { let s = self.span(); self.advance(); let o = self.parse_unary_expr()?; let sp = s.merge(o.span()); Ok(Expr::UnaryOp(UnaryOp::Not, Box::new(o), sp)) }
            _ => self.parse_app_expr(),
        }
    }

    /// Application by juxtaposition: `f x y` -> App(App(f, x), y)
    fn parse_app_expr(&mut self) -> Result<Expr, SyntaxError> {
        let mut e = self.parse_postfix_expr()?;
        while self.is_atom_start() {
            let a = self.parse_postfix_expr()?;
            let s = e.span().merge(a.span());
            e = Expr::App(Box::new(e), Box::new(a), s);
        }
        Ok(e)
    }
    fn is_atom_start(&self) -> bool {
        matches!(self.peek(), TokenKind::IntLit(_) | TokenKind::FloatLit(_) | TokenKind::StringLit(_)
            | TokenKind::True | TokenKind::False | TokenKind::Ident(_) | TokenKind::LParen
            | TokenKind::LBrace)
    }
    fn parse_postfix_expr(&mut self) -> Result<Expr, SyntaxError> {
        let mut e = self.parse_atom_expr()?;
        while self.eat(&TokenKind::Dot) {
            let s_start = e.span();
            match self.peek().clone() {
                TokenKind::IntLit(i) => {
                    self.advance();
                    let s = s_start.merge(self.tokens[self.pos-1].span);
                    e = Expr::TupleAccess(Box::new(e), i as u16, s);
                }
                TokenKind::Ident(ref name) => {
                    let name = name.clone();
                    self.advance();
                    let s = s_start.merge(self.tokens[self.pos-1].span);
                    e = Expr::FieldAccess(Box::new(e), name, s);
                }
                _ => return Err(SyntaxError::new("expected field name or index after '.'", self.span())),
            }
        }
        Ok(e)
    }
    fn parse_atom_expr(&mut self) -> Result<Expr, SyntaxError> {
        match self.peek().clone() {
            TokenKind::IntLit(v) => { let s = self.span(); self.advance(); Ok(Expr::IntLit(v, s)) }
            TokenKind::FloatLit(v) => { let s = self.span(); self.advance(); Ok(Expr::FloatLit(v, s)) }
            TokenKind::StringLit(ref sv) => { let sv = sv.clone(); let s = self.span(); self.advance(); Ok(Expr::StringLit(sv, s)) }
            TokenKind::True => { let s = self.span(); self.advance(); Ok(Expr::BoolLit(true, s)) }
            TokenKind::False => { let s = self.span(); self.advance(); Ok(Expr::BoolLit(false, s)) }
            TokenKind::Ident(ref n) => { let n = n.clone(); let s = self.span(); self.advance(); Ok(Expr::Var(n, s)) }
            TokenKind::LParen => {
                let start = self.span(); self.advance();
                if self.eat(&TokenKind::RParen) { return Ok(Expr::UnitLit(start.merge(self.tokens[self.pos-1].span))); }
                // Operator section: (+), (*), etc.
                let op = match self.peek() {
                    TokenKind::Plus=>Some(BinOp::Add), TokenKind::Minus=>Some(BinOp::Sub),
                    TokenKind::Star=>Some(BinOp::Mul), TokenKind::Slash=>Some(BinOp::Div),
                    TokenKind::Percent=>Some(BinOp::Mod), TokenKind::EqEq=>Some(BinOp::Eq),
                    TokenKind::BangEq=>Some(BinOp::Ne), TokenKind::LAngle=>Some(BinOp::Lt),
                    TokenKind::RAngle=>Some(BinOp::Gt), TokenKind::LtEq=>Some(BinOp::Le),
                    TokenKind::GtEq=>Some(BinOp::Ge), _=>None };
                if let Some(op) = op {
                    let saved = self.pos; self.advance();
                    if self.at(&TokenKind::RParen) { let end = self.span(); self.advance();
                        return Ok(Expr::OpSection(op, start.merge(end))); }
                    self.pos = saved; // backtrack
                }
                let first = self.parse_expr()?;
                if self.eat(&TokenKind::Comma) {
                    // Support trailing comma: (x,) creates a 1-element tuple
                    if self.at(&TokenKind::RParen) {
                        let end = self.span(); self.advance();
                        return Ok(Expr::Tuple(vec![first], start.merge(end)));
                    }
                    let mut v = vec![first]; loop {
                        v.push(self.parse_expr()?);
                        if !self.eat(&TokenKind::Comma) { break; }
                        // Allow trailing comma before RParen
                        if self.at(&TokenKind::RParen) { break; }
                    }
                    let end = self.expect(&TokenKind::RParen)?; Ok(Expr::Tuple(v, start.merge(end)))
                } else { self.expect(&TokenKind::RParen)?; Ok(first) }
            }
            TokenKind::LBrace => {
                let start = self.span(); self.advance();
                let mut fields = Vec::new();
                if !self.at(&TokenKind::RBrace) {
                    loop {
                        let (fname, _) = self.expect_ident()?;
                        self.expect(&TokenKind::Eq)?;
                        let fval = self.parse_expr()?;
                        fields.push((fname, Box::new(fval)));
                        if !self.eat(&TokenKind::Comma) { break; }
                        if self.at(&TokenKind::RBrace) { break; }
                    }
                }
                let end = self.expect(&TokenKind::RBrace)?;
                Ok(Expr::RecordLit(fields, start.merge(end)))
            }
            _ => Err(SyntaxError::new(format!("expected expression, found {:?}", self.peek()), self.span())),
        }
    }
}
