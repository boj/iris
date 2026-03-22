use crate::syntax::error::Span;

#[derive(Debug, Clone)]
pub struct Module {
    pub items: Vec<Item>,
    /// Module-level capability declarations (allow/deny).
    pub capabilities: Option<CapabilityDecl>,
}

/// Module-level capability declarations.
///
/// Specifies which effects a module is allowed or denied.
/// ```iris
/// allow [FileRead, FileWrite "/tmp/*"]
/// deny [TcpConnect, ThreadSpawn, MmapExec]
/// ```
#[derive(Debug, Clone)]
pub struct CapabilityDecl {
    /// Effects explicitly allowed, with optional path/host arguments.
    pub allow: Vec<CapEntry>,
    /// Effects explicitly denied.
    pub deny: Vec<CapEntry>,
    pub span: Span,
}

/// A single capability entry: an effect name with optional argument.
#[derive(Debug, Clone)]
pub struct CapEntry {
    pub effect_name: String,
    pub argument: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Item {
    LetDecl(LetDecl),
    /// Mutual recursion group: `let rec f x = ... and g y = ...`
    MutualRecGroup(Vec<LetDecl>),
    TypeDecl(TypeDecl),
    Import(ImportDecl),
    ClassDecl(ClassDecl),
    InstanceDecl(InstanceDecl),
}

/// `class Eq<A> [requires Ord<A>] where eq : A -> A -> Bool`
#[derive(Debug, Clone)]
pub struct ClassDecl {
    pub name: String,
    pub type_param: String,
    pub superclasses: Vec<String>,
    pub methods: Vec<MethodDecl>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct MethodDecl {
    pub name: String,
    pub type_sig: TypeExpr,
    pub default_impl: Option<Expr>,
    pub span: Span,
}

/// `instance Eq<Int> where eq = \a b -> a == b`
#[derive(Debug, Clone)]
pub struct InstanceDecl {
    pub class_name: String,
    pub type_arg: TypeExpr,
    pub methods: Vec<(String, Expr)>,
    pub span: Span,
}

/// `let [rec] name params [: type] [cost] [requires ...] [ensures ...] = body`
///
/// When `recursive` is `true`, the declaration was written as `let rec name ...`
/// and the binding is in scope within its own body (i.e. it is a fixpoint).
#[derive(Debug, Clone)]
pub struct LetDecl {
    pub name: String,
    pub params: Vec<String>,
    pub ret_type: Option<TypeExpr>,
    pub cost: Option<CostExpr>,
    pub requires: Vec<Expr>,
    pub ensures: Vec<Expr>,
    pub body: Expr,
    pub span: Span,
    /// True when the declaration was introduced with `let rec`.
    pub recursive: bool,
}

#[derive(Debug, Clone)]
pub struct TypeDecl {
    pub name: String,
    pub type_params: Vec<String>,
    pub def: TypeExpr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ImportDecl {
    pub source: ImportSource,
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ImportSource {
    /// Content-addressed import: `import #deadbeef... as name`
    Hash(String),
    /// Path-based import: `import "stdlib/option.iris" as name`
    Path(String),
}

#[derive(Debug, Clone)]
pub enum TypeExpr {
    Named(String, Span),
    Arrow(Box<TypeExpr>, Box<TypeExpr>, Span),
    Tuple(Vec<TypeExpr>, Span),
    App(String, Vec<TypeExpr>, Span),
    Refined(String, Box<TypeExpr>, Box<Expr>, Span),
    ForAll(String, Box<TypeExpr>, Span),
    Unit(Span),
    /// Sum type (algebraic data type): `Ok(Int) | Error(String) | None`
    /// Each variant has a name and an optional payload type.
    Sum(Vec<(String, Option<Box<TypeExpr>>)>, Span),
    /// Record type (struct): `{ x: Int, y: Int }`
    /// Named fields compiled to a Product (tuple) with a field-name map.
    Record(Vec<(String, Box<TypeExpr>)>, Span),
    /// Record composition: `{ x: Int } / Other / Another`
    /// Merges fields from multiple record types. Duplicate field names are a compile error.
    RecordMerge(Box<TypeExpr>, Box<TypeExpr>, Span),
}

impl TypeExpr {
    pub fn span(&self) -> Span {
        match self {
            TypeExpr::Named(_, s) | TypeExpr::Arrow(_, _, s) | TypeExpr::Tuple(_, s)
            | TypeExpr::App(_, _, s) | TypeExpr::Refined(_, _, _, s)
            | TypeExpr::ForAll(_, _, s) | TypeExpr::Unit(s) | TypeExpr::Sum(_, s)
            | TypeExpr::Record(_, s) | TypeExpr::RecordMerge(_, _, s) => *s,
        }
    }
}

#[derive(Debug, Clone)]
pub enum CostExpr {
    Unknown, Zero, Constant(u64),
    Linear(String), NLogN(String), Polynomial(String, u32),
    Sum(Box<CostExpr>, Box<CostExpr>),
}

#[derive(Debug, Clone)]
pub enum Expr {
    IntLit(i64, Span),
    FloatLit(f64, Span),
    BoolLit(bool, Span),
    StringLit(String, Span),
    UnitLit(Span),
    Var(String, Span),
    Tuple(Vec<Expr>, Span),
    TupleAccess(Box<Expr>, u16, Span),
    App(Box<Expr>, Box<Expr>, Span),
    BinOp(Box<Expr>, BinOp, Box<Expr>, Span),
    UnaryOp(UnaryOp, Box<Expr>, Span),
    OpSection(BinOp, Span),
    Lambda(Vec<String>, Box<Expr>, Span),
    Let(String, Box<Expr>, Box<Expr>, Span),
    /// `let rec name = value in body` — a recursive local binding.
    LetRec(String, Box<Expr>, Box<Expr>, Span),
    If(Box<Expr>, Box<Expr>, Box<Expr>, Span),
    Match(Box<Expr>, Vec<MatchArm>, Span),
    Pipe(Box<Expr>, Box<Expr>, Span),
    /// Record literal: `{ x = 3, y = 4 }`
    RecordLit(Vec<(String, Box<Expr>)>, Span),
    /// Named field access: `point.x` (resolved to tuple index at compile time)
    FieldAccess(Box<Expr>, String, Span),
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::IntLit(_, s) | Expr::FloatLit(_, s) | Expr::BoolLit(_, s)
            | Expr::StringLit(_, s) | Expr::UnitLit(s) | Expr::Var(_, s)
            | Expr::Tuple(_, s) | Expr::TupleAccess(_, _, s) | Expr::App(_, _, s)
            | Expr::BinOp(_, _, _, s) | Expr::UnaryOp(_, _, s) | Expr::OpSection(_, s)
            | Expr::Lambda(_, _, s) | Expr::Let(_, _, _, s) | Expr::LetRec(_, _, _, s)
            | Expr::If(_, _, _, s) | Expr::Match(_, _, s) | Expr::Pipe(_, _, s)
            | Expr::RecordLit(_, s) | Expr::FieldAccess(_, _, s) => *s,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp { Add, Sub, Mul, Div, Mod, Eq, Ne, Lt, Gt, Le, Ge, And, Or }

impl BinOp {
    pub fn opcode(self) -> u8 {
        match self {
            BinOp::Add => 0x00, BinOp::Sub => 0x01, BinOp::Mul => 0x02,
            BinOp::Div => 0x03, BinOp::Mod => 0x04,
            BinOp::Eq => 0x20, BinOp::Ne => 0x21, BinOp::Lt => 0x22,
            BinOp::Gt => 0x23, BinOp::Le => 0x24, BinOp::Ge => 0x25,
            BinOp::And => 0x10, BinOp::Or => 0x11,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp { Neg, Not }

impl UnaryOp {
    pub fn opcode(self) -> u8 {
        match self { UnaryOp::Neg => 0x05, UnaryOp::Not => 0x13 }
    }
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Wildcard(Span),
    Ident(String, Span),
    IntLit(i64, Span),
    BoolLit(bool, Span),
    /// Constructor pattern: `Some(x)` or bare `None`.
    Constructor(String, Option<Box<Pattern>>, Span),
    /// Tuple pattern: `(a, b, c)`.
    Tuple(Vec<Pattern>, Span),
}
