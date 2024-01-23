use super::errors::TypeCheckError;
use super::pkg::typecheck_pkg;
use super::scope::Scope;

use crate::package::{Composition, Edge, Export, MultiInstanceEdge, OracleSig, PackageInstance};
use crate::util::resolver::Named;

use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

#[derive(Debug, Clone, Eq, PartialOrd, Ord)]
struct IgnoreArgNameOracleSig(OracleSig);

impl PartialEq for IgnoreArgNameOracleSig {
    fn eq(&self, other: &Self) -> bool {
        self.0.name == other.0.name
            && self.0.multi_inst_idx == other.0.multi_inst_idx
            && self.0.tipe == other.0.tipe
            && self.0.args.len() == other.0.args.len()
            && self
                .0
                .args
                .iter()
                .zip(other.0.args.iter())
                .all(|((_, left_type), (_, right_type))| left_type == right_type)
    }
}

impl std::hash::Hash for IgnoreArgNameOracleSig {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.name.hash(state);
        self.0.tipe.hash(state);
        self.0.multi_inst_idx.hash(state);
        state.write_usize(self.0.args.len());

        for (_, arg_type) in &self.0.args {
            arg_type.hash(state)
        }
    }
}

pub fn typecheck_comp(
    comp: &Composition,
    scope: &mut Scope,
) -> Result<Composition, TypeCheckError> {
    let Composition {
        pkgs,
        edges,
        exports,
        name,
        multi_inst_edges,
        ..
    } = comp;

    // 1a. check signature exists in edge destination
    for Edge(_, to, sig_) in edges {
        let mut found = false;
        for sig in pkgs[*to].get_oracle_sigs() {
            if sig == sig_.clone() {
                found = true;
            }
        }
        if !found {
            return Err(TypeCheckError::TypeCheck(format!(
                "couldn't find signature for {:?} in package {:?} with id {:}",
                sig_, pkgs[*to], to
            )));
        }
    }

    // 1b. check signature matches in package imports
    let declared_imports: HashMap<_, _> = pkgs
        .iter()
        .enumerate()
        .map(|(i, pkg)| {
            (
                i,
                HashSet::from_iter(
                    pkg.pkg
                        .imports
                        .iter()
                        .map(|(osig, _filepos)| IgnoreArgNameOracleSig(osig.clone())),
                ),
            )
        })
        .filter(|(_i, imports)| !imports.is_empty())
        .collect();

    // println!("declared imports: {declared_imports:#?}");

    // edge_imports is a hashmap: from_edge_idx -> (hashset sig)
    let mut edge_imports = HashMap::new();

    println!("edges:    {edges:?}");
    println!("mi-edges: {multi_inst_edges:?}");

    for Edge(from, _, sig_) in edges {
        println!("singl: {}", sig_.name);
        edge_imports
            .entry(*from)
            .or_insert_with(HashSet::new)
            .insert(IgnoreArgNameOracleSig(sig_.clone()));
    }

    for MultiInstanceEdge {
        oracle_sig,
        source_pkgidx,
        ..
    } in multi_inst_edges
    {
        println!("multi: {}", oracle_sig.name);

        edge_imports
            .entry(*source_pkgidx)
            .or_insert_with(HashSet::new)
            .insert(IgnoreArgNameOracleSig(oracle_sig.clone()));
    }

    if declared_imports != edge_imports {
        if declared_imports.keys().collect::<HashSet<_>>()
            != edge_imports.keys().collect::<HashSet<_>>()
        {
            panic!(
                "Composition {}: Different set of keys with imports, declared: {:#?} edges: {:#?}",
                name,
                declared_imports.keys(),
                edge_imports.keys()
            )
        }
        for (i, pkg_inst) in pkgs.clone().iter().enumerate() {
            if !declared_imports.contains_key(&i) {
                continue;
            }
            if declared_imports[&i] != edge_imports[&i] {
                let left = &declared_imports[&i];
                let right = &edge_imports[&i];
                let mut declared_imports: Vec<_> = declared_imports[&i].iter().cloned().collect();
                // .iter()
                // .map(|osig| {
                //     let mut code = String::new();
                //     let mut w =
                //         crate::writers::pseudocode::fmtwriter::FmtWriter::new(&mut code, false);
                //     w.write_oraclesig(osig).unwrap();
                //     code
                // })
                // .collect();
                declared_imports.sort();

                let mut edge_imports: Vec<_> = edge_imports[&i].iter().cloned().collect();
                // .iter()
                // .map(|osig| {
                //     let mut code = String::new();
                //     let mut w =
                //         crate::writers::pseudocode::fmtwriter::FmtWriter::new(&mut code, false);
                //     w.write_oraclesig(osig).unwrap();
                //     code
                // })
                // .collect();
                edge_imports.sort();

                assert_eq!(declared_imports.len(), edge_imports.len());
                println!("comp: {name}, i:{i} pkg_inst: {}", pkg_inst.as_name());
                for i in 0..declared_imports.len() {
                    println!("declared: {:#?}", declared_imports[i]);
                    println!("edge:     {:#?}", edge_imports[i]);
                    println!("game:     {}", comp.name());
                    assert_eq!(declared_imports[i], edge_imports[i]);
                    println!("{i} ok")
                }

                panic!(
                    "Composition {}: package: {} declared: {:#?} edges: {:#?}\n ---\n  {left:#?}\n !=\n  {right:#?}",
                    name, pkg_inst.name, declared_imports, edge_imports
                );
            }
        }
    }

    // 2. check exports exists
    for Export(id, sig) in exports {
        if !pkgs[*id].get_oracle_sigs().contains(sig) {
            return Err(TypeCheckError::TypeCheck(format!(
                "signature {:?} is not in package {:?} with index {:}",
                sig,
                pkgs[*id].clone(),
                id
            )));
        }
    }

    let mut typed_pkgs = vec![];

    // 3. check all package instances
    for (_id, pkg) in pkgs.clone().into_iter().enumerate() {
        scope.enter();
        // for Edge(from, _, sig) in edges {
        //     if *from == id {
        //         scope.declare_oracle(
        //             Identifier::new_scalar(sig.name.as_str()),
        //             sig.args.clone().into_iter().map(|(_, tipe)| tipe).collect(),
        //             sig.tipe.clone(),
        //         )?;
        //     }
        // }
        let result = typecheck_pkg(&pkg.pkg, scope)?;
        scope.leave();

        typed_pkgs.push(PackageInstance {
            pkg: result,
            ..pkg.clone()
        });
    }

    Ok(Composition {
        pkgs: typed_pkgs,
        ..comp.clone()
    })
}
