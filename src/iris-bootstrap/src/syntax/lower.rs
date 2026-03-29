//! Lowering: AST → SemanticGraph
//!
//! Translates parsed ML-like IRIS surface syntax into the canonical
//! SemanticGraph representation.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

use iris_types::cost::{CostBound, CostTerm, CostVar};
use iris_types::fragment::{Boundary, Fragment, FragmentContracts, FragmentId, FragmentMeta};
use iris_types::graph::*;
use iris_types::hash::{compute_node_id, compute_type_id, SemanticHash};
use iris_types::types::*;

use crate::syntax::ast::{self, CostExpr, Expr, ImportSource, Item, LetDecl, Module, TypeExpr};
use crate::syntax::error::{Span, SyntaxError};
use crate::syntax::prim;

/// Standard Levenshtein edit distance (DP implementation).
fn levenshtein(a: &str, b: &str) -> usize {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr = vec![0; b.len() + 1];
    for i in 1..=a.len() {
        curr[0] = i;
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b.len()]
}

/// Map from SemanticGraph NodeIds back to source text locations.
pub type SourceMap = BTreeMap<NodeId, Span>;

#[derive(Debug)]
pub struct CompileResult {
    pub fragments: Vec<(String, Fragment, SourceMap)>,
    pub errors: Vec<SyntaxError>,
    /// Constructor bindings exported by this module (for import propagation)
    pub constructors: BTreeMap<String, (u16, TypeId)>,
    /// ADT type definitions exported by this module
    pub adt_types: BTreeMap<TypeId, TypeDef>,
}

#[derive(Debug, Clone)]
enum Binding {
    InputRef(u8, TypeId),
    Node(NodeId),
    Fragment(FragmentId),
    LambdaParam(u8),
    /// ADT constructor: tag index and the sum type it belongs to.
    Constructor(u16, TypeId),
}

struct LowerCtx {
    nodes: HashMap<NodeId, Node>,
    edges: Vec<Edge>,
    type_env: TypeEnv,
    scopes: Vec<BTreeMap<String, Binding>>,
    source_map: SourceMap,
    salt_counter: u64,
    /// Tracks lambda nesting depth. Each nested lambda gets a unique input-ref
    /// index (0x80 + depth) so inner lambdas don't collide with outer lambda
    /// params or function params (which use indices 0..N).
    lambda_depth: u32,
    /// Record field name -> index mappings, keyed by type name.
    record_fields: BTreeMap<String, Vec<String>>,
    /// Type variable bindings for parametric type monomorphization.
    /// Maps type parameter names (e.g. "T") to concrete TypeIds (e.g. Int).
    type_var_env: BTreeMap<String, TypeId>,
    /// Raw ADT definitions for monomorphization lookup.
    /// Maps type name (e.g. "Option") to its TypeDecl (including type_params).
    adt_defs: BTreeMap<String, ast::TypeDecl>,
    /// Type aliases: `type Alias = ExistingType` (structural, no new nominal type).
    /// Maps alias name to its expansion TypeExpr.
    type_aliases: BTreeMap<String, TypeExpr>,
}

impl LowerCtx {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            type_env: TypeEnv { types: BTreeMap::new() },
            scopes: vec![BTreeMap::new()],
            source_map: BTreeMap::new(),
            salt_counter: 0,
            lambda_depth: 0,
            record_fields: BTreeMap::new(),
            type_var_env: BTreeMap::new(),
            adt_defs: BTreeMap::new(),
            type_aliases: BTreeMap::new(),
        }
    }

    fn push_scope(&mut self) { self.scopes.push(BTreeMap::new()); }
    fn pop_scope(&mut self) { self.scopes.pop(); }

    fn bind(&mut self, name: String, binding: Binding) {
        self.scopes.last_mut().unwrap().insert(name, binding);
    }

    fn resolve(&self, name: &str) -> Option<&Binding> {
        for scope in self.scopes.iter().rev() {
            if let Some(b) = scope.get(name) { return Some(b); }
        }
        None
    }

    /// Collect all names visible in the current scope chain (user bindings only).
    fn scope_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.scopes.iter()
            .flat_map(|s| s.keys().map(|k| k.as_str()))
            .collect();
        names.sort_unstable();
        names.dedup();
        names
    }

    /// Find the closest name (Levenshtein distance <= 2) from user scope and primitives.
    fn suggest_name(&self, name: &str) -> Option<String> {
        let scope_names = self.scope_names();
        let prim_names = prim::primitive_names();
        let mut best: Option<(&str, usize)> = None;
        for candidate in scope_names.iter().chain(prim_names.iter()) {
            let d = levenshtein(name, candidate);
            if d > 0 && d <= 2 {
                if best.is_none() || d < best.unwrap().1 {
                    best = Some((candidate, d));
                }
            }
        }
        best.map(|(s, _)| s.to_string())
    }

    fn intern_type(&mut self, td: TypeDef) -> TypeId {
        let id = compute_type_id(&td);
        self.type_env.types.entry(id).or_insert(td);
        id
    }

    fn int_type(&mut self) -> TypeId {
        self.intern_type(TypeDef::Primitive(PrimType::Int))
    }

    #[allow(dead_code)]
    fn bool_type(&mut self) -> TypeId {
        self.intern_type(TypeDef::Primitive(PrimType::Bool))
    }

    fn unit_type(&mut self) -> TypeId {
        self.intern_type(TypeDef::Primitive(PrimType::Unit))
    }

    fn float_type(&mut self) -> TypeId {
        self.intern_type(TypeDef::Primitive(PrimType::Float64))
    }

    fn bytes_type(&mut self) -> TypeId {
        self.intern_type(TypeDef::Primitive(PrimType::Bytes))
    }

    /// Collect element TypeIds from a Product type definition.
    fn product_elements(&self, tid: TypeId) -> Vec<TypeId> {
        match self.type_env.types.get(&tid) {
            Some(TypeDef::Product(elems)) => elems.clone(),
            _ => vec![tid], // non-product: treat as single-element
        }
    }

    /// Convert a parsed TypeExpr into a content-addressed TypeId.
    fn lower_type_expr(&mut self, te: &TypeExpr) -> TypeId {
        match te {
            TypeExpr::Named(name, _) => match name.as_str() {
                "Int" | "Nat" => self.int_type(),
                "Bool" => self.intern_type(TypeDef::Primitive(PrimType::Bool)),
                "Float" | "Float64" => self.float_type(),
                "Float32" => self.intern_type(TypeDef::Primitive(PrimType::Float32)),
                "String" | "Bytes" => self.bytes_type(),
                "Unit" => self.unit_type(),
                "Program" => self.int_type(),
                _ => {
                    // Check if this is a bound type variable (e.g. T in Option<T>)
                    if let Some(&tid) = self.type_var_env.get(name.as_str()) {
                        tid
                    } else if let Some(alias_te) = self.type_aliases.get(name.as_str()).cloned() {
                        // Expand type alias to its definition
                        self.lower_type_expr(&alias_te)
                    } else {
                        self.int_type() // Unknown named types default to Int
                    }
                }
            },
            TypeExpr::Arrow(param, ret, _) => {
                let param_id = self.lower_type_expr(param);
                let ret_id = self.lower_type_expr(ret);
                self.intern_type(TypeDef::Arrow(param_id, ret_id, CostBound::Unknown))
            }
            TypeExpr::Tuple(elems, _) => {
                let elem_ids: Vec<TypeId> = elems.iter().map(|e| self.lower_type_expr(e)).collect();
                self.intern_type(TypeDef::Product(elem_ids))
            }
            TypeExpr::Unit(_) => self.unit_type(),
            TypeExpr::ForAll(var, body, _) => {
                let body_id = self.lower_type_expr(body);
                self.intern_type(TypeDef::ForAll(BoundVar(Self::hash_var_name(var)), body_id))
            }
            TypeExpr::Refined(var, base, pred, _) => {
                let base_id = self.lower_type_expr(base);
                let lia = self.lower_contract_expr(pred, &[(var.clone(), 0)]);
                self.intern_type(TypeDef::Refined(base_id, lia))
            }
            TypeExpr::App(name, args, _) => {
                if args.is_empty() {
                    return self.lower_type_expr(&TypeExpr::Named(name.clone(), Span::new(0, 0)));
                }
                // Monomorphize: look up the ADT definition, substitute type args, lower
                if let Some(adt) = self.adt_defs.get(name.as_str()).cloned() {
                    if adt.type_params.len() == args.len() {
                        // Build type_var_env: T -> Int, U -> String, etc.
                        let saved_env = self.type_var_env.clone();
                        for (param, arg) in adt.type_params.iter().zip(args.iter()) {
                            let arg_tid = self.lower_type_expr(arg);
                            self.type_var_env.insert(param.clone(), arg_tid);
                        }
                        let result = self.lower_type_expr(&adt.def);
                        self.type_var_env = saved_env;
                        return result;
                    }
                }
                // Fallback: treat as Product
                let elem_ids: Vec<TypeId> = args.iter().map(|a| self.lower_type_expr(a)).collect();
                self.intern_type(TypeDef::Product(elem_ids))
            }
            TypeExpr::Sum(variants, _) => {
                let tagged: Vec<(Tag, TypeId)> = variants.iter().enumerate().map(|(i, (_name, payload))| {
                    let payload_type = match payload {
                        Some(te) => self.lower_type_expr(te),
                        None => self.unit_type(),
                    };
                    (Tag(i as u16), payload_type)
                }).collect();
                self.intern_type(TypeDef::Sum(tagged))
            }
            TypeExpr::Record(fields, _) => {
                // Records compile to Product types (tuples) — field names are erased
                let elem_ids: Vec<TypeId> = fields.iter().map(|(_name, te)| self.lower_type_expr(te)).collect();
                self.intern_type(TypeDef::Product(elem_ids))
            }
            TypeExpr::RecordMerge(lhs, rhs, _) => {
                // Lower both sides, then concatenate their product elements
                let lhs_id = self.lower_type_expr(lhs);
                let rhs_id = self.lower_type_expr(rhs);
                let mut elems = self.product_elements(lhs_id);
                elems.extend(self.product_elements(rhs_id));
                self.intern_type(TypeDef::Product(elems))
            }
        }
    }

    /// Decompose an Arrow type annotation into parameter types and return type.
    /// `Int -> Int -> Bool` yields `([Int, Int], Bool)`.
    fn decompose_arrow(&mut self, te: &TypeExpr, n_params: usize) -> (Vec<TypeId>, TypeId) {
        let mut param_types = Vec::new();
        let mut current = te;
        for _ in 0..n_params {
            if let TypeExpr::Arrow(param, ret, _) = current {
                param_types.push(self.lower_type_expr(param));
                current = ret;
            } else {
                break;
            }
        }
        let ret_type = self.lower_type_expr(current);
        // Pad with Int if the annotation has fewer arrows than params
        while param_types.len() < n_params {
            param_types.push(self.int_type());
        }
        (param_types, ret_type)
    }

    fn hash_var_name(name: &str) -> u32 {
        let mut h: u32 = 0;
        for b in name.bytes() {
            h = h.wrapping_mul(31).wrapping_add(b as u32);
        }
        h
    }

    /// Convert a parsed contract expression (requires/ensures) into an LIAFormula.
    /// `var_map` maps variable names to BoundVar indices.
    fn lower_contract_expr(&self, expr: &Expr, var_map: &[(String, u32)]) -> LIAFormula {
        match expr {
            Expr::BoolLit(true, _) => LIAFormula::True,
            Expr::BoolLit(false, _) => LIAFormula::False,
            Expr::BinOp(lhs, op, rhs, _) => {
                use ast::BinOp::*;
                match op {
                    And => LIAFormula::And(
                        Box::new(self.lower_contract_expr(lhs, var_map)),
                        Box::new(self.lower_contract_expr(rhs, var_map)),
                    ),
                    Or => LIAFormula::Or(
                        Box::new(self.lower_contract_expr(lhs, var_map)),
                        Box::new(self.lower_contract_expr(rhs, var_map)),
                    ),
                    Eq => LIAFormula::Atom(LIAAtom::Eq(
                        self.lower_lia_term(lhs, var_map),
                        self.lower_lia_term(rhs, var_map),
                    )),
                    Lt => LIAFormula::Atom(LIAAtom::Lt(
                        self.lower_lia_term(lhs, var_map),
                        self.lower_lia_term(rhs, var_map),
                    )),
                    Le => LIAFormula::Atom(LIAAtom::Le(
                        self.lower_lia_term(lhs, var_map),
                        self.lower_lia_term(rhs, var_map),
                    )),
                    Gt => LIAFormula::Atom(LIAAtom::Lt(
                        self.lower_lia_term(rhs, var_map),
                        self.lower_lia_term(lhs, var_map),
                    )),
                    Ge => LIAFormula::Atom(LIAAtom::Le(
                        self.lower_lia_term(rhs, var_map),
                        self.lower_lia_term(lhs, var_map),
                    )),
                    Ne => LIAFormula::Not(Box::new(LIAFormula::Atom(LIAAtom::Eq(
                        self.lower_lia_term(lhs, var_map),
                        self.lower_lia_term(rhs, var_map),
                    )))),
                    _ => {
                        // Arithmetic comparison on the whole expression — treat as True
                        LIAFormula::True
                    }
                }
            }
            Expr::UnaryOp(ast::UnaryOp::Not, inner, _) => {
                LIAFormula::Not(Box::new(self.lower_contract_expr(inner, var_map)))
            }
            _ => LIAFormula::True, // Unparseable contracts default to True (no constraint)
        }
    }

    /// Convert a parsed expression into an LIA term for contract formulas.
    fn lower_lia_term(&self, expr: &Expr, var_map: &[(String, u32)]) -> LIATerm {
        match expr {
            Expr::IntLit(n, _) => LIATerm::Const(*n),
            Expr::Var(name, _) => {
                if name == "result" {
                    LIATerm::Var(BoundVar(0xFFFF))
                } else if let Some((_, idx)) = var_map.iter().find(|(n, _)| n == name) {
                    LIATerm::Var(BoundVar(*idx))
                } else {
                    LIATerm::Var(BoundVar(Self::hash_var_name(name)))
                }
            }
            Expr::BinOp(lhs, op, rhs, _) => {
                use ast::BinOp::*;
                match op {
                    Add => LIATerm::Add(
                        Box::new(self.lower_lia_term(lhs, var_map)),
                        Box::new(self.lower_lia_term(rhs, var_map)),
                    ),
                    Sub => LIATerm::Add(
                        Box::new(self.lower_lia_term(lhs, var_map)),
                        Box::new(LIATerm::Neg(Box::new(self.lower_lia_term(rhs, var_map)))),
                    ),
                    Mul => {
                        // LIATerm::Mul takes (i64, Box<LIATerm>)
                        if let Expr::IntLit(n, _) = lhs.as_ref() {
                            LIATerm::Mul(*n, Box::new(self.lower_lia_term(rhs, var_map)))
                        } else if let Expr::IntLit(n, _) = rhs.as_ref() {
                            LIATerm::Mul(*n, Box::new(self.lower_lia_term(lhs, var_map)))
                        } else {
                            LIATerm::Const(0) // Non-linear — can't represent
                        }
                    }
                    Mod => LIATerm::Mod(
                        Box::new(self.lower_lia_term(lhs, var_map)),
                        Box::new(self.lower_lia_term(rhs, var_map)),
                    ),
                    _ => LIATerm::Const(0),
                }
            }
            Expr::UnaryOp(ast::UnaryOp::Neg, inner, _) => {
                LIATerm::Neg(Box::new(self.lower_lia_term(inner, var_map)))
            }
            _ => LIATerm::Const(0),
        }
    }

    fn insert_node(&mut self, mut node: Node) -> NodeId {
        // Every node from the lowerer gets a unique salt so that
        // structurally-identical nodes (e.g. many Guard nodes using
        // the same eq comparison) map to distinct NodeIds.
        self.salt_counter += 1;
        node.salt = self.salt_counter;
        node.id = compute_node_id(&node);
        let id = node.id;
        self.nodes.insert(id, node);
        id
    }

    fn add_edge(&mut self, source: NodeId, target: NodeId, port: u8, label: EdgeLabel) {
        self.edges.push(Edge { source, target, port, label });
    }

    /// Record that `node_id` was produced from the source text at `span`.
    fn record_span(&mut self, node_id: NodeId, span: Span) {
        self.source_map.entry(node_id).or_insert(span);
    }

    fn make_int_lit(&mut self, value: i64) -> NodeId {
        let t = self.int_type();
        self.insert_node(Node { id: NodeId(0), kind: NodeKind::Lit, type_sig: t,
            cost: CostTerm::Unit, arity: 0, resolution_depth: 0, salt: 0,
            payload: NodePayload::Lit { type_tag: 0x00, value: value.to_le_bytes().to_vec() } })
    }

    fn make_float_lit(&mut self, value: f64) -> NodeId {
        let t = self.intern_type(TypeDef::Primitive(PrimType::Float64));
        self.insert_node(Node { id: NodeId(0), kind: NodeKind::Lit, type_sig: t,
            cost: CostTerm::Unit, arity: 0, resolution_depth: 0, salt: 0,
            payload: NodePayload::Lit { type_tag: 0x02, value: value.to_le_bytes().to_vec() } })
    }

    fn make_bool_lit(&mut self, value: bool) -> NodeId {
        let t = self.intern_type(TypeDef::Primitive(PrimType::Bool));
        self.insert_node(Node { id: NodeId(0), kind: NodeKind::Lit, type_sig: t,
            cost: CostTerm::Unit, arity: 0, resolution_depth: 0, salt: 0,
            payload: NodePayload::Lit { type_tag: 0x04, value: vec![value as u8] } })
    }

    fn make_string_lit(&mut self, value: &str) -> NodeId {
        let t = self.intern_type(TypeDef::Primitive(PrimType::Bytes));
        self.insert_node(Node { id: NodeId(0), kind: NodeKind::Lit, type_sig: t,
            cost: CostTerm::Unit, arity: 0, resolution_depth: 0, salt: 0,
            payload: NodePayload::Lit { type_tag: 0x07, value: value.as_bytes().to_vec() } })
    }

    fn make_input_ref(&mut self, index: u8, type_id: TypeId) -> NodeId {
        self.insert_node(Node { id: NodeId(0), kind: NodeKind::Lit, type_sig: type_id,
            cost: CostTerm::Unit, arity: 0, resolution_depth: 0, salt: 0,
            payload: NodePayload::Lit { type_tag: 0xFF, value: vec![index] } })
    }

    fn make_unit_lit(&mut self) -> NodeId {
        let t = self.intern_type(TypeDef::Primitive(PrimType::Unit));
        self.insert_node(Node { id: NodeId(0), kind: NodeKind::Lit, type_sig: t,
            cost: CostTerm::Unit, arity: 0, resolution_depth: 0, salt: 0,
            payload: NodePayload::Lit { type_tag: 0x06, value: vec![] } })
    }

    /// Create an Inject node: wraps a payload value with a tag to produce a Sum value.
    fn make_inject(&mut self, tag: u16, sum_type: TypeId, payload_id: NodeId) -> NodeId {
        let inject_id = self.insert_node(Node {
            id: NodeId(0), kind: NodeKind::Inject, type_sig: sum_type,
            cost: CostTerm::Unit, arity: 1, resolution_depth: 0, salt: 0,
            payload: NodePayload::Inject { tag_index: tag },
        });
        self.add_edge(inject_id, payload_id, 0, EdgeLabel::Argument);
        inject_id
    }

    fn make_prim(&mut self, opcode: u8, arity: u8) -> NodeId {
        let t = self.prim_result_type(opcode);
        self.insert_node(Node { id: NodeId(0), kind: NodeKind::Prim, type_sig: t,
            cost: CostTerm::Unit, arity, resolution_depth: 0, salt: 0,
            payload: NodePayload::Prim { opcode } })
    }

    /// Infer the result type of a primitive opcode.
    fn prim_result_type(&mut self, opcode: u8) -> TypeId {
        match opcode {
            // Comparisons return Bool
            0x20..=0x25 => self.intern_type(TypeDef::Primitive(PrimType::Bool)),
            // String comparisons return Bool
            0xB3 | 0xB8 | 0xB9 | 0xBA => self.intern_type(TypeDef::Primitive(PrimType::Bool)),
            // bool_to_int returns Int
            0x44 => self.int_type(),
            // int_to_float returns Float64
            0x40 => self.intern_type(TypeDef::Primitive(PrimType::Float64)),
            // float_to_int returns Int
            0x41 => self.int_type(),
            // String ops that return strings
            0xB1 | 0xB2 | 0xB5 | 0xB7 => self.intern_type(TypeDef::Primitive(PrimType::Bytes)),
            // String length, str_to_int return Int
            0xB0 | 0xB6 => self.int_type(),
            // Everything else: Int (arithmetic, bitwise, etc.)
            _ => self.int_type(),
        }
    }

    // -----------------------------------------------------------------------
    // Uncurry App chains: `f x y z` → (head=f, args=[x, y, z])
    // -----------------------------------------------------------------------

    fn uncurry<'a>(&self, expr: &'a Expr) -> (&'a Expr, Vec<&'a Expr>) {
        let mut head = expr;
        let mut args = Vec::new();
        while let Expr::App(f, a, _) = head {
            args.push(a.as_ref());
            head = f.as_ref();
        }
        args.reverse();
        (head, args)
    }

    // -----------------------------------------------------------------------
    // Expression lowering
    // -----------------------------------------------------------------------

    fn lower_expr(&mut self, expr: &Expr) -> Result<NodeId, SyntaxError> {
        let span = expr.span();
        let node_id = match expr {
            Expr::IntLit(v, _) => Ok(self.make_int_lit(*v)),
            Expr::FloatLit(v, _) => Ok(self.make_float_lit(*v)),
            Expr::BoolLit(v, _) => Ok(self.make_bool_lit(*v)),
            Expr::StringLit(s, _) => Ok(self.make_string_lit(s)),
            Expr::UnitLit(_) => Ok(self.make_unit_lit()),
            Expr::Var(name, span) => self.lower_var(name, *span),
            Expr::OpSection(op, _) => Ok(self.make_prim(op.opcode(), 2)),
            Expr::Tuple(elems, _) => self.lower_tuple(elems),
            Expr::TupleAccess(base, idx, _) => self.lower_tuple_access(base, *idx),
            Expr::BinOp(lhs, op, rhs, _) => self.lower_binop(lhs, *op, rhs),
            Expr::UnaryOp(op, operand, _) => self.lower_unary(*op, operand),
            Expr::Lambda(params, body, _) => self.lower_lambda(params, body),
            Expr::Let(name, value, body, _) => self.lower_let(name, value, body),
            // `let rec` lowers the same as `let` at the graph level; the `recursive`
            // flag carries semantic information for future fixpoint-aware passes.
            Expr::LetRec(name, value, body, _) => self.lower_let(name, value, body),
            Expr::If(cond, then_e, else_e, _) => self.lower_if(cond, then_e, else_e),
            Expr::Match(scrutinee, arms, _) => self.lower_match(scrutinee, arms),
            Expr::Pipe(lhs, rhs, _) => self.lower_pipe(lhs, rhs),
            Expr::App(_, _, _) => self.lower_app(expr),
            Expr::RecordLit(fields, _) => self.lower_record_lit(fields),
            Expr::FieldAccess(base, field, span) => self.lower_field_access(base, field, *span),
        }?;
        self.record_span(node_id, span);
        Ok(node_id)
    }

    fn lower_var(&mut self, name: &str, span: Span) -> Result<NodeId, SyntaxError> {
        match self.resolve(name) {
            Some(Binding::InputRef(idx, tid)) => {
                let idx = *idx; let tid = *tid;
                Ok(self.make_input_ref(idx, tid))
            }
            Some(Binding::Node(nid)) => Ok(*nid),
            Some(Binding::LambdaParam(idx)) => {
                let idx = *idx; let t = self.int_type();
                Ok(self.make_input_ref(idx, t))
            }
            Some(Binding::Fragment(fid)) => {
                let fid = *fid; let t = self.int_type();
                Ok(self.insert_node(Node { id: NodeId(0), kind: NodeKind::Ref, type_sig: t,
                    cost: CostTerm::Unit, arity: 0, resolution_depth: 0, salt: 0,
                    payload: NodePayload::Ref { fragment_id: fid } }))
            }
            Some(Binding::Constructor(tag, sum_type)) => {
                // Bare constructor (no args) → Inject with Unit payload
                let tag = *tag; let sum_type = *sum_type;
                let unit_id = self.make_unit_lit();
                Ok(self.make_inject(tag, sum_type, unit_id))
            }
            None => {
                // Named primitive used as a value (e.g., `add` passed to fold)
                if let Some((opcode, arity)) = prim::resolve_primitive(name) {
                    Ok(self.make_prim(opcode, arity))
                } else {
                    let msg = if let Some(suggestion) = self.suggest_name(name) {
                        format!("unknown identifier '{name}', did you mean '{suggestion}'?")
                    } else {
                        format!("undefined variable: '{name}'")
                    };
                    Err(SyntaxError::new(msg, span))
                }
            }
        }
    }

    fn lower_app(&mut self, expr: &Expr) -> Result<NodeId, SyntaxError> {
        let (head, args) = self.uncurry(expr);

        // Check if head is a known built-in form
        if let Expr::Var(name, span) = head {
            let span = *span;

            // If the name resolves to a user/imported binding (e.g., a Fragment
            // from an import), skip built-in handling so the imported definition
            // takes precedence over built-in keywords like "map" and "filter".
            let resolved = self.resolve(name);
            let has_user_binding = matches!(
                resolved,
                Some(Binding::Fragment(_) | Binding::Node(_) | Binding::LambdaParam(_) | Binding::Constructor(_, _))
            );

            if !has_user_binding {
            match name.as_str() {
                "fold" => return self.lower_fold_args(&args, span),
                "fold_until" => return self.lower_fold_until_args(&args, span),
                "unfold" => return self.lower_unfold_args(&args, span),
                "guard" => return self.lower_guard_args(&args, span),
                "effect" => return self.lower_effect_args(&args, span),
                "map" if args.len() == 2 => return self.lower_map_args(&args),
                "filter" if args.len() == 2 => return self.lower_filter_args(&args),
                _ => {}
            }
            }

            // ADT constructor with argument: `Some(42)` → Inject(tag, payload)
            if let Some(Binding::Constructor(tag, sum_type)) = self.resolve(name).cloned() {
                if args.len() == 1 {
                    let payload_id = self.lower_expr(args[0])?;
                    return Ok(self.make_inject(tag, sum_type, payload_id));
                }
                if args.len() > 1 {
                    let arg_ids: Vec<NodeId> = args.iter()
                        .map(|a| self.lower_expr(a)).collect::<Result<_, _>>()?;
                    let tuple_id = self.lower_tuple_from_ids(&arg_ids);
                    return Ok(self.make_inject(tag, sum_type, tuple_id));
                }
                // No args → bare inject with Unit
                let unit_id = self.make_unit_lit();
                return Ok(self.make_inject(tag, sum_type, unit_id));
            }

            // Effect primitives used as direct calls (e.g., `tcp_listen port`)
            if let Some(tag) = resolve_effect_name(name) {
                let arg_ids: Vec<NodeId> = args.iter()
                    .map(|a| self.lower_expr(a)).collect::<Result<_, _>>()?;
                let t = self.unit_type();
                let eff_id = self.insert_node(Node { id: NodeId(0), kind: NodeKind::Effect,
                    type_sig: t, cost: CostTerm::Unit, arity: arg_ids.len() as u8,
                    resolution_depth: 0, salt: 0, payload: NodePayload::Effect { effect_tag: tag } });
                for (port, arg) in arg_ids.iter().enumerate() {
                    self.add_edge(eff_id, *arg, port as u8, EdgeLabel::Argument);
                }
                return Ok(eff_id);
            }

            // Named primitive with arguments — skip if user has a binding
            // (e.g., imported function named "map" takes precedence over the
            // built-in map primitive).
            if !has_user_binding {
            if let Some((opcode, _)) = prim::resolve_primitive(name) {
                let arg_ids: Vec<NodeId> = args.iter()
                    .map(|a| self.lower_expr(a)).collect::<Result<_, _>>()?;
                let prim_id = self.make_prim(opcode, arg_ids.len() as u8);
                for (port, arg) in arg_ids.iter().enumerate() {
                    self.add_edge(prim_id, *arg, port as u8, EdgeLabel::Argument);
                }
                return Ok(prim_id);
            }
            }

            // Fragment reference call — attach all args to the Ref node so
            // eval_ref can apply them within the referenced fragment's context.
            if let Some(Binding::Fragment(fid)) = self.resolve(name) {
                let fid = *fid;
                let arg_ids: Vec<NodeId> = args.iter()
                    .map(|a| self.lower_expr(a)).collect::<Result<_, _>>()?;
                let t = self.int_type();
                let ref_id = self.insert_node(Node { id: NodeId(0), kind: NodeKind::Ref,
                    type_sig: t, cost: CostTerm::Unit, arity: arg_ids.len() as u8,
                    resolution_depth: 0, salt: 0, payload: NodePayload::Ref { fragment_id: fid } });
                for (port, arg) in arg_ids.iter().enumerate() {
                    self.add_edge(ref_id, *arg, port as u8, EdgeLabel::Argument);
                }
                return Ok(ref_id);
            }
        }

        // General application: lower head, then apply arguments one by one
        let mut fn_id = self.lower_expr(head)?;
        for arg in &args {
            let arg_id = self.lower_expr(arg)?;
            let t = self.int_type();
            let apply_id = self.insert_node(Node { id: NodeId(0), kind: NodeKind::Apply,
                type_sig: t, cost: CostTerm::Unit, arity: 2, resolution_depth: 0, salt: 0,
                payload: NodePayload::Apply });
            self.add_edge(apply_id, fn_id, 0, EdgeLabel::Argument);
            self.add_edge(apply_id, arg_id, 1, EdgeLabel::Argument);
            fn_id = apply_id;
        }
        Ok(fn_id)
    }

    fn lower_fold_args(&mut self, args: &[&Expr], span: Span) -> Result<NodeId, SyntaxError> {
        if args.len() < 2 || args.len() > 3 {
            return Err(SyntaxError::new(
                format!("fold expects 2 or 3 arguments, got {}", args.len()), span));
        }
        let base_id = self.lower_expr(args[0])?;
        let step_id = self.lower_expr(args[1])?;
        let t = self.int_type();
        let fold_id = self.insert_node(Node { id: NodeId(0), kind: NodeKind::Fold, type_sig: t,
            cost: CostTerm::Unit, arity: args.len() as u8, resolution_depth: 0, salt: 0,
            payload: NodePayload::Fold { recursion_descriptor: vec![0x00] } });
        self.add_edge(fold_id, base_id, 0, EdgeLabel::Argument);
        self.add_edge(fold_id, step_id, 1, EdgeLabel::Argument);
        if args.len() == 3 {
            let coll_id = self.lower_expr(args[2])?;
            self.add_edge(fold_id, coll_id, 2, EdgeLabel::Argument);
        }
        Ok(fold_id)
    }

    /// Lower `fold_until pred acc step list` — fold with early exit.
    /// pred: accumulator -> Bool (stop when true)
    /// Compiles to Fold node with recursion_descriptor 0x0A.
    fn lower_fold_until_args(&mut self, args: &[&Expr], span: Span) -> Result<NodeId, SyntaxError> {
        if args.len() != 4 {
            return Err(SyntaxError::new(
                format!("fold_until expects 4 arguments (pred, acc, step, list), got {}", args.len()), span));
        }
        let pred_id = self.lower_expr(args[0])?;
        let base_id = self.lower_expr(args[1])?;
        let step_id = self.lower_expr(args[2])?;
        let coll_id = self.lower_expr(args[3])?;
        let t = self.int_type();
        let fold_id = self.insert_node(Node { id: NodeId(0), kind: NodeKind::Fold, type_sig: t,
            cost: CostTerm::Unit, arity: 4, resolution_depth: 0, salt: 0,
            payload: NodePayload::Fold { recursion_descriptor: vec![0x0A] } });
        self.add_edge(fold_id, base_id, 0, EdgeLabel::Argument);
        self.add_edge(fold_id, step_id, 1, EdgeLabel::Argument);
        self.add_edge(fold_id, coll_id, 2, EdgeLabel::Argument);
        self.add_edge(fold_id, pred_id, 3, EdgeLabel::Argument);
        Ok(fold_id)
    }

    fn lower_unfold_args(&mut self, args: &[&Expr], span: Span) -> Result<NodeId, SyntaxError> {
        if args.len() != 3 {
            return Err(SyntaxError::new(
                format!("unfold expects 3 arguments, got {}", args.len()), span));
        }
        let seed_id = self.lower_expr(args[0])?;
        let step_id = self.lower_expr(args[1])?;
        let bound_id = self.lower_expr(args[2])?;
        let dummy_term = self.make_int_lit(0);
        let t = self.int_type();
        let unfold_id = self.insert_node(Node { id: NodeId(0), kind: NodeKind::Unfold,
            type_sig: t, cost: CostTerm::Unit, arity: 4, resolution_depth: 0, salt: 0,
            payload: NodePayload::Unfold { recursion_descriptor: vec![0x00] } });
        self.add_edge(unfold_id, seed_id, 0, EdgeLabel::Argument);
        self.add_edge(unfold_id, step_id, 1, EdgeLabel::Argument);
        self.add_edge(unfold_id, dummy_term, 2, EdgeLabel::Argument);
        self.add_edge(unfold_id, bound_id, 3, EdgeLabel::Argument);
        Ok(unfold_id)
    }

    fn lower_guard_args(&mut self, args: &[&Expr], span: Span) -> Result<NodeId, SyntaxError> {
        if args.len() != 3 {
            return Err(SyntaxError::new(
                format!("guard expects 3 arguments, got {}", args.len()), span));
        }
        let pred_id = self.lower_expr(args[0])?;
        let body_id = self.lower_expr(args[1])?;
        let fallback_id = self.lower_expr(args[2])?;
        let t = self.int_type();
        Ok(self.insert_node(Node { id: NodeId(0), kind: NodeKind::Guard, type_sig: t,
            cost: CostTerm::Unit, arity: 0, resolution_depth: 0, salt: 0,
            payload: NodePayload::Guard { predicate_node: pred_id, body_node: body_id,
                fallback_node: fallback_id } }))
    }

    fn lower_effect_args(&mut self, args: &[&Expr], span: Span) -> Result<NodeId, SyntaxError> {
        if args.len() < 2 {
            return Err(SyntaxError::new(
                format!("effect expects at least 2 arguments (tag, args...), got {}", args.len()), span));
        }
        let effect_tag = match args[0] {
            Expr::Var(name, _) => match resolve_effect_name(name) {
                Some(tag) => tag,
                None => return Err(SyntaxError::new(
                    format!("unknown effect tag: '{name}'"), span)),
            },
            Expr::IntLit(v, _) => *v as u8,
            _ => return Err(SyntaxError::new(
                "effect tag must be an identifier or integer", span)),
        };
        // Remaining args become effect payload arguments
        let arg_ids: Vec<NodeId> = args[1..].iter()
            .map(|a| self.lower_expr(a)).collect::<Result<_, _>>()?;
        let t = self.unit_type();
        let eff_id = self.insert_node(Node { id: NodeId(0), kind: NodeKind::Effect,
            type_sig: t, cost: CostTerm::Unit, arity: arg_ids.len() as u8,
            resolution_depth: 0, salt: 0, payload: NodePayload::Effect { effect_tag } });
        for (port, arg) in arg_ids.iter().enumerate() {
            self.add_edge(eff_id, *arg, port as u8, EdgeLabel::Argument);
        }
        Ok(eff_id)
    }

    fn lower_map_args(&mut self, args: &[&Expr]) -> Result<NodeId, SyntaxError> {
        let fn_id = self.lower_expr(args[0])?;
        let coll_id = self.lower_expr(args[1])?;
        let prim_id = self.make_prim(0x30, 2);
        self.add_edge(prim_id, coll_id, 0, EdgeLabel::Argument);
        self.add_edge(prim_id, fn_id, 1, EdgeLabel::Argument);
        Ok(prim_id)
    }

    fn lower_filter_args(&mut self, args: &[&Expr]) -> Result<NodeId, SyntaxError> {
        let fn_id = self.lower_expr(args[0])?;
        let coll_id = self.lower_expr(args[1])?;
        let prim_id = self.make_prim(0x31, 2);
        self.add_edge(prim_id, coll_id, 0, EdgeLabel::Argument);
        self.add_edge(prim_id, fn_id, 1, EdgeLabel::Argument);
        Ok(prim_id)
    }

    fn lower_tuple(&mut self, elems: &[Expr]) -> Result<NodeId, SyntaxError> {
        let child_ids: Vec<NodeId> = elems.iter()
            .map(|e| self.lower_expr(e)).collect::<Result<_, _>>()?;
        Ok(self.lower_tuple_from_ids(&child_ids))
    }

    /// Build a Tuple node from already-lowered child NodeIds.
    fn lower_tuple_from_ids(&mut self, child_ids: &[NodeId]) -> NodeId {
        let elem_types: Vec<TypeId> = child_ids.iter().map(|id| {
            self.nodes.get(id).map(|n| n.type_sig).unwrap_or_else(|| self.int_type())
        }).collect();
        let t = self.intern_type(TypeDef::Product(elem_types));
        let tuple_id = self.insert_node(Node { id: NodeId(0), kind: NodeKind::Tuple,
            type_sig: t, cost: CostTerm::Unit, arity: child_ids.len() as u8,
            resolution_depth: 0, salt: 0, payload: NodePayload::Tuple });
        for (port, child) in child_ids.iter().enumerate() {
            self.add_edge(tuple_id, *child, port as u8, EdgeLabel::Argument);
        }
        tuple_id
    }

    fn lower_tuple_access(&mut self, base: &Expr, idx: u16) -> Result<NodeId, SyntaxError> {
        let base_id = self.lower_expr(base)?;
        let t = self.int_type();
        let proj_id = self.insert_node(Node { id: NodeId(0), kind: NodeKind::Project,
            type_sig: t, cost: CostTerm::Unit, arity: 1, resolution_depth: 0, salt: 0,
            payload: NodePayload::Project { field_index: idx } });
        self.add_edge(proj_id, base_id, 0, EdgeLabel::Argument);
        Ok(proj_id)
    }

    fn lower_record_lit(&mut self, fields: &[(String, Box<Expr>)]) -> Result<NodeId, SyntaxError> {
        // Record literals compile to Tuple nodes with fields in declaration order.
        // Since we don't enforce a specific type here, we compile in source order.
        // The type checker validates field names against the declared record type.
        let t = self.int_type();
        let tuple_id = self.insert_node(Node { id: NodeId(0), kind: NodeKind::Tuple,
            type_sig: t, cost: CostTerm::Unit, arity: fields.len() as u8,
            resolution_depth: 0, salt: 0, payload: NodePayload::Tuple });
        for (i, (_name, val)) in fields.iter().enumerate() {
            let val_id = self.lower_expr(val)?;
            self.add_edge(tuple_id, val_id, i as u8, EdgeLabel::Argument);
        }
        Ok(tuple_id)
    }

    fn lower_field_access(&mut self, base: &Expr, field: &str, span: Span) -> Result<NodeId, SyntaxError> {
        // Resolve named field to a numeric index using the record_fields map.
        // Try all registered record types to find the field.
        let mut resolved_idx = None;
        for (_type_name, fields) in &self.record_fields {
            if let Some(idx) = fields.iter().position(|f| f == field) {
                resolved_idx = Some(idx as u16);
                break;
            }
        }
        match resolved_idx {
            Some(idx) => self.lower_tuple_access(base, idx),
            None => Err(SyntaxError::new(format!("unknown field '{}'", field), span)),
        }
    }

    fn lower_binop(&mut self, lhs: &Expr, op: ast::BinOp, rhs: &Expr) -> Result<NodeId, SyntaxError> {
        // Short-circuit: a && b  ->  if a then b else false
        if op == ast::BinOp::And {
            return self.lower_if(lhs, rhs, &Expr::BoolLit(false, lhs.span()));
        }
        // Short-circuit: a || b  ->  if a then true else b
        if op == ast::BinOp::Or {
            return self.lower_if(lhs, &Expr::BoolLit(true, lhs.span()), rhs);
        }

        let lhs_id = self.lower_expr(lhs)?;
        let rhs_id = self.lower_expr(rhs)?;
        let result_type = self.prim_result_type(op.opcode());
        let prim_id = self.insert_node(Node { id: NodeId(0), kind: NodeKind::Prim,
            type_sig: result_type, cost: CostTerm::Unit, arity: 2, resolution_depth: 0, salt: 0,
            payload: NodePayload::Prim { opcode: op.opcode() } });
        self.add_edge(prim_id, lhs_id, 0, EdgeLabel::Argument);
        self.add_edge(prim_id, rhs_id, 1, EdgeLabel::Argument);
        Ok(prim_id)
    }

    fn lower_unary(&mut self, op: ast::UnaryOp, operand: &Expr) -> Result<NodeId, SyntaxError> {
        let operand_id = self.lower_expr(operand)?;
        let result_type = self.prim_result_type(op.opcode());
        let prim_id = self.insert_node(Node { id: NodeId(0), kind: NodeKind::Prim,
            type_sig: result_type, cost: CostTerm::Unit, arity: 1, resolution_depth: 0, salt: 0,
            payload: NodePayload::Prim { opcode: op.opcode() } });
        self.add_edge(prim_id, operand_id, 0, EdgeLabel::Argument);
        Ok(prim_id)
    }

    fn lower_lambda(&mut self, params: &[String], body: &Expr) -> Result<NodeId, SyntaxError> {
        // Each lambda nesting level uses a unique input-ref index (0x80 + depth)
        // so that inner lambdas don't shadow outer lambda params or function
        // params (which use indices 0..N where N < 128 in practice).
        let ref_index = (0x80 + self.lambda_depth) as u8;
        let binder_id = BinderId(0xFFFF_0000 + ref_index as u32);
        self.lambda_depth += 1;
        self.push_scope();
        if params.len() == 1 {
            self.bind(params[0].clone(), Binding::LambdaParam(ref_index));
        } else {
            let t = self.int_type();
            let input_ref_id = self.make_input_ref(ref_index, t);
            for (i, param) in params.iter().enumerate() {
                let t = self.int_type();
                let proj_id = self.insert_node(Node { id: NodeId(0), kind: NodeKind::Project,
                    type_sig: t, cost: CostTerm::Unit, arity: 1,
                    resolution_depth: i as u8, salt: 0,
                    payload: NodePayload::Project { field_index: i as u16 } });
                self.add_edge(proj_id, input_ref_id, 0, EdgeLabel::Argument);
                self.bind(param.clone(), Binding::Node(proj_id));
            }
        }
        let body_id = self.lower_expr(body)?;
        self.pop_scope();
        self.lambda_depth -= 1;
        let t = self.int_type();
        let lambda_id = self.insert_node(Node { id: NodeId(0), kind: NodeKind::Lambda,
            type_sig: t, cost: CostTerm::Unit, arity: 0, resolution_depth: 0, salt: 0,
            payload: NodePayload::Lambda { binder: binder_id, captured_count: 0 } });
        self.add_edge(lambda_id, body_id, 0, EdgeLabel::Continuation);
        Ok(lambda_id)
    }

    fn lower_let(&mut self, name: &str, value: &Expr, body: &Expr) -> Result<NodeId, SyntaxError> {
        let value_id = self.lower_expr(value)?;
        self.push_scope();
        self.bind(name.to_string(), Binding::Node(value_id));
        let body_id = self.lower_expr(body)?;
        self.pop_scope();
        Ok(body_id)
    }

    fn lower_if(&mut self, cond: &Expr, then_e: &Expr, else_e: &Expr) -> Result<NodeId, SyntaxError> {
        let pred_id = self.lower_expr(cond)?;
        let body_id = self.lower_expr(then_e)?;
        let fallback_id = self.lower_expr(else_e)?;
        let t = self.int_type();
        Ok(self.insert_node(Node { id: NodeId(0), kind: NodeKind::Guard, type_sig: t,
            cost: CostTerm::Unit, arity: 0, resolution_depth: 0, salt: 0,
            payload: NodePayload::Guard { predicate_node: pred_id, body_node: body_id,
                fallback_node: fallback_id } }))
    }

    fn lower_match(&mut self, scrutinee: &Expr, arms: &[ast::MatchArm]) -> Result<NodeId, SyntaxError> {
        // Bool match → Guard
        if arms.len() == 2 {
            let (true_arm, false_arm) = if matches!(arms[0].pattern, ast::Pattern::BoolLit(true, _)) {
                (&arms[0], &arms[1])
            } else if matches!(arms[1].pattern, ast::Pattern::BoolLit(true, _)) {
                (&arms[1], &arms[0])
            } else {
                return self.lower_general_match(scrutinee, arms);
            };
            let pred_id = self.lower_expr(scrutinee)?;
            let then_id = self.lower_expr(&true_arm.body)?;
            let else_id = self.lower_expr(&false_arm.body)?;
            let t = self.int_type();
            return Ok(self.insert_node(Node { id: NodeId(0), kind: NodeKind::Guard,
                type_sig: t, cost: CostTerm::Unit, arity: 0, resolution_depth: 0, salt: 0,
                payload: NodePayload::Guard { predicate_node: pred_id, body_node: then_id,
                    fallback_node: else_id } }));
        }
        self.lower_general_match(scrutinee, arms)
    }

    fn lower_general_match(&mut self, scrutinee: &Expr, arms: &[ast::MatchArm]) -> Result<NodeId, SyntaxError> {
        let scrutinee_id = self.lower_expr(scrutinee)?;

        // For constructor patterns, we need to wrap arm bodies in lambdas
        // so the evaluator can bind the inner value.
        let mut arm_ids = Vec::new();
        for arm in arms {
            let body_id = match &arm.pattern {
                ast::Pattern::Constructor(_, Some(inner), _) => {
                    // Constructor with binding: wrap body in a lambda
                    let binder_name = match inner.as_ref() {
                        ast::Pattern::Ident(n, _) => n.clone(),
                        ast::Pattern::Wildcard(_) => "_ctor_inner".to_string(),
                        _ => "_ctor_inner".to_string(),
                    };
                    self.push_scope();
                    let ref_index = (0x80 + self.lambda_depth) as u8;
                    let binder_id = BinderId(0xFFFF_0000 + ref_index as u32);
                    self.lambda_depth += 1;
                    self.bind(binder_name, Binding::LambdaParam(ref_index));

                    // For nested constructor patterns (e.g. Some(Some(x))),
                    // the inner binding would need further destructuring.
                    // For now, only one level of binding is supported.

                    let raw_body = self.lower_expr(&arm.body)?;
                    // If there's a guard, wrap: if guard then body else <fall_through>
                    let body_with_guard = self.wrap_guard(&arm.guard, raw_body)?;
                    self.pop_scope();
                    self.lambda_depth -= 1;
                    let t = self.int_type();
                    let lambda_id = self.insert_node(Node {
                        id: NodeId(0), kind: NodeKind::Lambda, type_sig: t,
                        cost: CostTerm::Unit, arity: 1, resolution_depth: 0, salt: 0,
                        payload: NodePayload::Lambda { binder: binder_id, captured_count: 0 },
                    });
                    self.add_edge(lambda_id, body_with_guard, 0, EdgeLabel::Continuation);
                    lambda_id
                }
                ast::Pattern::Tuple(pats, _) => {
                    // Tuple destructuring: bind each element by position.
                    // Generate projections first, then bind them in scope.
                    let bindings: Vec<(String, NodeId)> = pats.iter().enumerate()
                        .filter_map(|(idx, pat)| {
                            if let ast::Pattern::Ident(n, _) = pat {
                                let proj_id = self.lower_tuple_access_node(scrutinee_id, idx);
                                Some((n.clone(), proj_id))
                            } else {
                                None // Wildcard: skip
                            }
                        })
                        .collect();
                    self.push_scope();
                    for (name, node_id) in bindings {
                        self.bind(name, Binding::Node(node_id));
                    }
                    let raw_body = self.lower_expr(&arm.body)?;
                    let body_with_guard = self.wrap_guard(&arm.guard, raw_body)?;
                    self.pop_scope();
                    body_with_guard
                }
                _ => {
                    // Ident patterns: bind the scrutinee itself
                    if let ast::Pattern::Ident(n, _) = &arm.pattern {
                        self.push_scope();
                        self.bind(n.clone(), Binding::Node(scrutinee_id));
                        let raw_body = self.lower_expr(&arm.body)?;
                        let body_with_guard = self.wrap_guard(&arm.guard, raw_body)?;
                        self.pop_scope();
                        body_with_guard
                    } else {
                        let raw_body = self.lower_expr(&arm.body)?;
                        self.wrap_guard(&arm.guard, raw_body)?
                    }
                }
            };
            arm_ids.push(body_id);
        }

        let arm_patterns: Vec<u8> = arms.iter().map(|arm| match &arm.pattern {
            ast::Pattern::IntLit(v, _) => *v as u8,
            ast::Pattern::BoolLit(v, _) => *v as u8,
            ast::Pattern::Constructor(name, _, _) => {
                if let Some(Binding::Constructor(tag, _)) = self.resolve(name) {
                    *tag as u8
                } else {
                    0xFF
                }
            }
            ast::Pattern::Wildcard(_) | ast::Pattern::Ident(_, _) | ast::Pattern::Tuple(_, _) => 0xFF,
        }).collect();
        let t = self.int_type();
        let match_id = self.insert_node(Node { id: NodeId(0), kind: NodeKind::Match,
            type_sig: t, cost: CostTerm::Unit, arity: (arm_ids.len() + 1) as u8,
            resolution_depth: 0, salt: 0, payload: NodePayload::Match {
                arm_count: arm_ids.len() as u16, arm_patterns } });
        self.add_edge(match_id, scrutinee_id, 0, EdgeLabel::Scrutinee);
        for (port, arm_id) in arm_ids.iter().enumerate() {
            self.add_edge(match_id, *arm_id, (port + 1) as u8, EdgeLabel::Argument);
        }
        Ok(match_id)
    }

    /// Wrap a body node with a guard expression: `if guard then body else Unit`.
    /// Returns body unchanged if guard is None.
    fn wrap_guard(&mut self, guard: &Option<ast::Expr>, body_id: NodeId) -> Result<NodeId, SyntaxError> {
        match guard {
            None => Ok(body_id),
            Some(guard_expr) => {
                let pred_id = self.lower_expr(guard_expr)?;
                let t = self.int_type();
                let unit_id = self.insert_node(Node {
                    id: NodeId(0), kind: NodeKind::Lit, type_sig: t,
                    cost: CostTerm::Unit, arity: 0, resolution_depth: 0, salt: 0,
                    payload: NodePayload::Lit { type_tag: 0xFF, value: vec![] },
                });
                let guard_id = self.insert_node(Node {
                    id: NodeId(0), kind: NodeKind::Guard, type_sig: t,
                    cost: CostTerm::Unit, arity: 0, resolution_depth: 0, salt: 0,
                    payload: NodePayload::Guard {
                        predicate_node: pred_id,
                        body_node: body_id,
                        fallback_node: unit_id,
                    },
                });
                Ok(guard_id)
            }
        }
    }

    /// Create a Project node for tuple access: scrutinee.field_index
    fn lower_tuple_access_node(&mut self, tuple_id: NodeId, field: usize) -> NodeId {
        let t = self.int_type();
        let proj_id = self.insert_node(Node {
            id: NodeId(0), kind: NodeKind::Project, type_sig: t,
            cost: CostTerm::Unit, arity: 1, resolution_depth: 0, salt: 0,
            payload: NodePayload::Project { field_index: field as u16 },
        });
        self.add_edge(proj_id, tuple_id, 0, EdgeLabel::Argument);
        proj_id
    }

    fn lower_pipe(&mut self, lhs: &Expr, rhs: &Expr) -> Result<NodeId, SyntaxError> {
        // x |> f  →  f x
        let app = Expr::App(
            Box::new(rhs.clone()),
            Box::new(lhs.clone()),
            lhs.span().merge(rhs.span()),
        );
        self.lower_expr(&app)
    }
}

/// Resolve an effect name to its tag byte, or None if not an effect.
fn resolve_effect_name(name: &str) -> Option<u8> {
    match name {
        // Legacy (0x00-0x0D)
        "print" => Some(0x00),
        "read_line" => Some(0x01),
        "sleep" => Some(0x08),
        "timestamp" => Some(0x09),
        "random" => Some(0x0A),
        "log" => Some(0x0B),
        "send_message" => Some(0x0C),
        "recv_message" => Some(0x0D),
        // Network (0x10-0x15)
        "tcp_connect" => Some(0x10),
        "tcp_read" => Some(0x11),
        "tcp_write" => Some(0x12),
        "tcp_close" => Some(0x13),
        "tcp_listen" => Some(0x14),
        "tcp_accept" => Some(0x15),
        // Filesystem (0x16-0x1B)
        "file_open" => Some(0x16),
        "file_read_bytes" => Some(0x17),
        "file_write_bytes" => Some(0x18),
        "file_close" => Some(0x19),
        "file_stat" => Some(0x1A),
        "dir_list" => Some(0x1B),
        // System (0x1C-0x1F)
        "env_get" => Some(0x1C),
        "clock_ns" => Some(0x1D),
        "random_bytes" => Some(0x1E),
        "sleep_ms" => Some(0x1F),
        // Threading / atomic (0x20-0x28)
        "thread_spawn" => Some(0x20),
        "thread_join" => Some(0x21),
        "atomic_read" => Some(0x22),
        "atomic_write" => Some(0x23),
        "atomic_swap" => Some(0x24),
        "atomic_add" => Some(0x25),
        "rwlock_read" => Some(0x26),
        "rwlock_write" => Some(0x27),
        "rwlock_release" => Some(0x28),
        // JIT primitives (0x29-0x2A)
        "mmap_exec" => Some(0x29),
        "call_native" => Some(0x2A),
        // FFI (0x2B)
        "ffi_call" => Some(0x2B),
        _ => None,
    }
}

fn lower_cost(cost: &CostExpr) -> CostBound {
    match cost {
        CostExpr::Unknown => CostBound::Unknown,
        CostExpr::Zero => CostBound::Zero,
        CostExpr::Constant(v) => CostBound::Constant(*v),
        CostExpr::Linear(v) => CostBound::Linear(CostVar(var_hash(v))),
        CostExpr::NLogN(v) => CostBound::NLogN(CostVar(var_hash(v))),
        CostExpr::Polynomial(v, d) => CostBound::Polynomial(CostVar(var_hash(v)), *d),
        CostExpr::Sum(a, b) => CostBound::Sum(Box::new(lower_cost(a)), Box::new(lower_cost(b))),
    }
}

fn var_hash(name: &str) -> u32 {
    let h = blake3::hash(name.as_bytes());
    u32::from_le_bytes(h.as_bytes()[..4].try_into().unwrap())
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

pub fn compile_module(module: &Module) -> CompileResult {
    compile_module_with_path(module, None, &mut HashSet::new())
}

/// Compile a module with path resolution support.
/// `source_dir` is the directory of the file being compiled (for relative path resolution).
/// `compiling` tracks files currently on the compile stack for cycle detection.
pub fn compile_module_with_path(
    module: &Module,
    source_dir: Option<&Path>,
    compiling: &mut HashSet<PathBuf>,
) -> CompileResult {
    let mut fragments = Vec::new();
    let mut errors = Vec::new();
    let mut imports: BTreeMap<String, FragmentId> = BTreeMap::new();
    let mut constructors: BTreeMap<String, (u16, TypeId)> = BTreeMap::new();
    let mut adt_types: BTreeMap<TypeId, TypeDef> = BTreeMap::new();

    // Resolve imports (both hash-based and path-based)
    for item in &module.items {
        if let Item::Import(imp) = item {
            match &imp.source {
                ImportSource::Hash(hash) => {
                    imports.insert(imp.name.clone(), hex_to_fragment_id(hash));
                }
                ImportSource::Path(path) => {
                    match resolve_path_import(path, source_dir, compiling) {
                        Ok(imported) => {
                            // Register all fragments from the imported module
                            for (name, frag, _) in &imported.fragments {
                                imports.insert(name.clone(), frag.id);
                            }
                            // Merge the last fragment as the import binding name
                            if let Some((_, frag, _)) = imported.fragments.last() {
                                imports.insert(imp.name.clone(), frag.id);
                            }
                            // Merge type declarations and constructors from imported module
                            for (tid, tdef) in &imported.adt_types {
                                adt_types.entry(*tid).or_insert_with(|| tdef.clone());
                            }
                            for (cname, cbinding) in &imported.constructors {
                                constructors.entry(cname.clone()).or_insert(*cbinding);
                            }
                            // Collect all fragments for the caller
                            fragments.extend(imported.fragments);
                        }
                        Err(e) => errors.push(e),
                    }
                }
            }
        }
    }

    // Collect all ADT definitions for parametric type monomorphization
    let mut adt_defs: BTreeMap<String, ast::TypeDecl> = BTreeMap::new();
    for item in &module.items {
        if let Item::TypeDecl(td) = item {
            if !td.type_params.is_empty() {
                adt_defs.insert(td.name.clone(), td.clone());
            }
        }
    }

    // Collect class declarations for instance resolution
    let mut classes: BTreeMap<String, ast::ClassDecl> = BTreeMap::new();
    for item in &module.items {
        if let Item::ClassDecl(cd) = item {
            classes.insert(cd.name.clone(), cd.clone());
        }
    }

    // Register type declarations (constructors, record fields, and type aliases)
    let mut record_fields: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut type_aliases: BTreeMap<String, TypeExpr> = BTreeMap::new();

    // Register class dictionaries as record types
    for (class_name, cd) in &classes {
        let dict_name = format!("{}Dict", class_name);
        let method_names: Vec<String> = cd.methods.iter().map(|m| m.name.clone()).collect();
        record_fields.insert(dict_name, method_names);
    }

    for item in &module.items {
        if let Item::TypeDecl(td) = item {
            if let TypeExpr::Sum(variants, _) = &td.def {
                let mut ctx = LowerCtx::new();
                ctx.adt_defs = adt_defs.clone();
                let sum_type_id = ctx.lower_type_expr(&td.def);
                for (tid, tdef) in &ctx.type_env.types {
                    adt_types.insert(*tid, tdef.clone());
                }
                for (i, (vname, _payload)) in variants.iter().enumerate() {
                    constructors.insert(vname.clone(), (i as u16, sum_type_id));
                }
            } else if let TypeExpr::Record(fields, _) = &td.def {
                let names: Vec<String> = fields.iter().map(|(n, _)| n.clone()).collect();
                record_fields.insert(td.name.clone(), names);
            } else if matches!(&td.def, TypeExpr::RecordMerge(_, _, _)) {
                match resolve_record_merge(&td.def, &record_fields) {
                    Ok(names) => { record_fields.insert(td.name.clone(), names); }
                    Err(e) => errors.push(e),
                }
            } else {
                // Not a Sum, Record, or RecordMerge: treat as a structural type alias.
                // e.g., `type UserId = Int` or `type Pair = (Int, Int)`
                type_aliases.insert(td.name.clone(), td.def.clone());
            }
        }
    }

    for item in &module.items {
        match item {
            Item::LetDecl(decl) => {
                match compile_fn(decl, &imports, &constructors, &adt_types, &record_fields, &adt_defs, &type_aliases) {
                    Ok((fragment, smap)) => {
                        imports.insert(decl.name.clone(), fragment.id);
                        fragments.push((decl.name.clone(), fragment, smap));
                    }
                    Err(e) => errors.push(e),
                }
            }
            Item::MutualRecGroup(decls) => {
                // Pre-register all names with placeholder FragmentIds so that
                // each function body can reference its siblings.
                let placeholder = FragmentId([0xFE; 32]);
                for decl in decls {
                    imports.insert(decl.name.clone(), placeholder);
                }
                // Compile each function. Because `compile_fn` already handles
                // `decl.recursive` by binding the function's own name as a
                // placeholder, each function can reference itself AND its
                // siblings (which are in `imports` as placeholders).
                let mut group_fragments = Vec::new();
                let mut group_ok = true;
                for decl in decls {
                    match compile_fn(decl, &imports, &constructors, &adt_types, &record_fields, &adt_defs, &type_aliases) {
                        Ok((fragment, smap)) => {
                            group_fragments.push((decl.name.clone(), fragment, smap));
                        }
                        Err(e) => { errors.push(e); group_ok = false; }
                    }
                }
                // Register real fragment IDs for downstream declarations.
                if group_ok {
                    for (name, frag, _) in &group_fragments {
                        imports.insert(name.clone(), frag.id);
                    }
                    fragments.extend(group_fragments);
                }
            }
            Item::ClassDecl(_) => {} // Already processed in Phase 3
            Item::InstanceDecl(inst) => {
                // Desugar instance to a synthetic LetDecl with a RecordLit body.
                // instance Eq<Int> where eq = \a b -> a == b
                // becomes:
                // let Eq_Int_dict = { eq = \a b -> a == b }
                let type_name = match &inst.type_arg {
                    TypeExpr::Named(n, _) => n.clone(),
                    _ => "Unknown".to_string(),
                };
                let dict_name = format!("{}_{}_dict", inst.class_name, type_name);

                // Build method list: instance methods + class defaults for missing ones
                let mut method_exprs: Vec<(String, Box<Expr>)> = Vec::new();
                if let Some(cd) = classes.get(&inst.class_name) {
                    for class_method in &cd.methods {
                        // Check if instance provides this method
                        let impl_expr = inst.methods.iter()
                            .find(|(n, _)| n == &class_method.name)
                            .map(|(_, e)| e.clone())
                            .or_else(|| class_method.default_impl.clone());
                        if let Some(expr) = impl_expr {
                            method_exprs.push((class_method.name.clone(), Box::new(expr)));
                        }
                    }
                } else {
                    // Unknown class — just use the instance methods directly
                    for (name, expr) in &inst.methods {
                        method_exprs.push((name.clone(), Box::new(expr.clone())));
                    }
                }

                let body = Expr::RecordLit(method_exprs, inst.span);
                let decl = ast::LetDecl {
                    name: dict_name.clone(),
                    params: vec![],
                    ret_type: None,
                    cost: None,
                    requires: vec![],
                    ensures: vec![],
                    body,
                    span: inst.span,
                    recursive: false,
                };
                match compile_fn(&decl, &imports, &constructors, &adt_types, &record_fields, &adt_defs, &type_aliases) {
                    Ok((fragment, smap)) => {
                        imports.insert(dict_name.clone(), fragment.id);
                        fragments.push((dict_name, fragment, smap));
                    }
                    Err(e) => errors.push(e),
                }
            }
            _ => {}
        }
    }
    CompileResult { fragments, errors, constructors, adt_types }
}

/// Flatten a `RecordMerge` chain into a single list of field names.
/// Resolves named type references via `record_fields` and errors on duplicate field names.
fn resolve_record_merge(
    te: &TypeExpr,
    record_fields: &BTreeMap<String, Vec<String>>,
) -> Result<Vec<String>, SyntaxError> {
    let mut entries: Vec<(String, Span)> = Vec::new();
    collect_merge_names(te, record_fields, &mut entries)?;
    // Check for duplicate field names
    let mut seen: BTreeMap<String, Span> = BTreeMap::new();
    for (name, span) in &entries {
        if let Some(first_span) = seen.get(name) {
            return Err(SyntaxError::new(
                format!(
                    "duplicate field '{}' in record composition (first defined at offset {})",
                    name, first_span.start
                ),
                *span,
            ));
        }
        seen.insert(name.clone(), *span);
    }
    Ok(entries.into_iter().map(|(n, _)| n).collect())
}

/// Collect (field_name, span) pairs from a RecordMerge chain.
fn collect_merge_names(
    te: &TypeExpr,
    record_fields: &BTreeMap<String, Vec<String>>,
    out: &mut Vec<(String, Span)>,
) -> Result<(), SyntaxError> {
    match te {
        TypeExpr::Record(fields, span) => {
            for (name, _) in fields {
                out.push((name.clone(), *span));
            }
            Ok(())
        }
        TypeExpr::RecordMerge(lhs, rhs, _) => {
            collect_merge_names(lhs, record_fields, out)?;
            collect_merge_names(rhs, record_fields, out)?;
            Ok(())
        }
        TypeExpr::Named(name, span) => {
            match record_fields.get(name) {
                Some(fields) => {
                    for f in fields {
                        out.push((f.clone(), *span));
                    }
                    Ok(())
                }
                None => Err(SyntaxError::new(
                    format!("'{}' is not a record type (cannot use in /)", name),
                    *span,
                )),
            }
        }
        _ => Err(SyntaxError::new(
            "record composition (/) requires record types or named record types".to_string(),
            te.span(),
        )),
    }
}

/// Resolve a path-based import by compiling the target file.
fn resolve_path_import(
    import_path: &str,
    source_dir: Option<&Path>,
    compiling: &mut HashSet<PathBuf>,
) -> Result<CompileResult, SyntaxError> {
    use crate::syntax::lexer::Lexer;
    use crate::syntax::parser::Parser;

    // Resolve relative to source directory, or CWD
    let resolved = if let Some(dir) = source_dir {
        dir.join(import_path)
    } else {
        PathBuf::from(import_path)
    };

    let canonical = resolved.canonicalize().map_err(|e| {
        SyntaxError::new(format!("import path not found: {}: {}", import_path, e), Span::new(0, 0))
    })?;

    // Cycle detection
    if compiling.contains(&canonical) {
        return Err(SyntaxError::new(
            format!("circular import detected: {}", import_path),
            Span::new(0, 0),
        ));
    }

    compiling.insert(canonical.clone());

    let source = std::fs::read_to_string(&canonical).map_err(|e| {
        SyntaxError::new(format!("cannot read {}: {}", import_path, e), Span::new(0, 0))
    })?;

    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module()?;

    let import_dir = canonical.parent().map(|p| p.to_path_buf());
    let result = compile_module_with_path(
        &module,
        import_dir.as_deref(),
        compiling,
    );

    compiling.remove(&canonical);

    if result.errors.is_empty() {
        Ok(result)
    } else {
        Err(SyntaxError::new(
            format!("errors in imported file {}: {:?}", import_path, result.errors),
            Span::new(0, 0),
        ))
    }
}

pub fn compile_fn(
    decl: &LetDecl,
    imports: &BTreeMap<String, FragmentId>,
    constructors: &BTreeMap<String, (u16, TypeId)>,
    adt_types: &BTreeMap<TypeId, TypeDef>,
    record_fields: &BTreeMap<String, Vec<String>>,
    parametric_adt_defs: &BTreeMap<String, ast::TypeDecl>,
    type_aliases: &BTreeMap<String, TypeExpr>,
) -> Result<(Fragment, SourceMap), SyntaxError> {
    let mut ctx = LowerCtx::new();
    ctx.record_fields = record_fields.clone();
    ctx.adt_defs = parametric_adt_defs.clone();
    ctx.type_aliases = type_aliases.clone();
    // Pre-register ADT type definitions so they appear in the fragment's type_env
    for (tid, tdef) in adt_types {
        ctx.type_env.types.entry(*tid).or_insert_with(|| tdef.clone());
    }
    for (name, fid) in imports {
        ctx.bind(name.clone(), Binding::Fragment(*fid));
    }
    for (name, (tag, sum_type)) in constructors {
        ctx.bind(name.clone(), Binding::Constructor(*tag, *sum_type));
    }

    // Compute param types from annotation BEFORE binding params
    let (input_types, output_type) = if let Some(ref type_expr) = decl.ret_type {
        ctx.decompose_arrow(type_expr, decl.params.len())
    } else {
        let int_type = compute_type_id(&TypeDef::Primitive(PrimType::Int));
        (vec![int_type; decl.params.len()], int_type)
    };

    for (i, param) in decl.params.iter().enumerate() {
        ctx.bind(param.clone(), Binding::InputRef(i as u8, input_types[i]));
    }

    // For recursive declarations, bind the function's own name so the body
    // can reference it.  We use a placeholder FragmentId that will be
    // overwritten once the real hash is computed.
    if decl.recursive {
        let placeholder = FragmentId([0xFFu8; 32]);
        ctx.bind(decl.name.clone(), Binding::Fragment(placeholder));
    }

    let root_id = ctx.lower_expr(&decl.body)?;
    let cost = decl.cost.as_ref().map(|c| lower_cost(c)).unwrap_or(CostBound::Unknown);
    let hash = compute_semantic_hash(&ctx.nodes, &ctx.edges, root_id);
    let source_map = ctx.source_map.clone();

    // Contract lowering: convert requires/ensures AST exprs to LIA formulas
    let var_map: Vec<(String, u32)> = decl.params.iter().enumerate()
        .map(|(i, name)| (name.clone(), i as u32))
        .collect();
    let requires: Vec<LIAFormula> = decl.requires.iter()
        .map(|e| ctx.lower_contract_expr(e, &var_map))
        .collect();
    let ensures: Vec<LIAFormula> = decl.ensures.iter()
        .map(|e| ctx.lower_contract_expr(e, &var_map))
        .collect();
    let contracts = FragmentContracts { requires, ensures };

    let mut graph = SemanticGraph {
        root: root_id, nodes: ctx.nodes, edges: ctx.edges,
        type_env: ctx.type_env.clone(), cost: cost.clone(), resolution: Resolution::Implementation, hash,
    };

    // Propagate the user's cost annotation to the root node so the checker
    // can verify it against the proven cost.  Without this, the root node
    // carries CostTerm::Unit and the per-node cost check never fires.
    if !matches!(cost, CostBound::Unknown) {
        if let Some(root_node) = graph.nodes.get_mut(&root_id) {
            root_node.cost = CostTerm::Annotated(cost);
        }
    }

    // Run type inference pass to propagate types through composite nodes
    infer_types(&mut graph);

    let inputs: Vec<(NodeId, TypeId)> = (0..decl.params.len())
        .map(|i| (root_id, input_types[i]))
        .collect();
    let outputs = vec![(root_id, output_type)];

    let mut fragment = Fragment {
        id: FragmentId([0u8; 32]), graph, boundary: Boundary { inputs, outputs },
        type_env: ctx.type_env, imports: imports.values().copied().collect(),
        metadata: FragmentMeta { name: Some(decl.name.clone()), created_at: 0, generation: 0, lineage_hash: 0 },
        proof: None, contracts,
    };
    fragment.id = iris_types::hash::compute_fragment_id(&fragment);
    Ok((fragment, source_map))
}

// ---------------------------------------------------------------------------
// Post-lowering type inference: propagate types bottom-up through the graph
// ---------------------------------------------------------------------------

/// Propagate types bottom-up through a SemanticGraph.
/// Leaves (Lit, Prim) already carry correct types from the lowerer.
/// This pass infers types for composite nodes (Apply, Guard, Let, Fold, etc.)
/// by examining their children's types via the edge graph.
pub fn infer_types(graph: &mut SemanticGraph) {
    // Build adjacency: for each node, collect its input edges sorted by port
    let mut children: HashMap<NodeId, Vec<(u8, NodeId)>> = HashMap::new();
    for edge in &graph.edges {
        children.entry(edge.source).or_default().push((edge.port, edge.target));
    }
    for v in children.values_mut() {
        v.sort_by_key(|(port, _)| *port);
    }

    // Topological sort (children before parents)
    let topo = topological_sort_nodes(&graph.nodes, &graph.edges, graph.root);

    let int_type = compute_type_id(&TypeDef::Primitive(PrimType::Int));
    let _bool_type = compute_type_id(&TypeDef::Primitive(PrimType::Bool));

    for node_id in &topo {
        let kind = match graph.nodes.get(node_id) {
            Some(n) => n.kind,
            None => continue,
        };

        let child_types: Vec<TypeId> = children.get(node_id)
            .map(|cs| cs.iter().map(|(_, cid)| {
                graph.nodes.get(cid).map(|n| n.type_sig).unwrap_or(int_type)
            }).collect())
            .unwrap_or_default();

        // For payload-linked nodes, read child types from payload fields
        let payload_child_types: Option<Vec<TypeId>> = match &graph.nodes.get(node_id).unwrap().payload {
            NodePayload::Guard { body_node, fallback_node, .. } => {
                let body_t = graph.nodes.get(body_node).map(|n| n.type_sig).unwrap_or(int_type);
                let _fallback_t = graph.nodes.get(fallback_node).map(|n| n.type_sig).unwrap_or(int_type);
                Some(vec![body_t])
            }
            _ => None,
        };

        let inferred = match kind {
            // Leaves: already typed correctly by the lowerer
            NodeKind::Lit | NodeKind::Prim => continue,

            // Guard (if/else): type is the type of the then-branch (from payload)
            NodeKind::Guard => {
                if let Some(ref pcts) = payload_child_types {
                    pcts.first().copied().unwrap_or(int_type)
                } else {
                    child_types.get(1).copied().unwrap_or(int_type)
                }
            }

            // Apply: if the function child has Arrow type, result is the return type
            NodeKind::Apply => {
                if let Some(&fn_type_id) = child_types.first() {
                    match graph.type_env.types.get(&fn_type_id) {
                        Some(TypeDef::Arrow(_, ret, _)) => *ret,
                        Some(TypeDef::ForAll(_, inner_id)) => {
                            // Unwrap ForAll to get inner Arrow's return type.
                            match graph.type_env.types.get(inner_id) {
                                Some(TypeDef::Arrow(_, ret, _)) => *ret,
                                _ => continue,
                            }
                        }
                        _ => continue,
                    }
                } else {
                    continue;
                }
            }

            // Lambda: Arrow(param_type, body_type)
            NodeKind::Lambda => {
                if let Some(&body_type) = child_types.last() {
                    let param_type = int_type; // Lambda params default to Int
                    let arrow = TypeDef::Arrow(param_type, body_type, CostBound::Unknown);
                    let arrow_id = compute_type_id(&arrow);
                    graph.type_env.types.entry(arrow_id).or_insert(arrow);
                    arrow_id
                } else {
                    continue;
                }
            }

            // Let/LetRec: type is the body (last child).
            // If the bound value has a ForAll type, it's preserved for the body.
            NodeKind::Let | NodeKind::LetRec => {
                child_types.last().copied().unwrap_or(int_type)
            }

            // Fold: type depends on accumulator (first child)
            NodeKind::Fold => {
                child_types.first().copied().unwrap_or(int_type)
            }

            // Unfold: type depends on seed (first child)
            NodeKind::Unfold => {
                child_types.first().copied().unwrap_or(int_type)
            }

            // Match: type of first arm body
            NodeKind::Match => {
                // Match arms connect at ports 1, 2, 3, ...
                child_types.get(1).copied().unwrap_or(int_type)
            }

            // Tuple: Product of child types
            NodeKind::Tuple => {
                if !child_types.is_empty() {
                    let prod = TypeDef::Product(child_types.clone());
                    let prod_id = compute_type_id(&prod);
                    graph.type_env.types.entry(prod_id).or_insert(prod);
                    prod_id
                } else {
                    continue;
                }
            }

            // Project: extract from tuple — hard to know which element, keep Int
            NodeKind::Project => continue,

            // Ref: cross-fragment call — keep existing type
            NodeKind::Ref => continue,

            // Effect: result depends on effect — keep Int default
            NodeKind::Effect => continue,

            // Neural: keep existing
            NodeKind::Neural => continue,

            // Anything else: skip
            _ => continue,
        };

        if let Some(node) = graph.nodes.get_mut(node_id) {
            node.type_sig = inferred;
        }
    }
}

/// Topological sort of nodes (leaves first, root last).
fn topological_sort_nodes(
    nodes: &HashMap<NodeId, Node>,
    edges: &[Edge],
    _root: NodeId,
) -> Vec<NodeId> {
    let mut in_degree: HashMap<NodeId, usize> = nodes.keys().map(|id| (*id, 0)).collect();
    let mut rev_adj: HashMap<NodeId, Vec<NodeId>> = HashMap::new();

    for edge in edges {
        *in_degree.entry(edge.source).or_insert(0) += 1;
        rev_adj.entry(edge.target).or_default().push(edge.source);
    }

    // Start with leaves (no incoming edges from perspective of data flow)
    let mut queue: Vec<NodeId> = in_degree.iter()
        .filter(|&(_, deg)| *deg == 0)
        .map(|(id, _)| *id)
        .collect();
    queue.sort(); // deterministic order

    let mut result = Vec::with_capacity(nodes.len());
    while let Some(nid) = queue.pop() {
        result.push(nid);
        if let Some(parents) = rev_adj.get(&nid) {
            for parent in parents {
                if let Some(deg) = in_degree.get_mut(parent) {
                    *deg = deg.saturating_sub(1);
                    if *deg == 0 {
                        queue.push(*parent);
                    }
                }
            }
        }
    }

    // Add any unreachable nodes (shouldn't happen in well-formed graphs)
    for id in nodes.keys() {
        if !result.contains(id) {
            result.push(*id);
        }
    }

    result
}

fn compute_semantic_hash(nodes: &HashMap<NodeId, Node>, edges: &[Edge], root: NodeId) -> SemanticHash {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&root.0.to_le_bytes());
    hasher.update(&(nodes.len() as u64).to_le_bytes());
    { let mut sorted: Vec<_> = nodes.iter().collect(); sorted.sort_by_key(|(id, _)| *id); for (nid, node) in sorted { hasher.update(&nid.0.to_le_bytes()); hasher.update(&[node.kind as u8]); } }
    hasher.update(&(edges.len() as u64).to_le_bytes());
    for edge in edges { hasher.update(&edge.source.0.to_le_bytes()); hasher.update(&edge.target.0.to_le_bytes()); hasher.update(&[edge.port, edge.label as u8]); }
    SemanticHash(*hasher.finalize().as_bytes())
}

fn hex_to_fragment_id(hex: &str) -> FragmentId {
    let mut bytes = [0u8; 32];
    let hex_bytes: Vec<u8> = (0..hex.len()).step_by(2)
        .filter_map(|i| if i + 2 <= hex.len() { u8::from_str_radix(&hex[i..i + 2], 16).ok() } else { None })
        .collect();
    let len = hex_bytes.len().min(32);
    bytes[..len].copy_from_slice(&hex_bytes[..len]);
    FragmentId(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::lexer::Lexer;
    use crate::syntax::parser::Parser;

    fn compile_str(src: &str) -> CompileResult {
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let module = parser.parse_module().unwrap();
        compile_module(&module)
    }

    #[test]
    fn test_compile_sum() {
        let result = compile_str("let sum xs = fold 0 (+) xs");
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        assert_eq!(result.fragments.len(), 1);
        let (name, frag, _) = &result.fragments[0];
        assert_eq!(name, "sum");
        let root = &frag.graph.nodes[&frag.graph.root];
        assert_eq!(root.kind, NodeKind::Fold);
    }

    #[test]
    fn test_compile_binop() {
        let result = compile_str("let add2 x y = x + y");
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        let (_, frag, _) = &result.fragments[0];
        let root = &frag.graph.nodes[&frag.graph.root];
        assert_eq!(root.kind, NodeKind::Prim);
    }

    #[test]
    fn test_compile_dot_product() {
        let result = compile_str("let dot xs ys = fold 0 (+) (map (*) (zip xs ys))");
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        let (_, frag, _) = &result.fragments[0];
        let root = &frag.graph.nodes[&frag.graph.root];
        assert_eq!(root.kind, NodeKind::Fold);
    }

    #[test]
    fn test_compile_if_then_else() {
        let result = compile_str("let safe x y = if y != 0 then x / y else 0");
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        let (_, frag, _) = &result.fragments[0];
        let root = &frag.graph.nodes[&frag.graph.root];
        assert_eq!(root.kind, NodeKind::Guard);
    }

    #[test]
    fn test_compile_lambda() {
        let result = compile_str("let f xs = fold 0 (\\acc x -> acc + x) xs");
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        let (_, frag, _) = &result.fragments[0];
        let has_lambda = frag.graph.nodes.values().any(|n| n.kind == NodeKind::Lambda);
        assert!(has_lambda);
    }

    #[test]
    fn test_compile_let_in() {
        let result = compile_str(
            "let f xs ys = let pairs = zip xs ys in fold 0 (+) (map (*) pairs)");
        assert!(result.errors.is_empty(), "{:?}", result.errors);
    }

    #[test]
    fn test_compile_pipe() {
        let result = compile_str("let f x = x |> neg");
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        let (_, frag, _) = &result.fragments[0];
        // x |> neg desugars to neg(x) → Prim(neg) with one argument edge
        let root = &frag.graph.nodes[&frag.graph.root];
        assert_eq!(root.kind, NodeKind::Prim);
        if let NodePayload::Prim { opcode } = &root.payload {
            assert_eq!(*opcode, 0x05); // neg
        }
    }

    #[test]
    fn test_compile_pipe_chain() {
        let result = compile_str(
            "let f xs = xs |> filter (\\x -> x > 0) |> fold 0 (+)");
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        let (_, frag, _) = &result.fragments[0];
        // Final result should be a Fold (the last in the chain)
        let root = &frag.graph.nodes[&frag.graph.root];
        assert_eq!(root.kind, NodeKind::Fold);
    }

    #[test]
    fn test_compile_io_direct_call() {
        let result = compile_str("let listen port = tcp_listen port");
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        let (_, frag, _) = &result.fragments[0];
        let root = &frag.graph.nodes[&frag.graph.root];
        assert_eq!(root.kind, NodeKind::Effect);
        if let NodePayload::Effect { effect_tag } = &root.payload {
            assert_eq!(*effect_tag, 0x14);
        }
    }

    // -------------------------------------------------------------------
    // Type annotation and contract lowering tests
    // -------------------------------------------------------------------

    #[test]
    fn test_type_annotation_int_to_int() {
        let result = compile_str("let inc x : Int -> Int = x + 1");
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        let (_, frag, _) = &result.fragments[0];
        // Input should be typed as Int
        let int_id = compute_type_id(&TypeDef::Primitive(PrimType::Int));
        assert_eq!(frag.boundary.inputs.len(), 1);
        assert_eq!(frag.boundary.inputs[0].1, int_id);
        // Output should be typed as Int
        assert_eq!(frag.boundary.outputs.len(), 1);
        assert_eq!(frag.boundary.outputs[0].1, int_id);
    }

    #[test]
    fn test_type_annotation_int_to_bool() {
        let result = compile_str("let is_zero x : Int -> Bool = x == 0");
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        let (_, frag, _) = &result.fragments[0];
        let int_id = compute_type_id(&TypeDef::Primitive(PrimType::Int));
        let bool_id = compute_type_id(&TypeDef::Primitive(PrimType::Bool));
        assert_eq!(frag.boundary.inputs[0].1, int_id);
        assert_eq!(frag.boundary.outputs[0].1, bool_id);
    }

    #[test]
    fn test_type_annotation_two_params() {
        let result = compile_str("let add x y : Int -> Int -> Int = x + y");
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        let (_, frag, _) = &result.fragments[0];
        let int_id = compute_type_id(&TypeDef::Primitive(PrimType::Int));
        assert_eq!(frag.boundary.inputs.len(), 2);
        assert_eq!(frag.boundary.inputs[0].1, int_id);
        assert_eq!(frag.boundary.inputs[1].1, int_id);
        assert_eq!(frag.boundary.outputs[0].1, int_id);
    }

    #[test]
    fn test_type_annotation_arrow_stored_in_type_env() {
        let result = compile_str("let f x : Int -> Bool = x > 0");
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        let (_, frag, _) = &result.fragments[0];
        let bool_id = compute_type_id(&TypeDef::Primitive(PrimType::Bool));
        // The return type should be Bool, stored in the type_env
        assert!(frag.type_env.types.contains_key(&bool_id));
    }

    #[test]
    fn test_type_annotation_string_return() {
        let result = compile_str("let greet name : String -> String = name");
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        let (_, frag, _) = &result.fragments[0];
        let bytes_id = compute_type_id(&TypeDef::Primitive(PrimType::Bytes));
        assert_eq!(frag.boundary.inputs[0].1, bytes_id);
        assert_eq!(frag.boundary.outputs[0].1, bytes_id);
    }

    #[test]
    fn test_type_annotation_tuple_return() {
        let result = compile_str("let pair x y : Int -> Int -> (Int, Bool) = (x, y > 0)");
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        let (_, frag, _) = &result.fragments[0];
        let int_id = compute_type_id(&TypeDef::Primitive(PrimType::Int));
        let bool_id = compute_type_id(&TypeDef::Primitive(PrimType::Bool));
        let tuple_id = compute_type_id(&TypeDef::Product(vec![int_id, bool_id]));
        assert_eq!(frag.boundary.outputs[0].1, tuple_id);
    }

    #[test]
    fn test_no_type_annotation_defaults_to_int() {
        let result = compile_str("let inc x = x + 1");
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        let (_, frag, _) = &result.fragments[0];
        let int_id = compute_type_id(&TypeDef::Primitive(PrimType::Int));
        assert_eq!(frag.boundary.inputs[0].1, int_id);
        assert_eq!(frag.boundary.outputs[0].1, int_id);
    }

    #[test]
    fn test_requires_lowered_to_contracts() {
        let result = compile_str(
            "let safe_div x y : Int -> Int -> Int requires y != 0 = x / y"
        );
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        let (_, frag, _) = &result.fragments[0];
        assert_eq!(frag.contracts.requires.len(), 1);
        // y != 0 becomes Not(Eq(Var(1), Const(0)))
        match &frag.contracts.requires[0] {
            LIAFormula::Not(inner) => match inner.as_ref() {
                LIAFormula::Atom(LIAAtom::Eq(LIATerm::Var(v), LIATerm::Const(0))) => {
                    assert_eq!(v.0, 1); // y is param index 1
                }
                other => panic!("expected Eq atom, got {:?}", other),
            },
            other => panic!("expected Not(...), got {:?}", other),
        }
    }

    #[test]
    fn test_ensures_lowered_to_contracts() {
        let result = compile_str(
            "let abs x : Int -> Int ensures result >= 0 = if x >= 0 then x else 0 - x"
        );
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        let (_, frag, _) = &result.fragments[0];
        assert_eq!(frag.contracts.ensures.len(), 1);
        // result >= 0 becomes Le(Const(0), Var(0xFFFF))
        // which is Ge flipped: Le(rhs=Const(0), lhs=Var(result))
        match &frag.contracts.ensures[0] {
            LIAFormula::Atom(LIAAtom::Le(LIATerm::Const(0), LIATerm::Var(v))) => {
                assert_eq!(v.0, 0xFFFF); // result variable
            }
            other => panic!("expected Le(Const(0), Var(result)), got {:?}", other),
        }
    }

    #[test]
    fn test_multiple_requires_ensures() {
        let result = compile_str(
            "let bounded_add x y : Int -> Int -> Int \
             requires x >= 0 \
             requires y >= 0 \
             ensures result >= 0 \
             ensures result == x + y \
             = x + y"
        );
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        let (_, frag, _) = &result.fragments[0];
        assert_eq!(frag.contracts.requires.len(), 2);
        assert_eq!(frag.contracts.ensures.len(), 2);
    }

    #[test]
    fn test_no_contracts_means_empty() {
        let result = compile_str("let inc x = x + 1");
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        let (_, frag, _) = &result.fragments[0];
        assert!(frag.contracts.requires.is_empty());
        assert!(frag.contracts.ensures.is_empty());
    }

    #[test]
    fn test_input_ref_carries_annotated_type() {
        // When param is typed as String, the input ref node should have Bytes type
        let result = compile_str("let echo msg : String -> String = msg");
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        let (_, frag, _) = &result.fragments[0];
        let bytes_id = compute_type_id(&TypeDef::Primitive(PrimType::Bytes));
        // The root node is an input ref for 'msg' — should carry Bytes type
        let root = &frag.graph.nodes[&frag.graph.root];
        assert_eq!(root.type_sig, bytes_id, "input ref should carry Bytes type from annotation");
    }

    #[test]
    fn test_input_ref_default_int_without_annotation() {
        let result = compile_str("let id x = x");
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        let (_, frag, _) = &result.fragments[0];
        let int_id = compute_type_id(&TypeDef::Primitive(PrimType::Int));
        let root = &frag.graph.nodes[&frag.graph.root];
        assert_eq!(root.type_sig, int_id, "unannotated input ref should default to Int");
    }

    #[test]
    fn test_bool_param_type_propagates() {
        let result = compile_str("let negate flag : Bool -> Bool = if flag then false else true");
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        let (_, frag, _) = &result.fragments[0];
        let bool_id = compute_type_id(&TypeDef::Primitive(PrimType::Bool));
        assert_eq!(frag.boundary.inputs[0].1, bool_id);
        assert_eq!(frag.boundary.outputs[0].1, bool_id);
    }

    #[test]
    fn test_guard_node_infers_branch_type() {
        // if x > 0 then "pos" else "neg" → Guard should have Bytes type
        let result = compile_str(r#"let classify x : Int -> String = if x > 0 then "pos" else "neg""#);
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        let (_, frag, _) = &result.fragments[0];
        let bytes_id = compute_type_id(&TypeDef::Primitive(PrimType::Bytes));
        let root = &frag.graph.nodes[&frag.graph.root];
        assert_eq!(root.kind, NodeKind::Guard);
        assert_eq!(root.type_sig, bytes_id, "Guard node should infer Bytes from string branches");
    }

    #[test]
    fn test_lambda_infers_arrow_type() {
        let result = compile_str("let f xs = fold 0 (\\acc x -> acc + x) xs");
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        let (_, frag, _) = &result.fragments[0];
        // Find the lambda node
        let lambda = frag.graph.nodes.values().find(|n| n.kind == NodeKind::Lambda).unwrap();
        // Lambda should have Arrow type
        if let Some(TypeDef::Arrow(_, _, _)) = frag.graph.type_env.types.get(&lambda.type_sig) {
            // Good — lambda has Arrow type
        } else {
            panic!("Lambda should have Arrow type, got type_sig={:?}", lambda.type_sig);
        }
    }

    // -----------------------------------------------------------------------
    // ADT (algebraic data type) tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_sum_type_decl() {
        let src = "type Color = Red | Green | Blue\nlet x = 0";
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let module = parser.parse_module().unwrap();
        assert_eq!(module.items.len(), 2);
        if let ast::Item::TypeDecl(td) = &module.items[0] {
            assert_eq!(td.name, "Color");
            if let ast::TypeExpr::Sum(variants, _) = &td.def {
                assert_eq!(variants.len(), 3);
                assert_eq!(variants[0].0, "Red");
                assert!(variants[0].1.is_none());
                assert_eq!(variants[1].0, "Green");
                assert_eq!(variants[2].0, "Blue");
            } else { panic!("Expected Sum type"); }
        } else { panic!("Expected TypeDecl"); }
    }

    #[test]
    fn test_parse_sum_with_payloads() {
        let src = "type Option = Some(Int) | None\nlet x = 0";
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let module = parser.parse_module().unwrap();
        if let ast::Item::TypeDecl(td) = &module.items[0] {
            if let ast::TypeExpr::Sum(variants, _) = &td.def {
                assert_eq!(variants.len(), 2);
                assert_eq!(variants[0].0, "Some");
                assert!(variants[0].1.is_some());
                assert_eq!(variants[1].0, "None");
                assert!(variants[1].1.is_none());
            } else { panic!("Expected Sum type"); }
        } else { panic!("Expected TypeDecl"); }
    }

    #[test]
    fn test_compile_adt_constructor_bare() {
        let src = "type Color = Red | Green | Blue\nlet x = Red";
        let result = compile_str(src);
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        let (_, frag, _) = &result.fragments[0];
        let root = &frag.graph.nodes[&frag.graph.root];
        assert_eq!(root.kind, NodeKind::Inject);
        if let NodePayload::Inject { tag_index } = &root.payload {
            assert_eq!(*tag_index, 0);
        } else { panic!("Expected Inject payload"); }
    }

    #[test]
    fn test_compile_adt_constructor_with_payload() {
        let src = "type Option = Some(Int) | None\nlet x = Some 42";
        let result = compile_str(src);
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        let (_, frag, _) = &result.fragments[0];
        let root = &frag.graph.nodes[&frag.graph.root];
        assert_eq!(root.kind, NodeKind::Inject);
        if let NodePayload::Inject { tag_index } = &root.payload {
            assert_eq!(*tag_index, 0);
        } else { panic!("Expected Inject payload"); }
    }

    #[test]
    fn test_compile_adt_match_constructors() {
        let src = r#"
type Option = Some(Int) | None
let unwrap_or x default_val =
    match x with
      | Some(v) -> v
      | None -> default_val
"#;
        let result = compile_str(src);
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        let (_, frag, _) = &result.fragments[0];
        let has_match = frag.graph.nodes.values().any(|n| n.kind == NodeKind::Match);
        assert!(has_match, "Expected a Match node");
    }

    #[test]
    fn test_compile_adt_sum_type_lowered() {
        let src = "type Result = Ok(Int) | Err(Int)\nlet x = Ok 1";
        let result = compile_str(src);
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        let (_, frag, _) = &result.fragments[0];
        let root = &frag.graph.nodes[&frag.graph.root];
        if let Some(TypeDef::Sum(variants)) = frag.graph.type_env.types.get(&root.type_sig) {
            assert_eq!(variants.len(), 2);
            assert_eq!(variants[0].0, Tag(0));
            assert_eq!(variants[1].0, Tag(1));
        } else {
            panic!("Inject type_sig should be Sum, got {:?}", frag.graph.type_env.types.get(&root.type_sig));
        }
    }

    #[test]
    fn test_cross_fragment_ref_call() {
        // Simplest case: one-arg function called from another
        let src = "let double x = x + x\nlet test_ok = double 21";
        let result = compile_str(src);
        assert!(result.errors.is_empty(), "compile errors: {:?}", result.errors);

        let mut registry = std::collections::BTreeMap::new();
        for (_, frag, _) in &result.fragments {
            registry.insert(frag.id, frag.graph.clone());
        }

        let (_, test_frag, _) = result.fragments.iter()
            .find(|(name, _, _)| name == "test_ok").unwrap();
        let val = crate::evaluate_with_registry(
            &test_frag.graph, &[], 100_000, &registry,
        ).expect("should evaluate to Int");
        assert_eq!(val, iris_types::eval::Value::Int(42), "double 21 should be 42");
    }

    #[test]
    fn test_cross_fragment_ref_call_lambda() {
        // Lambda-bodied function called from another
        let src = r#"
type Option = Some(Int) | None
let unwrap_or : Option -> Int -> Int =
  \opt -> \default ->
    match opt with
    | Some(v) -> v
    | None -> default
let test_ok : Int =
  unwrap_or (Some(42)) 0
"#;
        let result = compile_str(src);
        assert!(result.errors.is_empty(), "compile errors: {:?}", result.errors);

        let mut registry = std::collections::BTreeMap::new();
        for (_, frag, _) in &result.fragments {
            registry.insert(frag.id, frag.graph.clone());
        }

        let (_, test_frag, _) = result.fragments.iter()
            .find(|(name, _, _)| name == "test_ok").unwrap();
        let val = crate::evaluate_with_registry(
            &test_frag.graph, &[], 100_000, &registry,
        ).expect("should evaluate to Int");
        assert_eq!(val, iris_types::eval::Value::Int(42), "unwrap_or (Some(42)) 0 should be 42");
    }

    // -------------------------------------------------------------------
    // Record composition (/) tests
    // -------------------------------------------------------------------

    #[test]
    fn test_record_merge_basic() {
        let result = compile_str(
            "type A = { x: Int, y: Int }\n\
             type B = { z: Int }\n\
             type C = A / B\n\
             let f p : C -> Int = p.x + p.z"
        );
        assert!(result.errors.is_empty(), "{:?}", result.errors);
    }

    #[test]
    fn test_record_merge_inline_and_named() {
        let result = compile_str(
            "type Contact = { email: Int, phone: Int }\n\
             type User = { name: Int } / Contact\n\
             let f u : User -> Int = u.name + u.email + u.phone"
        );
        assert!(result.errors.is_empty(), "{:?}", result.errors);
    }

    #[test]
    fn test_record_merge_triple_chain() {
        let result = compile_str(
            "type A = { x: Int }\n\
             type B = { y: Int }\n\
             type C = { z: Int }\n\
             type All = A / B / C\n\
             let f p : All -> Int = p.x + p.y + p.z"
        );
        assert!(result.errors.is_empty(), "{:?}", result.errors);
    }

    #[test]
    fn test_record_merge_duplicate_field_error() {
        let result = compile_str(
            "type A = { x: Int }\n\
             type B = { x: Int }\n\
             type C = A / B\n\
             let f p = p.x"
        );
        assert!(!result.errors.is_empty(), "should error on duplicate field 'x'");
        let msg = &result.errors[0].message;
        assert!(msg.contains("duplicate field 'x'"),
            "error should mention duplicate field, got: {}", msg);
        assert!(msg.contains("first defined at"),
            "error should reference first definition, got: {}", msg);
    }

    #[test]
    fn test_record_merge_non_record_type_error() {
        let result = compile_str(
            "type NotARecord = Int\n\
             type Bad = { x: Int } / NotARecord\n\
             let f p = p.x"
        );
        assert!(!result.errors.is_empty(), "should error on non-record type in /");
        assert!(result.errors[0].message.contains("not a record type"),
            "error should say not a record type, got: {}", result.errors[0].message);
    }

    #[test]
    fn test_record_merge_two_inline_records() {
        let result = compile_str(
            "type Both = { a: Int } / { b: Int }\n\
             let f p : Both -> Int = p.a + p.b"
        );
        assert!(result.errors.is_empty(), "{:?}", result.errors);
    }

    // -------------------------------------------------------------------
    // Mutual recursion tests
    // -------------------------------------------------------------------

    #[test]
    fn test_mutual_rec_parses_two_functions() {
        let src = "let rec is_even n = if n == 0 then 1 else is_odd (n - 1)\n\
                   and is_odd n = if n == 0 then 0 else is_even (n - 1)";
        let result = compile_str(src);
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        assert_eq!(result.fragments.len(), 2, "expected 2 fragments for mutual rec group");
        assert_eq!(result.fragments[0].0, "is_even");
        assert_eq!(result.fragments[1].0, "is_odd");
    }

    #[test]
    fn test_mutual_rec_three_functions() {
        let src = "let rec a x = b x\n\
                   and b x = c x\n\
                   and c x = a x";
        let result = compile_str(src);
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        assert_eq!(result.fragments.len(), 3);
        assert_eq!(result.fragments[0].0, "a");
        assert_eq!(result.fragments[1].0, "b");
        assert_eq!(result.fragments[2].0, "c");
    }

    #[test]
    fn test_mutual_rec_contains_ref_nodes() {
        // is_even calls is_odd and vice versa: both should produce Ref nodes
        let src = "let rec is_even n = if n == 0 then 1 else is_odd (n - 1)\n\
                   and is_odd n = if n == 0 then 0 else is_even (n - 1)";
        let result = compile_str(src);
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        // is_even's fragment should contain a Ref to is_odd
        let (_, even_frag, _) = &result.fragments[0];
        let has_ref = even_frag.graph.nodes.values().any(|n| n.kind == NodeKind::Ref);
        assert!(has_ref, "is_even should contain a Ref node for calling is_odd");
        // is_odd's fragment should contain a Ref to is_even
        let (_, odd_frag, _) = &result.fragments[1];
        let has_ref = odd_frag.graph.nodes.values().any(|n| n.kind == NodeKind::Ref);
        assert!(has_ref, "is_odd should contain a Ref node for calling is_even");
    }

    #[test]
    fn test_mutual_rec_followed_by_regular() {
        // Mutual rec group followed by a regular let decl
        let src = "let rec f x = g x\n\
                   and g x = f x\n\
                   let h x = x + 1";
        let result = compile_str(src);
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        assert_eq!(result.fragments.len(), 3);
        assert_eq!(result.fragments[0].0, "f");
        assert_eq!(result.fragments[1].0, "g");
        assert_eq!(result.fragments[2].0, "h");
    }

    #[test]
    fn test_single_rec_no_and() {
        // Single `let rec` without `and` should NOT produce a MutualRecGroup
        let src = "let rec fac n = if n == 0 then 1 else n * fac (n - 1)";
        let result = compile_str(src);
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        assert_eq!(result.fragments.len(), 1);
        assert_eq!(result.fragments[0].0, "fac");
    }

    #[test]
    fn test_mutual_rec_with_type_annotations() {
        let src = "let rec is_even n : Int -> Int = if n == 0 then 1 else is_odd (n - 1)\n\
                   and is_odd n : Int -> Int = if n == 0 then 0 else is_even (n - 1)";
        let result = compile_str(src);
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        assert_eq!(result.fragments.len(), 2);
        let int_id = compute_type_id(&TypeDef::Primitive(PrimType::Int));
        // Both should have Int -> Int types
        assert_eq!(result.fragments[0].1.boundary.inputs[0].1, int_id);
        assert_eq!(result.fragments[0].1.boundary.outputs[0].1, int_id);
        assert_eq!(result.fragments[1].1.boundary.inputs[0].1, int_id);
        assert_eq!(result.fragments[1].1.boundary.outputs[0].1, int_id);
    }

    // -------------------------------------------------------------------
    // Type alias tests
    // -------------------------------------------------------------------

    #[test]
    fn test_type_alias_primitive() {
        // type UserId = Int should be a structural alias
        let src = "type UserId = Int\n\
                   let get_id x : UserId -> UserId = x + 1";
        let result = compile_str(src);
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        let (_, frag, _) = &result.fragments[0];
        let int_id = compute_type_id(&TypeDef::Primitive(PrimType::Int));
        // UserId should resolve to Int
        assert_eq!(frag.boundary.inputs[0].1, int_id,
            "UserId alias should resolve to Int");
        assert_eq!(frag.boundary.outputs[0].1, int_id,
            "UserId alias should resolve to Int");
    }

    #[test]
    fn test_type_alias_arrow() {
        // type Predicate = Int -> Bool
        let src = "type Predicate = Int -> Bool\n\
                   let apply f x : Predicate -> Int -> Bool = f x";
        let result = compile_str(src);
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        let (_, frag, _) = &result.fragments[0];
        let int_id = compute_type_id(&TypeDef::Primitive(PrimType::Int));
        let bool_id = compute_type_id(&TypeDef::Primitive(PrimType::Bool));
        let arrow_id = compute_type_id(&TypeDef::Arrow(int_id, bool_id, CostBound::Unknown));
        // First param should be Predicate = Int -> Bool
        assert_eq!(frag.boundary.inputs[0].1, arrow_id,
            "Predicate alias should resolve to Int -> Bool");
        // Second param should be Int
        assert_eq!(frag.boundary.inputs[1].1, int_id);
        // Return should be Bool
        assert_eq!(frag.boundary.outputs[0].1, bool_id);
    }

    #[test]
    fn test_type_alias_tuple() {
        let src = "type Point = (Int, Int)\n\
                   let origin : Point = (0, 0)";
        let result = compile_str(src);
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        let (_, frag, _) = &result.fragments[0];
        let int_id = compute_type_id(&TypeDef::Primitive(PrimType::Int));
        let tuple_id = compute_type_id(&TypeDef::Product(vec![int_id, int_id]));
        assert_eq!(frag.boundary.outputs[0].1, tuple_id,
            "Point alias should resolve to (Int, Int)");
    }

    #[test]
    fn test_type_alias_chained() {
        // type A = Int, type B = A -> the chain should expand
        let src = "type A = Int\n\
                   type B = A\n\
                   let f x : B -> B = x + 1";
        let result = compile_str(src);
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        let (_, frag, _) = &result.fragments[0];
        let int_id = compute_type_id(&TypeDef::Primitive(PrimType::Int));
        assert_eq!(frag.boundary.inputs[0].1, int_id,
            "Chained alias B -> A -> Int should resolve to Int");
    }

    #[test]
    fn test_type_alias_does_not_shadow_sum() {
        // Sum types should NOT become aliases
        let src = "type Color = Red | Green | Blue\n\
                   let x = Red";
        let result = compile_str(src);
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        let (_, frag, _) = &result.fragments[0];
        let root = &frag.graph.nodes[&frag.graph.root];
        assert_eq!(root.kind, NodeKind::Inject,
            "Sum types should still produce constructors, not be treated as aliases");
    }

    #[test]
    fn test_type_alias_does_not_shadow_record() {
        // Record types should NOT become aliases
        let src = "type Point = { x: Int, y: Int }\n\
                   let p = { x = 1, y = 2 }\n\
                   let get_x p : Point -> Int = p.x";
        let result = compile_str(src);
        assert!(result.errors.is_empty(), "{:?}", result.errors);
    }

    #[test]
    fn test_levenshtein_basic() {
        assert_eq!(super::levenshtein("kitten", "sitting"), 3);
        assert_eq!(super::levenshtein("abc", "abc"), 0);
        assert_eq!(super::levenshtein("abc", "ab"), 1);
        assert_eq!(super::levenshtein("", "abc"), 3);
        assert_eq!(super::levenshtein("abc", ""), 3);
    }

    #[test]
    fn test_did_you_mean_typo_in_scope() {
        // 'ad' is 1 edit away from 'add' (a primitive)
        let src = "let x = ad 1 2";
        let result = compile_str(src);
        assert!(!result.errors.is_empty());
        let msg = &result.errors[0].message;
        assert!(msg.contains("did you mean"), "Expected did-you-mean suggestion, got: {}", msg);
        assert!(msg.contains("'add'"), "Expected suggestion 'add', got: {}", msg);
    }

    #[test]
    fn test_did_you_mean_user_binding() {
        // 'conut' is 2 edits from 'count' (a user binding)
        let src = "let count x = x + 1\nlet y = conut 5";
        let result = compile_str(src);
        assert!(!result.errors.is_empty());
        let msg = &result.errors[0].message;
        assert!(msg.contains("did you mean"), "Expected did-you-mean suggestion, got: {}", msg);
        assert!(msg.contains("'count'"), "Expected suggestion 'count', got: {}", msg);
    }

    #[test]
    fn test_no_suggestion_for_distant_name() {
        // 'xyzzy' is too far from anything
        let src = "let x = xyzzy 1 2";
        let result = compile_str(src);
        assert!(!result.errors.is_empty());
        let msg = &result.errors[0].message;
        assert!(msg.contains("undefined variable"), "Expected plain error, got: {}", msg);
        assert!(!msg.contains("did you mean"), "Should not suggest for distant name, got: {}", msg);
    }
}
