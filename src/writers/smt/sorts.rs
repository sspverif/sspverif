use super::exprs::SmtExpr;

pub struct Array<K, V>
where
    K: Into<SmtExpr>,
    V: Into<SmtExpr>,
{
    pub key: K,
    pub value: V,
}

impl<K, V> From<Array<K, V>> for SmtExpr
where
    K: Into<SmtExpr>,
    V: Into<SmtExpr>,
{
    fn from(t: Array<K, V>) -> Self {
        let Array { key, value } = t;
        ("Array", key, value).into()
    }
}
