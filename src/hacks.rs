use std::fmt::{Display, Result};

pub struct MaybeDeclaration;

impl Display for MaybeDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        writeln!(
            f,
            "(declare-datatypes ((Maybe 1)) ((par (T) ((mk-some (maybe-get T)) (mk-none)))))"
        )
    }
}

pub struct TupleDeclaration(pub usize);

impl Display for TupleDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        let TupleDeclaration(n) = self;
        let n = *n;

        let types: String = (1..n + 1)
            .map(|i| format!("T{i}"))
            .collect::<Vec<_>>()
            .join(" ");

        let ds: String = (1..n + 1)
            .map(|i| format!("(el{i} T{i})"))
            .collect::<Vec<_>>()
            .join(" ");

        writeln!(
            f,
            "(declare-datatypes ((Tuple{n} {n})) ((par ({types}) ((mk-tuple{n} {ds})))))"
        )
    }
}

pub struct TuplesDeclaration(pub std::ops::Range<usize>);

impl Display for TuplesDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        let TuplesDeclaration(range) = self;
        let iter = range.clone().map(|i| TupleDeclaration(i));
        for decl in iter {
            write!(f, "{decl}")?;
        }

        Ok(())
    }
}

pub struct BitsDeclaration(pub String);

impl Display for BitsDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        let BitsDeclaration(id) = self;
        writeln!(f, "(declare-sort Bits_{id} 0)")
    }
}

pub struct EmptyDeclaration;

impl Display for EmptyDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        writeln!(f, "(declare-datatype Empty ((mk-empty)) )")
    }
}
