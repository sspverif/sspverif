use super::exprs::{SmtExpr, SmtWrap};

pub fn declare_datatype(
    sort_name: &str,
    constructor_name: &str,
    fields: impl Iterator<Item = (String, SmtExpr)>,
) -> SmtExpr {
    (
        "declare-datatype",
        sort_name,
        SmtWrap({
            let mut tmp = vec![constructor_name.into()];
            tmp.extend(fields.into_iter().map(|field_pair| field_pair.into()));
            SmtExpr::List(tmp)
        }),
    )
        .into()
}

pub fn declare_const<S: Into<SmtExpr>>(const_name: String, sort: S) -> SmtExpr {
    ("declare-const", const_name, sort).into()
}
