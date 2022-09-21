use std::{
    collections::{HashMap, HashSet},
    iter::FromIterator,
};

use crate::package::{Composition, Edge, Export, OracleSig};

#[derive(Clone)]
pub struct DiffCell {
    pub pkg_offset: usize,
    pub path: Vec<(usize, OracleSig)>,
}

impl DiffCell {
    fn new(pkg_offset: usize, path: Vec<(usize, OracleSig)>) -> DiffCell {
        DiffCell { pkg_offset, path }
    }

    fn new_base(pkg_offset: usize) -> DiffCell {
        DiffCell::new(pkg_offset, vec![])
    }
}

#[derive(Clone)]
pub struct DiffRow(pub DiffCell, pub DiffCell);

impl DiffRow {
    fn new_base(left_offset: usize, right_offset: usize) -> DiffRow {
        DiffRow(
            DiffCell::new_base(left_offset),
            DiffCell::new_base(right_offset),
        )
    }

    fn push_left(&mut self, offset: usize, sig: OracleSig) {
        self.0.path.push((offset, sig))
    }

    fn push_right(&mut self, offset: usize, sig: OracleSig) {
        self.1.path.push((offset, sig))
    }

    fn push(&mut self, left_offset: usize, right_offset: usize, sig: OracleSig) {
        self.push_left(left_offset, sig.clone());
        self.push_right(right_offset, sig.clone());
    }
}

pub struct Diff(pub Vec<DiffRow>);

impl Diff {
    fn new_base(left_offset: usize, right_offset: usize) -> Diff {
        Diff(vec![DiffRow::new_base(left_offset, right_offset)])
    }

    pub fn is_same(&self) -> bool {
        let Diff(rows) = self;
        rows.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &DiffRow> {
        self.0.iter()
    }
}

pub fn diff(left: &Composition, right: &Composition) -> Diff {
    // TODO: merge edges that are same except for the oracle sig into one
    let left_sigs_to_idx: HashMap<_, _> = left
        .exports
        .iter()
        .map(|Export(idx, sig)| (sig.to_owned(), *idx))
        .collect();
    let right_sigs_to_idx: HashMap<_, _> = right
        .exports
        .iter()
        .map(|Export(idx, sig)| (sig.to_owned(), *idx))
        .collect();

    let left_exported_sigs: HashSet<_> = left_sigs_to_idx.keys().cloned().collect();
    let right_exported_sigs: HashSet<_> = right_sigs_to_idx.keys().cloned().collect();
    assert_eq!(left_exported_sigs, right_exported_sigs);

    assert!(check_parameters_match(left, right));

    let all_diff_rows = left_exported_sigs
        .iter()
        .map(|sig| {
            let left_idx = left_sigs_to_idx[sig];
            let right_idx = right_sigs_to_idx[sig];

            let Diff(diff_rows) = traverse(left, right, left_idx, right_idx);
            diff_rows
        })
        .flatten()
        .collect();

    return Diff(all_diff_rows);
}

//pub fn build_subcomposition(comp: &Composition, roots: &[usize], exports: &[(usize, OracleSig)]) -> Composition

pub fn matches_assumption2(game: CompositionSlice, assumption: &Composition) -> bool {
    let game_exports: HashMap<_, _> = game
        .exports
        .iter()
        .map(|Export(idx, sig)| (sig.clone(), *idx))
        .collect();
    let assumption_exports: HashMap<_, _> = assumption
        .exports
        .iter()
        .map(|Export(idx, sig)| (sig.clone(), *idx))
        .collect();

    let game_sigs: HashSet<&OracleSig> = HashSet::from_iter(game_exports.keys());
    let assumption_sigs = HashSet::from_iter(assumption_exports.keys());

    // check that game does not export too many oracles
    if !game_sigs.is_subset(&assumption_sigs) {
        return false;
    }

    // use each exported oracle as a root to traverse and look for differences
    for (sig, game_idx) in game_exports {
        let assumption_idx: usize = assumption_exports[&sig];

        let diff = traverse(game.comp, assumption, game_idx, assumption_idx);
        if !diff.is_same() {
            panic!("Assumption doesn't match:\n - sig: {:?}\n - game pkg: {:?} at index {}\n - assumption pkg: {:?} at index {}\n", sig, game.comp.pkgs[game_idx], game_idx, assumption.pkgs[assumption_idx], assumption_idx);
        }
    }

    true
}

pub fn matches_assumption(left: &Composition, assumption: &Composition, left_idx: usize) -> bool {
    let left_sigs_to_idx: HashMap<_, _> = left
        .edges
        .iter()
        .filter(|Edge(from, _, _)| *from == left_idx)
        .map(|Edge(from, to, sig)| (sig.to_owned(), *to))
        .collect();
    let left_exported_sigs: HashSet<_> = left
        .exports
        .iter()
        .filter(|Export(idx, _)| *idx == left_idx)
        .map(|Export(_, sig)| sig.to_owned())
        .collect();

    let assumption_sigs_to_idx: HashMap<_, _> = assumption
        .exports
        .iter()
        .map(|Export(idx, sig)| (sig.to_owned(), *idx))
        .collect();

    let left_pointing_to_idx: HashSet<_> = left
        .edges
        .iter()
        .filter(|Edge(_, to, _)| *to == left_idx)
        .map(|Edge(_, _, sig)| sig.to_owned())
        .collect();

    let left_incoming =
        HashSet::from_iter(left_exported_sigs.union(&left_pointing_to_idx).cloned());

    let assumption_exported_sigs: HashSet<_> = assumption_sigs_to_idx.keys().cloned().collect();

    assert!(
        left_incoming.is_subset(&assumption_exported_sigs),
        "left: {:?} | assumption: {:?}",
        left_exported_sigs,
        assumption_exported_sigs
    );

    assert!(check_parameters_match(left, assumption));

    for sig in &left_exported_sigs {
        let left_idx = left_sigs_to_idx[sig];
        let assumption_idx = assumption_sigs_to_idx[sig];

        let Diff(diff_rows) = traverse(left, assumption, left_idx, assumption_idx);

        for DiffRow(mut left, mut assumption) in diff_rows {
            let diff = (left.pkg_offset, assumption.pkg_offset);

            left.path.push((left_idx, sig.clone()));
            assumption.path.push((assumption_idx, sig.clone()));

            panic!("found a difference! \n left_idx {}\n assumption_idx {}\n diff {:?}\n left_path {:?}\n assumption_path {:?}", left_idx, assumption_idx, diff, &left.path, &assumption.path);
            return false;
        }
    }

    true
}

fn traverse(left: &Composition, right: &Composition, left_idx: usize, right_idx: usize) -> Diff {
    // TODO: merge edges that are same except for the oracle sig into one

    if left.pkgs[left_idx].pkg.name != right.pkgs[right_idx].pkg.name {
        return Diff::new_base(left_idx, right_idx);
    }

    let left_sig_to_idx: HashMap<_, _> = left
        .edges
        .iter()
        .filter(|Edge(from, _, _)| *from == left_idx)
        .map(|Edge(_, to, sig)| (sig.to_owned(), *to))
        .collect();

    let right_sig_to_idx: HashMap<_, _> = right
        .edges
        .iter()
        .filter(|Edge(from, _, _)| *from == right_idx)
        .map(|Edge(_, to, sig)| (sig.to_owned(), *to))
        .collect();

    // signatures the index points to on both sides
    let left_to_sigs: HashSet<_> = left_sig_to_idx.keys().cloned().collect();
    let right_to_sigs: HashSet<_> = right_sig_to_idx.keys().cloned().collect();

    if left_to_sigs != right_to_sigs {
        return Diff::new_base(left_idx, right_idx);
    }

    let all_diff_rows = left_to_sigs
        .iter()
        .map(|sig| {
            let next_left_idx = left_sig_to_idx[sig];
            let next_right_idx = right_sig_to_idx[sig];

            let Diff(mut diff_rows) = traverse(left, right, next_left_idx, next_right_idx);
            for row in &mut diff_rows {
                row.push(left_idx, right_idx, sig.clone());
            }
            diff_rows
        })
        .flatten()
        .collect();

    Diff(all_diff_rows)
}

fn check_parameters_match(left: &Composition, right: &Composition) -> bool {
    // TODO
    //also there is a function called check_matching_parameters that lives in equivalence.rs
    true
}

pub struct CompositionSlice<'a> {
    comp: &'a Composition,
    exports: Vec<Export>,
}

pub fn walk_up_paths<'a>(comp: &'a Composition, diff: &[DiffCell]) -> CompositionSlice<'a> {
    let mut exports = vec![];

    let reverse_edges: HashMap<_, _> = comp
        .edges
        .iter()
        .map(|Edge(from, to, sig)| ((*to, sig.to_owned()), *from))
        .collect();

    let mut all_incoming_edges = HashMap::new();

    for Edge(from, to, sig) in &comp.edges {
        all_incoming_edges
            .entry(*to)
            .or_insert(vec![])
            .push((*from, sig.clone()));
    }

    let mut all_slice_internal_edges = HashSet::new();

    // 1st pass: validate inputs and prepare set

    for DiffCell {
        pkg_offset: mut diff_idx,
        path,
    } in diff
    {
        for (next_idx, sig) in path.iter().cloned().rev() {
            // check that path makes sense
            let found_next_idx = *reverse_edges.get(&(diff_idx, sig.clone())).expect(&format!(
                "couldn't walk path as there was no such signature: {diff_idx} {sig:?}"
            ));
            assert_eq!(found_next_idx, next_idx);

            // generate set of internal edges
            all_slice_internal_edges.insert((next_idx, diff_idx, sig));

            // next
            diff_idx = next_idx;
        }
    }

    // 2nd pass: build result

    for DiffCell {
        pkg_offset: mut diff_idx,
        path,
    } in diff
    {
        for (next_idx, sig) in path {
            let edge = (*next_idx, diff_idx, sig.clone());

            // find edges that points outside our compslice, add them as an export
            for (_, sig) in &all_incoming_edges[&diff_idx] {
                if !all_slice_internal_edges.contains(&edge) {
                    exports.push(Export(diff_idx, sig.clone()))
                }
            }

            diff_idx = *next_idx;
        }

        match all_incoming_edges.get(&diff_idx) {
            Some(edges) => {
                for (_, sig) in edges {
                    exports.push(Export(diff_idx, sig.clone()))
                }
            }
            None => {}
        }
    }

    return CompositionSlice { comp, exports };
}
