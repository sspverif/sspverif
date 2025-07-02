use std::borrow::Borrow;

use crate::{expressions::Expression, types::Type, writers::smt::exprs::SmtExpr};

pub fn only_ints<'a, T: 'a, I: IntoIterator<Item = &'a (T, Expression)>>(
    iter: I,
) -> impl Iterator<Item = &'a Expression> {
    iter.into_iter()
        .filter_map(|(_, expr)| match expr.get_type() {
            Type::Integer => Some(expr),
            _ => None,
        })
}

pub fn only_ints_and_funs<'a, T: 'a, I: IntoIterator<Item = &'a (T, Expression)>>(
    iter: I,
) -> impl Iterator<Item = &'a Expression> {
    iter.into_iter()
        .filter_map(|(_, expr)| match expr.get_type() {
            Type::Integer | Type::Fn(_, _) => Some(expr),
            _ => None,
        })
}

pub fn only_non_function_expression<'a, T: 'a, I: IntoIterator<Item = &'a (T, Expression)>>(
    iter: I,
) -> impl Iterator<Item = &'a Expression> {
    iter.into_iter()
        .filter_map(|(_, expr)| match expr.get_type() {
            Type::Fn(_, _) => None,
            _ => Some(expr),
        })
}

pub struct Separated<'a, T> {
    item: Option<T>,
    sep: &'a str,
}

impl<'a, T> Separated<'a, T> {
    pub fn new(item: Option<T>, sep: &'a str) -> Self {
        Self { item, sep }
    }
}
impl<'a, T: std::fmt::Display> std::fmt::Display for Separated<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(item) = &self.item {
            write!(f, "{}{}", self.sep, item)
        } else {
            Ok(())
        }
    }
}

pub fn encode_params<'a, I>(params_iter: I) -> Option<String>
where
    I: IntoIterator<Item = &'a Expression>,
{
    encode_smt_exprs(
        params_iter
            .into_iter()
            .map(|expr: &Expression| -> SmtExpr { expr.clone().into() }),
    )
}

pub fn encode_smt_exprs<Ref: Borrow<SmtExpr>, Iter: IntoIterator<Item = Ref>>(
    exprs: Iter,
) -> Option<String> {
    let mut out = String::with_capacity(64);

    // Don't print the angle brackets if there are no parames
    let mut peekable = exprs.into_iter().peekable();
    if peekable.peek().is_none() {
        return None;
    }

    out.push_str("<$");
    let mut out = peekable.fold(out, |mut acc, expr| {
        if let Some(encoding) = encode_smt_expr(expr.borrow()) {
            acc.push_str(&encoding);
        }
        acc
    });
    out.push_str("$>");

    Some(out)
}

fn encode_smt_expr(expr: &SmtExpr) -> Option<String> {
    match expr {
        SmtExpr::Comment(_) => None,
        SmtExpr::Atom(atom) => Some(format!("<!{atom}!>")),
        SmtExpr::List(exprs) => encode_smt_exprs(exprs),
    }
}
