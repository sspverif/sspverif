use crate::expressions::Expression;
use crate::statement::{CodeBlock, Statement};
use crate::types::Type;

use std::collections::HashMap;
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OracleDef {
    pub sig: OracleSig,
    pub code: CodeBlock,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Package {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub state: Vec<(String, Type)>,
    pub oracles: Vec<OracleDef>,
    pub imports: Vec<OracleSig>,
}

impl Package {
    pub fn called_oracles(&self, oracle: &OracleDef) -> Vec<OracleSig> {
        let mut result = Vec::new();
        let mut called_oracles_helper = |cb: &CodeBlock| {
            for stmt in &cb.0 {
                match stmt {
                    Statement::InvokeOracle {
                        name,
                        args,
                        tipe: Some(tipe),
                        ..
                    } => {
                        let found = self.imports.iter().find(|sig|{
                            sig.name == *name && sig.tipe == *tipe &&
                                sig.args.iter().zip(args.iter()).all(|(l,r)| {
                                    if let Expression::Typed(t, _) = r {
                                        l.1 == *t
                                    } else {
                                        panic!("OracleDef called_oracles() only to be called after typing")
                                    }


                                })
                        });

                        result.push(found.unwrap().clone());
                    }
                    _ => {}
                }
            }
        };

        called_oracles_helper(&oracle.code);
        result
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageInstance {
    pub params: HashMap<String, String>,
    pub pkg: Package,
    pub name: String,
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
pub struct Export(pub usize, pub OracleSig);

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
    pub name: String,
    pub consts: Vec<(String, Type)>,
}

impl Composition {
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
