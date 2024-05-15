use super::errors::TypeCheckError;
use super::pkg::typecheck_pkg;
use super::scope::Scope;

use crate::package::{Composition, Edge, Export, MultiInstanceEdge, OracleSig, PackageInstance};
use crate::parser::package::MultiInstanceIndicesGroup;
use crate::types::Type;
use crate::util::prover_process::ProverResponse;
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
        //self.0.multi_inst_idx.hash(state);
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

    println!("!!!! starting high-confidence checks, probably not false positives !!!!!");

    // 1 (new): for each pkg instance i, for each import j, check that there exists a package instance at the destination that has an oracle sig that has matching name, args and return (ignore indices!)
    for (i, pkg_inst) in pkgs.iter().enumerate() {
        let pkg_edges = edges
            .iter()
            .filter_map(|Edge(src, dst, osig)| if *src == i { Some((dst, osig)) } else { None });
        let pkg_mi_edges = multi_inst_edges.iter().filter_map(
            |MultiInstanceEdge {
                 source_pkgidx: src,
                 dest_pkgidx: dst,
                 oracle_sig,
                 ..
             }| {
                if *src == i {
                    Some((dst, oracle_sig))
                } else {
                    None
                }
            },
        );

        let edges: Vec<_> = pkg_edges.chain(pkg_mi_edges).collect();

        let game_name = comp.name();
        let pkg_inst_name = &pkg_inst.name;

        for import in &pkg_inst.pkg.imports {
            let import_sig = &import.0;

            let single_instance_sigs = edges
                .iter()
                .filter(|(_, edge_sig)| edge_sig.name == import_sig.name);

            let multi_instance_sigs = edges
                .iter()
                .filter(|(_, edge_sig)| edge_sig.name == import_sig.name);

            let mut at_least_one = false;

            for (i_dst, edge_sig) in single_instance_sigs.chain(multi_instance_sigs) {
                at_least_one = true;

                let return_types_match = edge_sig.tipe == import_sig.tipe;
                let arg_lengths_match = edge_sig.args.len() == import_sig.args.len();
                // I think this might fail due to a mismatch in paramenters?
                let arg_types_match = edge_sig.args.iter().zip(import_sig.args.iter()).all(
                    |((_, edge_arg_type), (_, import_arg_type))| edge_arg_type == import_arg_type,
                );

                let dst_pkg_inst_name = &pkgs[**i_dst].name;

                if !arg_lengths_match {
                    return Err(TypeCheckError::TypeCheck(format!(
                    "in game {game_name}, argument count for package import oracle {import_sig:?} in package instance {pkg_inst_name} with id {i:} does not match edge, which has signature {edge_sig:?} and points to package instance {dst_pkg_inst_name}",
                )));
                }

                if !arg_types_match {
                    return Err(TypeCheckError::TypeCheck(format!(
                    "in game {game_name}, argument types for package import oracle {import_sig:?} in package instance {pkg_inst_name} with id {i:} does not match edge, which has signature {edge_sig:?}",
                )));
                }

                if !return_types_match {
                    return Err(TypeCheckError::TypeCheck(format!(
                    "in game {game_name}, return types for package import oracle {import_sig:?} in package instance {pkg_inst_name} with id {i:} does not match edge, which has signature {edge_sig:?}",
                )));
                }
            }

            if !at_least_one {
                return Err(TypeCheckError::TypeCheck(format!(
                    "in game {game_name}, couldn't find an edge for oracle {import_sig:?} that starts in package instance {pkg_inst_name} with id {i:}",
                )));
            }
        }
    }

    println!("!!!! from now on, expect false positives !!!!");

    // 2 (new): for each pkg instance i, for each import j, do the smt checks
    let multi_inst_edges_map = multi_inst_edges
        .iter()
        .map(|mi_edge| {
            (
                (mi_edge.dest_pkgidx, mi_edge.oracle_sig.name.as_str()), // key
                mi_edge,                                                 // edge
            )
        })
        .fold(
            HashMap::<(usize, &str), Vec<_>>::new(),
            |mut map, (key, mi_edge)| {
                if !mi_edge.oracle_sig.multi_inst_idx.indices.is_empty() {
                    map.entry(key)
                        .or_insert(vec![])
                        .push(mi_edge.oracle_sig.multi_inst_idx.clone());
                }
                map
            },
        );

    let consts: Vec<_> = comp
        .consts
        .iter()
        .filter_map(|(name, tipe)| {
            if *tipe == Type::Integer {
                Some(name.as_str())
            } else {
                None
            }
        })
        .collect();

    let mut smt_solver_process = crate::util::prover_process::Communicator::new_cvc5().unwrap();

    for (k, indices) in multi_inst_edges_map.into_iter() {
        let varname = "___foooo___-__-__nla";
        let pkg_inst = &pkgs[k.0];
        let multi_instance_indices = &pkg_inst.multi_instance_indices;
        let assumptions = multi_instance_indices.smt_range_predicate(varname);

        let group = MultiInstanceIndicesGroup::new(indices);
        let smt_statements = group.smt_check_total(vec![assumptions], &consts, "___fooo___");

        for stmt in smt_statements {
            smt_solver_process.write_smt(stmt).unwrap();
        }

        let sat_status = smt_solver_process.check_sat().unwrap();
        println!("{}", sat_status);

        assert_eq!(sat_status, ProverResponse::Unsat);
    }

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

    println!("comparing...");
    if declared_imports != edge_imports {
        println!("done comparing inside branch");
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
    println!("done comparing post branch");

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

    // 3. Check multi instance exports exist
    // TODO

    // 4. Check that multi instance edges are total
    let multi_inst_edges_map = multi_inst_edges
        .iter()
        .map(|mi_edge| {
            (
                (mi_edge.dest_pkgidx, mi_edge.oracle_sig.name.as_str()),
                mi_edge,
            )
        })
        .fold(
            HashMap::<(usize, &str), Vec<_>>::new(),
            |mut map, (key, mi_edge)| {
                if !mi_edge.oracle_sig.multi_inst_idx.indices.is_empty() {
                    map.entry(key)
                        .or_insert(vec![])
                        .push(mi_edge.oracle_sig.multi_inst_idx.clone());
                }
                map
            },
        );

    let consts: Vec<_> = comp
        .consts
        .iter()
        .filter_map(|(name, tipe)| {
            if *tipe == Type::Integer {
                Some(name.as_str())
            } else {
                None
            }
        })
        .collect();

    let mut smt_solver_process = crate::util::prover_process::Communicator::new_cvc5().unwrap();

    for (k, indices) in multi_inst_edges_map.into_iter() {
        let varname = "___foooo___-__-__nla";
        let pkg_inst = &pkgs[k.0];
        let multi_instance_indices = &pkg_inst.multi_instance_indices;
        let assumptions = multi_instance_indices.smt_range_predicate(varname);

        let group = MultiInstanceIndicesGroup::new(indices);
        let smt_statements = group.smt_check_total(vec![assumptions], &consts, "___fooo___");

        for stmt in smt_statements {
            smt_solver_process.write_smt(stmt).unwrap();
        }

        let sat_status = smt_solver_process.check_sat().unwrap();
        println!("{}", sat_status);

        assert_eq!(sat_status, ProverResponse::Unsat);
    }

    //
    //
    // 5. Check that multi instance exports are total

    let mut typed_pkgs = vec![];

    // 6. check all package instances
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
