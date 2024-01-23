use crate::expressions::Expression;
use crate::identifier::Identifier;
use crate::parser::package::{ForSpec, MultiInstanceIndices};
use crate::split::{SplitOracleDef, SplitOracleSig};
use crate::statement::{CodeBlock, FilePosition};
use crate::types::Type;

use std::convert::TryInto;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnSig {
    pub name: String,
    pub args: Vec<(String, Type)>,
    pub tipe: Type,
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct OracleSig {
    pub name: String,
    pub args: Vec<(String, Type)>,
    pub multi_inst_idx: Option<MultiInstanceIndices>,
    pub tipe: Type,
}

impl OracleSig {
    pub(crate) fn is_compatible(&self, other: &Self) -> bool {
        self.name == other.name
            && self.args.len() == other.args.len()
            && self
                .args
                .iter()
                .zip(other.args.iter())
                .all(|((_, left_type), (_, right_type))| left_type == right_type)
            && self.tipe == other.tipe
    }
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
    pub multi_instance_indices: Vec<String>,
    pub forspecs: Vec<ForSpec>,
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
    pub source_pkgidx: usize,
    pub source_instance_idx: Vec<Identifier>,
    pub dest_pkgidx: usize,
    pub oracle_sig: OracleSig,
}

#[derive(Debug)]
pub struct NotSingleInstanceEdgeError(pub MultiInstanceEdge);

impl std::error::Error for NotSingleInstanceEdgeError {}

impl std::fmt::Display for NotSingleInstanceEdgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "is not a single instance edge: {:?}", self.0)
    }
}

impl TryInto<Edge> for MultiInstanceEdge {
    type Error = NotSingleInstanceEdgeError;

    fn try_into(self) -> Result<Edge, Self::Error> {
        if self.oracle_sig.multi_inst_idx.is_none() && self.source_instance_idx.is_empty() {
            Ok(Edge(self.source_pkgidx, self.dest_pkgidx, self.oracle_sig))
        } else {
            Err(NotSingleInstanceEdgeError(self))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Export(pub usize, pub OracleSig);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MultiInstanceExport {
    pub dest_pkgidx: usize,
    pub oracle_sig: OracleSig,
}

#[derive(Debug)]
pub struct NotSingleInstanceExportError(pub MultiInstanceExport);

impl std::error::Error for NotSingleInstanceExportError {}

impl std::fmt::Display for NotSingleInstanceExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "is not a single instance edge: {:?}", self.0)
    }
}

impl TryInto<Export> for MultiInstanceExport {
    type Error = NotSingleInstanceExportError;

    fn try_into(self) -> Result<Export, Self::Error> {
        if self.oracle_sig.multi_inst_idx.is_none() {
            Ok(Export(self.dest_pkgidx, self.oracle_sig))
        } else {
            Err(NotSingleInstanceExportError(self))
        }
    }
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

    pub multi_inst_edges: Vec<MultiInstanceEdge>,
    pub multi_inst_exports: Vec<MultiInstanceExport>,
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
