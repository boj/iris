//! IRIS source code formatter.
//!
//! Parses .iris source and pretty-prints with consistent style.
//! Used by `iris fmt` CLI command.

use super::ast::*;

/// Format an IRIS module to a string.
pub fn format_module(module: &Module) -> String {
    let mut out = String::new();
    let mut first = true;
    for item in &module.items {
        if !first { out.push('\n'); }
        first = false;
        format_item(&mut out, item, 0);
    }
    if !out.ends_with('\n') { out.push('\n'); }
    out
}

fn format_item(out: &mut String, item: &Item, indent: usize) {
    match item {
        Item::LetDecl(decl) => format_let_decl(out, decl, indent),
        Item::MutualRecGroup(decls) => {
            for (i, decl) in decls.iter().enumerate() {
                if i > 0 {
                    write_indent(out, indent);
                    out.push_str("and ");
                    format_let_decl_body(out, decl, indent);
                } else {
                    format_let_decl(out, decl, indent);
                }
            }
        }
        Item::TypeDecl(td) => format_type_decl(out, td, indent),
        Item::Import(imp) => {
            write_indent(out, indent);
            out.push_str("import ");
            match &imp.source {
                ImportSource::Path(p) => { out.push('"'); out.push_str(p); out.push('"'); }
                ImportSource::Hash(h) => { out.push('#'); out.push_str(h); }
            }
            out.push_str(" as ");
            out.push_str(&imp.name);
            out.push('\n');
        }
        Item::ClassDecl(cd) => {
            write_indent(out, indent);
            out.push_str("class "); out.push_str(&cd.name);
            out.push('<'); out.push_str(&cd.type_param); out.push('>');
            out.push_str(" where\n");
            for m in &cd.methods {
                write_indent(out, indent + 2);
                out.push_str(&m.name); out.push_str(" : ");
                format_type_expr(out, &m.type_sig);
                if let Some(ref def) = m.default_impl {
                    out.push_str(" = "); format_expr(out, def, indent + 2);
                }
                out.push('\n');
            }
        }
        Item::InstanceDecl(inst) => {
            write_indent(out, indent);
            out.push_str("instance "); out.push_str(&inst.class_name);
            out.push('<'); format_type_expr(out, &inst.type_arg); out.push('>');
            out.push_str(" where\n");
            for (name, expr) in &inst.methods {
                write_indent(out, indent + 2);
                out.push_str(name); out.push_str(" = ");
                format_expr(out, expr, indent + 2);
                out.push('\n');
            }
        }
    }
}

fn format_let_decl(out: &mut String, decl: &LetDecl, indent: usize) {
    write_indent(out, indent);
    out.push_str("let ");
    if decl.recursive { out.push_str("rec "); }
    format_let_decl_body(out, decl, indent);
}

fn format_let_decl_body(out: &mut String, decl: &LetDecl, indent: usize) {
    out.push_str(&decl.name);
    for param in &decl.params {
        out.push(' ');
        out.push_str(param);
    }
    if let Some(ref te) = decl.ret_type {
        out.push_str(" : ");
        format_type_expr(out, te);
    }
    if let Some(ref cost) = decl.cost {
        out.push_str(" [cost: ");
        format_cost(out, cost);
        out.push(']');
    }
    out.push_str(" =\n");
    write_indent(out, indent + 2);
    format_expr(out, &decl.body, indent + 2);
    out.push('\n');
}

fn format_cost(out: &mut String, cost: &CostExpr) {
    match cost {
        CostExpr::Unknown => out.push_str("Unknown"),
        CostExpr::Zero => out.push_str("Zero"),
        CostExpr::Constant(n) => { out.push_str("Const("); out.push_str(&n.to_string()); out.push(')'); }
        CostExpr::Linear(v) => { out.push_str("Linear("); out.push_str(v); out.push(')'); }
        CostExpr::NLogN(v) => { out.push_str("NLogN("); out.push_str(v); out.push(')'); }
        CostExpr::Polynomial(v, d) => {
            out.push_str("Polynomial("); out.push_str(v);
            out.push_str(", "); out.push_str(&d.to_string()); out.push(')');
        }
        CostExpr::Sum(a, b) => {
            format_cost(out, a); out.push_str(" + "); format_cost(out, b);
        }
    }
}

fn format_type_decl(out: &mut String, td: &TypeDecl, indent: usize) {
    write_indent(out, indent);
    out.push_str("type ");
    out.push_str(&td.name);
    if !td.type_params.is_empty() {
        out.push('<');
        for (i, p) in td.type_params.iter().enumerate() {
            if i > 0 { out.push_str(", "); }
            out.push_str(p);
        }
        out.push('>');
    }
    out.push_str(" = ");
    format_type_expr(out, &td.def);
    out.push('\n');
}

fn format_type_expr(out: &mut String, te: &TypeExpr) {
    match te {
        TypeExpr::Named(name, _) => out.push_str(name),
        TypeExpr::Arrow(a, b, _) => {
            format_type_expr(out, a); out.push_str(" -> "); format_type_expr(out, b);
        }
        TypeExpr::Tuple(elems, _) => {
            out.push('(');
            for (i, e) in elems.iter().enumerate() {
                if i > 0 { out.push_str(", "); }
                format_type_expr(out, e);
            }
            out.push(')');
        }
        TypeExpr::Unit(_) => out.push_str("()"),
        TypeExpr::ForAll(var, body, _) => {
            out.push_str("forall "); out.push_str(var); out.push_str(". ");
            format_type_expr(out, body);
        }
        TypeExpr::Refined(var, base, pred, _) => {
            out.push('{'); out.push_str(var); out.push_str(": ");
            format_type_expr(out, base); out.push_str(" | ");
            format_expr(out, pred, 0); out.push('}');
        }
        TypeExpr::App(name, args, _) => {
            out.push_str(name); out.push('<');
            for (i, a) in args.iter().enumerate() {
                if i > 0 { out.push_str(", "); }
                format_type_expr(out, a);
            }
            out.push('>');
        }
        TypeExpr::Sum(variants, _) => {
            for (i, (name, payload)) in variants.iter().enumerate() {
                if i > 0 { out.push_str(" | "); }
                out.push_str(name);
                if let Some(p) = payload {
                    out.push('('); format_type_expr(out, p); out.push(')');
                }
            }
        }
        TypeExpr::Record(fields, _) => {
            out.push_str("{ ");
            for (i, (name, te)) in fields.iter().enumerate() {
                if i > 0 { out.push_str(", "); }
                out.push_str(name); out.push_str(": "); format_type_expr(out, te);
            }
            out.push_str(" }");
        }
        TypeExpr::RecordMerge(lhs, rhs, _) => {
            format_type_expr(out, lhs); out.push_str(" / "); format_type_expr(out, rhs);
        }
    }
}

fn format_expr(out: &mut String, expr: &Expr, indent: usize) {
    match expr {
        Expr::IntLit(v, _) => out.push_str(&v.to_string()),
        Expr::FloatLit(v, _) => {
            let s = v.to_string();
            out.push_str(&s);
            if !s.contains('.') { out.push_str(".0"); }
        }
        Expr::BoolLit(v, _) => out.push_str(if *v { "true" } else { "false" }),
        Expr::StringLit(s, _) => { out.push('"'); out.push_str(s); out.push('"'); }
        Expr::UnitLit(_) => out.push_str("()"),
        Expr::Var(name, _) => out.push_str(name),
        Expr::App(f, arg, _) => {
            let needs_parens_f = matches!(f.as_ref(), Expr::Lambda(_, _, _) | Expr::If(_, _, _, _));
            if needs_parens_f { out.push('('); }
            format_expr(out, f, indent);
            if needs_parens_f { out.push(')'); }
            out.push(' ');
            let needs_parens = matches!(arg.as_ref(),
                Expr::App(_, _, _) | Expr::BinOp(_, _, _, _) | Expr::If(_, _, _, _));
            if needs_parens { out.push('('); }
            format_expr(out, arg, indent);
            if needs_parens { out.push(')'); }
        }
        Expr::Lambda(params, body, _) => {
            out.push('\\');
            for (i, p) in params.iter().enumerate() {
                if i > 0 { out.push(' '); }
                out.push_str(p);
            }
            out.push_str(" -> ");
            format_expr(out, body, indent);
        }
        Expr::Let(name, val, body, _) => {
            out.push_str("let "); out.push_str(name); out.push_str(" = ");
            format_expr(out, val, indent); out.push_str(" in\n");
            write_indent(out, indent); format_expr(out, body, indent);
        }
        Expr::LetRec(name, val, body, _) => {
            out.push_str("let rec "); out.push_str(name); out.push_str(" = ");
            format_expr(out, val, indent); out.push_str(" in\n");
            write_indent(out, indent); format_expr(out, body, indent);
        }
        Expr::If(cond, then_, else_, _) => {
            out.push_str("if "); format_expr(out, cond, indent);
            out.push_str(" then "); format_expr(out, then_, indent);
            out.push_str(" else "); format_expr(out, else_, indent);
        }
        Expr::BinOp(lhs, op, rhs, _) => {
            format_expr(out, lhs, indent);
            out.push(' '); out.push_str(op_str(*op)); out.push(' ');
            format_expr(out, rhs, indent);
        }
        Expr::UnaryOp(op, inner, _) => {
            out.push_str(match op { UnaryOp::Neg => "neg ", UnaryOp::Not => "not " });
            format_expr(out, inner, indent);
        }
        Expr::OpSection(op, _) => {
            out.push('('); out.push_str(op_str(*op)); out.push(')');
        }
        Expr::Tuple(elems, _) => {
            out.push('(');
            for (i, e) in elems.iter().enumerate() {
                if i > 0 { out.push_str(", "); }
                format_expr(out, e, indent);
            }
            out.push(')');
        }
        Expr::TupleAccess(expr, idx, _) => {
            format_expr(out, expr, indent); out.push('.'); out.push_str(&idx.to_string());
        }
        Expr::Match(scr, arms, _) => {
            out.push_str("match "); format_expr(out, scr, indent); out.push_str(" with\n");
            for arm in arms {
                write_indent(out, indent); out.push_str("| ");
                format_pattern(out, &arm.pattern);
                if let Some(ref guard) = arm.guard {
                    out.push_str(" when "); format_expr(out, guard, indent);
                }
                out.push_str(" -> "); format_expr(out, &arm.body, indent + 4);
                out.push('\n');
            }
        }
        Expr::Pipe(lhs, rhs, _) => {
            format_expr(out, lhs, indent); out.push_str(" |> "); format_expr(out, rhs, indent);
        }
        Expr::RecordLit(fields, _) => {
            out.push_str("{ ");
            for (i, (name, val)) in fields.iter().enumerate() {
                if i > 0 { out.push_str(", "); }
                out.push_str(name); out.push_str(" = "); format_expr(out, val, indent);
            }
            out.push_str(" }");
        }
        Expr::FieldAccess(expr, field, _) => {
            format_expr(out, expr, indent); out.push('.'); out.push_str(field);
        }
    }
}

fn format_pattern(out: &mut String, pat: &Pattern) {
    match pat {
        Pattern::Wildcard(_) => out.push('_'),
        Pattern::Ident(name, _) => out.push_str(name),
        Pattern::IntLit(v, _) => out.push_str(&v.to_string()),
        Pattern::BoolLit(v, _) => out.push_str(if *v { "true" } else { "false" }),
        Pattern::Constructor(name, inner, _) => {
            out.push_str(name);
            if let Some(p) = inner {
                out.push('('); format_pattern(out, p); out.push(')');
            }
        }
        Pattern::Tuple(pats, _) => {
            out.push('(');
            for (i, p) in pats.iter().enumerate() {
                if i > 0 { out.push_str(", "); }
                format_pattern(out, p);
            }
            out.push(')');
        }
    }
}

fn op_str(op: BinOp) -> &'static str {
    match op {
        BinOp::Add => "+", BinOp::Sub => "-", BinOp::Mul => "*",
        BinOp::Div => "/", BinOp::Mod => "%",
        BinOp::Eq => "==", BinOp::Ne => "!=",
        BinOp::Lt => "<", BinOp::Gt => ">",
        BinOp::Le => "<=", BinOp::Ge => ">=",
        BinOp::And => "&&", BinOp::Or => "||",
    }
}

fn write_indent(out: &mut String, n: usize) {
    for _ in 0..n { out.push(' '); }
}
