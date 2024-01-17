use crate::{
    package::{Composition, OracleDef, Package, PackageInstance},
    proof::Assumption,
};

pub trait Resolver<'a, T>: IndexLifetime<'a, usize, Output = T> {
    fn resolve_index(&self, name: &str) -> Option<usize>;

    fn resolve_value(&self, name: &str) -> Option<&'a T> {
        self.resolve_index(name).map(|idx| self.index(idx))
    }

    fn resolve(&self, name: &str) -> Option<(usize, &'a T)> {
        self.resolve_index(name).map(|idx| (idx, self.index(idx)))
    }
}

pub trait IndexLifetime<'a, T> {
    type Output;

    fn index(&self, index: T) -> &'a Self::Output;
}

#[derive(Debug)]
pub struct SliceResolver<'a, T>(pub &'a [T]);

impl<'a, S: AsRef<str>, T> Resolver<'a, (S, T)> for SliceResolver<'a, (S, T)> {
    fn resolve_index(&self, name: &str) -> Option<usize> {
        self.0
            .iter()
            .enumerate()
            .find(|(_i, (item_name, _))| item_name.as_ref() == name)
            .map(|(i, _)| i)
    }
}

impl<'a, S: AsRef<str>, T> IndexLifetime<'a, usize> for SliceResolver<'a, (S, T)> {
    type Output = (S, T);

    fn index(&self, index: usize) -> &'a Self::Output {
        &self.0[index]
    }
}

impl<'a, S: AsRef<str>, T, U> Resolver<'a, (S, T, U)> for SliceResolver<'a, (S, T, U)> {
    fn resolve_index(&self, name: &str) -> Option<usize> {
        self.0
            .iter()
            .enumerate()
            .find(|(_i, (item_name, _, _))| item_name.as_ref() == name)
            .map(|(i, _)| i)
    }
}

impl<'a, S: AsRef<str>, T, U> IndexLifetime<'a, usize> for SliceResolver<'a, (S, T, U)> {
    type Output = (S, T, U);

    fn index(&self, index: usize) -> &'a Self::Output {
        &self.0[index]
    }
}

pub trait Named {
    fn as_name(&self) -> &str;
}

impl<'a, T: Named> IndexLifetime<'a, usize> for SliceResolver<'a, T> {
    type Output = T;

    fn index(&self, index: usize) -> &'a Self::Output {
        &self.0[index]
    }
}

impl<'a, T: Named> Resolver<'a, T> for SliceResolver<'a, T> {
    fn resolve_index(&self, name: &str) -> Option<usize> {
        self.0
            .iter()
            .enumerate()
            .find(|(_i, v)| v.as_name() == name)
            .map(|(i, _v)| i)
    }
}

impl<'a, T: Named> std::ops::Index<usize> for SliceResolver<'a, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<'a, S: AsRef<str>, T> std::ops::Index<usize> for SliceResolver<'a, (S, T)> {
    type Output = (S, T);

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<'a, S: AsRef<str>, T, U> std::ops::Index<usize> for SliceResolver<'a, (S, T, U)> {
    type Output = (S, T, U);

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

#[macro_export]
macro_rules! impl_Named {
    ($tipe:ident) => {
        impl $crate::util::resolver::Named for $tipe {
            fn as_name(&self) -> &str {
                &self.name
            }
        }
    };
}

impl_Named!(Package);
impl_Named!(PackageInstance);
impl_Named!(Composition);
impl_Named!(Assumption);

impl Named for OracleDef {
    fn as_name(&self) -> &str {
        &self.sig.name
    }
}
