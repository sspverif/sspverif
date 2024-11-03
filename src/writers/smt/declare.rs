use super::{
    exprs::{SmtExpr, SmtWrap},
    sorts::Sort,
};

pub fn declare_single_constructor_datatype(
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

pub fn declare_datatype(
    sort_name: &str,
    constructors: impl Iterator<Item = (String, Vec<(String, SmtExpr)>)>,
) -> SmtExpr {
    (
        "declare-datatype",
        sort_name,
        SmtExpr::List(
            constructors
                .map(|(name, fields)| {
                    let mut tmp = vec![name.into()];
                    tmp.extend(fields.into_iter().map(|field_pair| field_pair.into()));
                    SmtExpr::List(tmp)
                })
                .collect(),
        ),
    )
        .into()
}

pub fn declare_const<N: Into<SmtExpr>>(const_name: N, sort: Sort) -> SmtExpr {
    ("declare-const", const_name, sort).into()
}
