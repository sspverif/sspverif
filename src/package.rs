use crate::expressions::Expression;
use crate::identifier::Identifier;
use crate::split::{SplitOracleDef, SplitOracleSig};
use crate::statement::{CodeBlock, FilePosition};
use crate::types::Type;

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnSig {
    pub name: String,
    pub args: Vec<(String, Type)>,
    pub tipe: Type,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OracleSig {
    pub name: String,
    pub args: Vec<(String, Type)>,
    pub tipe: Type,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OracleDef {
    pub sig: OracleSig,
    pub code: CodeBlock,
    pub file_pos: FilePosition,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Package {
    pub name: String,
    pub types: Vec<Type>,
    pub params: Vec<(String, Type, FilePosition)>,
    pub state: Vec<(String, Type, FilePosition)>,
    pub oracles: Vec<OracleDef>,
    pub split_oracles: Vec<SplitOracleDef>,
    pub imports: Vec<(OracleSig, FilePosition)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageInstance {
    pub params: Vec<(String, Expression)>,
    pub types: Vec<(Type, Type)>,
    pub pkg: Package,
    pub name: String,
    pub multi_instance_indeces: Vec<(String, Type)>,
}

impl PackageInstance {
    pub fn get_oracle_sigs(&self) -> Vec<OracleSig> {
        self.pkg
            .oracles
            .clone()
            .into_iter()
            .map(|d| d.sig)
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge(pub usize, pub usize, pub OracleSig);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiInstanceEdge {
    // name, from, to. name: from <= name < to
    // expressions are normalized to fit the above
    pub loopvars: Vec<(String, Expression, Expression)>,
    pub source_pkgidx: usize,
    pub source_instance_idx: Vec<Identifier>,
    pub dest_pkgidx: usize,
    pub dest_instance_idx: Vec<(Identifier, Expression)>,
    pub oracle_sig: OracleSig,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Export(pub usize, pub OracleSig);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MultiInstanceExport {
    // name, from, to. name: from <= name < to
    // expressions are normalized to fit the above
    pub loopvars: Vec<(String, Expression, Expression)>,
    pub dest_pkgidx: usize,
    pub dest_instance_idx: Vec<Expression>,
    pub oracle_sig: OracleSig,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SplitExport(pub usize, pub SplitOracleSig);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Composition {
    pub pkgs: Vec<PackageInstance>,
    pub edges: Vec<Edge>, // (from, to, oraclesig)
    // TODO: how do we deal with the case where we have
    // e.g. multiple key packages that we Set into?
    // Idea: Add a name to this tuple that is used by
    // the invoking package
    // contemplation: globally unique oracle identifiers vs
    // multiple shades of local uniqueness
    pub exports: Vec<Export>,
    pub split_exports: Vec<SplitExport>,
    pub name: String,
    pub consts: Vec<(String, Type)>,
}

impl Composition {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn get_oracle_sigs(&self) -> Vec<OracleSig> {
        self.exports
            .iter()
            .map(|Export(_, sig)| sig.clone())
            .collect()
    }

    pub fn ordered_pkgs(&self) -> Vec<PackageInstance> {
        let mut result = Vec::new();
        let mut added_pkgs = vec![false; self.pkgs.len()];

        while result.len() < self.pkgs.len() {
            let mut candidates = vec![true; self.pkgs.len()];
            for Edge(from, to, _) in &self.edges {
                if !added_pkgs[*to] {
                    candidates[*from] = false;
                }
            }
            for i in 0..self.pkgs.len() {
                if !added_pkgs[i] && candidates[i] {
                    result.push(self.pkgs[i].clone());
                    added_pkgs[i] = true;
                }
            }
        }
        result
    }
}

impl Composition {
    #[allow(unused_mut)]
    pub fn map_pkg_inst<E, F>(&self, mut f: F) -> Result<Composition, E>
    where
        F: FnMut(&PackageInstance) -> Result<PackageInstance, E>,
    {
        Ok(Composition {
            pkgs: {
                let res: Result<Vec<_>, E> = self.pkgs.iter().map(f).collect();
                res?
            },
            ..self.clone()
        })
    }

    pub fn map_oracle<E>(
        &self,
        f: &mut impl FnMut(&OracleDef) -> Result<OracleDef, E>,
    ) -> Result<Composition, E> {
        self.map_pkg_inst(|pkg_inst| {
            Ok(PackageInstance {
                pkg: pkg_inst.pkg.map_oracle(f)?,
                ..pkg_inst.clone()
            })
        })
    }
}

impl Package {
    pub fn map_oracle<E>(
        &self,
        f: &mut impl FnMut(&OracleDef) -> Result<OracleDef, E>,
    ) -> Result<Package, E> {
        Ok(Package {
            oracles: {
                let res: Result<Vec<_>, E> = self.oracles.iter().map(f).collect();
                res?
            },
            ..self.clone()
        })
    }
}

impl fmt::Display for OracleSig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use `self.number` to refer to each positional data point.
        write!(
            f,
            "{}({}) -> {:?}",
            self.name,
            self.args
                .iter()
                .map(|(name, tipe)| format!("{}: {:?}", name, tipe))
                .collect::<Vec<_>>()
                .join(", "),
            self.tipe
        )
    }
}
