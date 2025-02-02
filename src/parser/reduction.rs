use std::collections::{HashMap, HashSet};

use itertools::Itertools as _;
use pest::iterators::Pair;

use crate::{
    expressions::Expression,
    gamehops::{
        reduction::{Assumption, Reduction},
        GameHop,
    },
    identifier::{
        game_ident::{GameConstIdentifier, GameIdentifier},
        pkg_ident::PackageConstIdentifier,
        proof_ident::ProofIdentifier,
        Identifier,
    },
    package::{Edge, PackageInstance},
    parser::error::{
        AssumptionMappingLeftGameInstanceIsNotFromAssumption,
        AssumptionMappingRightGameInstanceIsFromAssumption, InvalidGameInstanceInReductionError,
    },
    proof::GameInstance,
    util::resolver::Named,
};

use super::{
    ast::{Identifier as _, *},
    error::{
        AssumptionExportsNotSufficientError, AssumptionMappingContainsDifferentPackagesError,
        AssumptionMappingDuplicatePackageInstanceError,
        AssumptionMappingMissesPackageInstanceError, AssumptionMappingParameterMismatchError,
        ReductionContainsDifferentPackagesError, ReductionInconsistentAssumptionBoundaryError,
        ReductionPackageInstanceParameterMismatchError, UndefinedAssumptionError,
        UndefinedGameInstanceError, UndefinedPackageInstanceError,
    },
    proof::{handle_identifiers, ParseProofContext, ParseProofError},
    Rule,
};

pub(crate) fn handle_reduction<'a>(
    ctx: &mut ParseProofContext<'a>,
    ast: Pair<'a, Rule>,
) -> Result<Vec<GameHop<'a>>, ParseProofError> {
    let mut ast = ast.into_inner();

    let left_name_ast = ast.next().unwrap();
    let right_name_ast = ast.next().unwrap();
    let body_ast = ast.next().unwrap();

    let reduction = handle_reduction_body(ctx, left_name_ast, right_name_ast, body_ast)?;
    Ok(vec![GameHop::Reduction(reduction)])
}

fn compare_reduction(
    ctx: &ParseProofContext,
    reduction_span: pest::Span,
    inst_offs_left: usize,
    inst_offs_right: usize,
    left_game_inst: &GameInstance,
    right_game_inst: &GameInstance,
    mapping_left: &ReductionMapping,
    mapping_right: &ReductionMapping,
) -> Result<(), ParseProofError> {
    let game_left = left_game_inst.game();
    let game_right = right_game_inst.game();

    let left_pkg_inst = &game_left.pkgs[inst_offs_left];
    let right_pkg_inst = &game_right.pkgs[inst_offs_right];

    let left_mapping_entry = mapping_left
        .entries
        .iter()
        .find(|mapping_entry| mapping_entry.construction().as_str() == left_pkg_inst.name());

    let right_mapping_entry = mapping_right
        .entries
        .iter()
        .find(|mapping_entry| mapping_entry.construction().as_str() == right_pkg_inst.name());

    match (left_mapping_entry, right_mapping_entry) {
        // both are in assumption, this is the happy "done" case
        (Some(_), Some(_)) => return Ok(()),

        // only one of them in the assumption, this is an error
        (Some(mapping_entry), None) => {
            println!("asd");
            let mapping_entry_span = mapping_entry.construction().as_span();
            Err(ReductionInconsistentAssumptionBoundaryError {
                source_code: ctx.named_source(),
                at_reduction: (reduction_span.start()..reduction_span.end()).into(),
                at_mapping_entry: (mapping_entry_span.start()..mapping_entry_span.end()).into(),
                game_name_1: game_left.name().to_string(),
                game_name_2: game_right.name().to_string(),
                pkg_inst_name_1: left_pkg_inst.name().to_string(),
                pkg_inst_name_2: right_pkg_inst.name().to_string(),
            })
        }
        (None, Some(mapping_entry)) => {
            println!("fgh");
            let mapping_entry_span = mapping_entry.construction().as_span();
            Err(ReductionInconsistentAssumptionBoundaryError {
                source_code: ctx.named_source(),
                at_reduction: (reduction_span.start()..reduction_span.end()).into(),
                at_mapping_entry: (mapping_entry_span.start()..mapping_entry_span.end()).into(),
                game_name_1: game_right.name().to_string(),
                game_name_2: game_left.name().to_string(),
                pkg_inst_name_1: right_pkg_inst.name().to_string(),
                pkg_inst_name_2: left_pkg_inst.name().to_string(),
            })
        }

        // still in reduction, continue traversing
        (None, None) => Ok(()),
    }?;

    if left_pkg_inst.pkg.name() != right_pkg_inst.pkg.name() {
        return Err(ReductionContainsDifferentPackagesError {
            source_code: ctx.named_source(),
            at_reduction: (reduction_span.start()..reduction_span.end()).into(),

            left_pkg_inst_name: left_pkg_inst.name().to_string(),
            left_pkg_name: left_pkg_inst.pkg_name().to_string(),

            right_pkg_inst_name: right_pkg_inst.name().to_string(),
            right_pkg_name: right_pkg_inst.pkg_name().to_string(),
        }
        .into());
    }

    let diff = package_instances_diff(
        ctx,
        left_pkg_inst,
        left_game_inst,
        right_pkg_inst,
        right_game_inst,
    );

    match diff {
        PackageInstanceDiff::DifferentPackage(left_pkg_name, right_pkg_name) => {
            return Err(ReductionContainsDifferentPackagesError {
                source_code: ctx.named_source(),
                at_reduction: (reduction_span.start()..reduction_span.end()).into(),
                left_pkg_inst_name: left_pkg_inst.name().to_string(),
                right_pkg_inst_name: right_pkg_inst.name().to_string(),

                left_pkg_name,
                right_pkg_name,
            }
            .into());
        }
        PackageInstanceDiff::DifferentParams(param_diff) => {
            return Err(ReductionPackageInstanceParameterMismatchError {
                source_code: ctx.named_source(),
                at_reduction: (reduction_span.start()..reduction_span.end()).into(),
                left_pkg_inst_name: left_pkg_inst.name().to_string(),
                right_pkg_inst_name: right_pkg_inst.name().to_string(),

                param_names: param_diff.iter().map(|(name, _, _)| &name.name).join(", "),
            }
            .into());
        }
        PackageInstanceDiff::Same => {}
    }

    // these are the same in both games, because the package name is the same
    for (sig, _) in &left_pkg_inst.pkg.imports {
        dbg!(sig);
        dbg!(&game_left.edges);

        let left_edge = game_left
            .edges
            .iter()
            .find(|edge| edge.from() == inst_offs_left && edge.sig() == sig)
            .unwrap_or_else(|| {
                let edge = game_left
                    .edges
                    .iter()
                    .find(|edge| edge.from() == inst_offs_left && edge.sig().name == sig.name)
                    .unwrap();

                panic!("failed to find edge {edge:?} by comapring with signature {sig:?}");

                // again, the problem is that there is a Bits(_) type in the signature.
                // what is new is that we are on the level of proofs here, so we need another layer
                // of instantiation here.
                // Or we add a nestable EqWithContext somehow. I am not sure that is a good idea,
                // but let's play with the idea for a bit.
                //
                // Actually let's do that somewhere else.
                //
                // ----
                //
                // Alternative idea: rewrite during instantiation and put that piece of info
                // somewhere. For some things we already do that, but these are just strings, not
                // identifiers. maybe that is the actual problem. Let's try to change that
            });
        let right_edge = game_right
            .edges
            .iter()
            .find(|edge| edge.from() == inst_offs_right && edge.sig() == sig)
            .unwrap();

        compare_reduction(
            ctx,
            reduction_span,
            left_edge.to(),
            right_edge.to(),
            left_game_inst,
            right_game_inst,
            mapping_left,
            mapping_right,
        )?
    }

    Ok(())
}

fn handle_reduction_body<'a>(
    ctx: &mut ParseProofContext,
    left_name: Pair<'a, Rule>,
    right_name: Pair<'a, Rule>,
    body: Pair<'a, Rule>,
) -> Result<Reduction<'a>, ParseProofError> {
    let reduction_span = body.as_span();
    let mut ast = body.into_inner();
    let assumptions_spec_ast = ast.next().unwrap();
    let assumptions_name_ast = assumptions_spec_ast.into_inner().next().unwrap();
    let assumptions_name_span = assumptions_name_ast.as_span();
    let assumption_name = assumptions_name_ast.as_str();

    // check that assumption_name turns up in the assumptions list
    // and fetch the assumption definition
    let assumption = ctx
        .assumptions
        .iter()
        .find(|assumption| assumption.name == assumption_name)
        .ok_or(UndefinedAssumptionError {
            source_code: ctx.named_source(),
            at: (assumptions_name_span.start()..assumptions_name_span.end()).into(),
            assumption_name: assumption_name.to_string(),
        })?;

    // Check that the reduction has different game instances before and after the game hop.
    // This is not technically a problem, but it is a noop and likely unintended
    if left_name.as_str() == right_name.as_str() {
        let name = left_name.as_str();
        let source = ctx.named_source();
        let start = left_name.as_span().start();
        let end = right_name.as_span().end();

        #[derive(miette::Diagnostic, Debug, thiserror::Error)]
        #[error("Reduction hash identical construction game instance {name} left and right")]
        #[diagnostic(severity(Warning))]
        struct SameConstructionGameInstanceWarning {
            #[source_code]
            source_code: miette::NamedSource<String>,

            #[label("these probably should not be the same")]
            span: miette::SourceSpan,

            name: String,
        }

        let diag = SameConstructionGameInstanceWarning {
            source_code: source,
            span: (start..end).into(),
            name: name.to_string(),
        };

        // TODO: find a better way to emit warnings
        log::warn!("{diag:?}")
    }

    let construction_game_instance_names = [left_name.as_str(), right_name.as_str()];

    let map1_ast = ast.next().unwrap();
    let map2_ast = ast.next().unwrap();

    let mapping1 = handle_mapspec_assumption(ctx, map1_ast, assumption)?;
    let mapping2 = handle_mapspec_assumption(ctx, map2_ast, assumption)?;

    let header_span = left_name.as_span().start()..right_name.as_span().end();

    let (mapping_left, mapping_right) =
        if mapping1.construction_game_instance_name().as_str() != right_name.as_str() {
            // we know mapping1 should be for left and mapping2 should be for right

            if mapping1.construction_game_instance_name().as_str() != left_name.as_str() {
                return Err(InvalidGameInstanceInReductionError::new(
                    ctx.named_source(),
                    mapping1.construction_game_instance_name().as_pair(),
                    header_span,
                )
                .into());
            }

            if mapping2.construction_game_instance_name().as_str() != right_name.as_str() {
                return Err(InvalidGameInstanceInReductionError::new(
                    ctx.named_source(),
                    mapping2.construction_game_instance_name().as_pair(),
                    header_span,
                )
                .into());
            }

            (mapping1, mapping2)
        } else {
            // we know mapping1 should be for right and mapping2 should be for left

            if mapping1.construction_game_instance_name().as_str() != right_name.as_str() {
                return Err(InvalidGameInstanceInReductionError::new(
                    ctx.named_source(),
                    mapping1.construction_game_instance_name().as_pair(),
                    header_span,
                )
                .into());
            }

            if mapping2.construction_game_instance_name().as_str() != left_name.as_str() {
                return Err(InvalidGameInstanceInReductionError::new(
                    ctx.named_source(),
                    mapping2.construction_game_instance_name().as_pair(),
                    header_span,
                )
                .into());
            }

            (mapping2, mapping1)
        };

    let left_game_inst = &ctx.game_instance(left_name.as_str()).unwrap().1;
    let right_game_inst = &ctx.game_instance(right_name.as_str()).unwrap().1;

    let left_exports = &left_game_inst.game().exports;

    for left_export in left_exports {
        let right_export = right_game_inst
            .game()
            .exports
            .iter()
            .find(|export| export.sig() == left_export.sig())
            .unwrap();

        compare_reduction(
            ctx,
            reduction_span,
            left_export.to(),
            right_export.to(),
            left_game_inst,
            right_game_inst,
            &mapping_left,
            &mapping_right,
        )?;
    }
    // TODO: implement reduction mapspec and do third check in there
    //let mapping3 = handle_mapspec_reduction(ctx, map3_ast, &mapping1, &mapping2)?;

    // these are the construction game names
    if mapping_left.assumption_game_instance_name().as_str() == mapping_right.assumption.as_str() {
        panic!();
        // TODO reutrn err
    }

    // these are the assumption game names
    if mapping_left.construction_game_instance_name().as_str()
        == mapping_right.construction_game_instance_name().as_str()
    {
        panic!();
        // TODO reutrn err
    }

    let (_, left_game_inst) = ctx
        .game_instance(mapping_left.assumption_game_instance_name().as_str())
        .unwrap();
    let (_, right_game_inst) = ctx
        .game_instance(mapping_right.assumption_game_instance_name().as_str())
        .unwrap();

    let mut unmapped_pkg_insts1: Vec<_> = {
        let mapped_bigger_pkg_insts = mapping_left
            .entries()
            .iter()
            .map(|entry| entry.assumption.as_str())
            .collect::<HashSet<_>>();
        left_game_inst
            .game()
            .pkgs
            .iter()
            .filter(|pkg_inst| !mapped_bigger_pkg_insts.contains(&pkg_inst.name.as_str()))
            .collect()
    };

    unmapped_pkg_insts1.sort_by(|left, right| left.name.cmp(&right.name));

    let mut unmapped_pkg_insts2: Vec<_> = {
        let mapped_bigger_pkg_insts = mapping_right
            .entries()
            .iter()
            .map(|entry| entry.assumption.as_str())
            .collect::<HashSet<_>>();
        right_game_inst
            .game()
            .pkgs
            .iter()
            .filter(|pkg_inst| !mapped_bigger_pkg_insts.contains(&pkg_inst.name.as_str()))
            .collect()
    };

    unmapped_pkg_insts2.sort_by(|left, right| left.name.cmp(&right.name));

    for (left, right) in unmapped_pkg_insts1.iter().zip(unmapped_pkg_insts2.iter()) {
        match package_instances_diff(ctx, left, left_game_inst, right, right_game_inst) {
            PackageInstanceDiff::DifferentPackage(left_pkg_name, right_pkg_name) => {
                return Err(ReductionContainsDifferentPackagesError {
                    source_code: ctx.named_source(),
                    at_reduction: (reduction_span.start()..reduction_span.end()).into(),
                    left_pkg_inst_name: left.name.clone(),
                    right_pkg_inst_name: right.name.clone(),

                    left_pkg_name,
                    right_pkg_name,
                }
                .into());
            }
            PackageInstanceDiff::DifferentParams(param_diff) => {
                return Err(ReductionPackageInstanceParameterMismatchError {
                    source_code: ctx.named_source(),
                    at_reduction: (reduction_span.start()..reduction_span.end()).into(),
                    left_pkg_inst_name: left.name.clone(),
                    right_pkg_inst_name: right.name.clone(),

                    param_names: param_diff.iter().map(|(name, _, _)| &name.name).join(", "),
                }
                .into());
            }
            PackageInstanceDiff::Same => {}
        }
    }

    let reduction = Reduction::new(mapping_left, mapping_right, assumption_name.to_string());

    Ok(reduction)
}

fn handle_mapspec_assumption<'a>(
    ctx: &ParseProofContext,
    ast: Pair<'a, Rule>,
    assumption: &Assumption,
) -> Result<ReductionMapping<'a>, ParseProofError> {
    let mapping_span = ast.as_span();
    let mut ast = ast.into_inner();

    let [assumption_game_inst_name, construction_game_inst_name]: [GameInstanceName; 2] =
        handle_identifiers(&mut ast);
    // let (
    //     (assumption_game_inst_name, assumption_game_inst_name_span),
    //     (construction_game_inst_name, construction_game_inst_name_span),
    // ) = handle_string_pair(&mut ast);

    let assumption_game_inst_name_span = assumption_game_inst_name.as_span();
    let construction_game_inst_name_span = construction_game_inst_name.as_span();

    // check that game instance names can be resolved
    let (_, assumption_game_inst) = ctx
        .game_instance(assumption_game_inst_name.as_str())
        .ok_or(UndefinedGameInstanceError {
            source_code: ctx.named_source(),
            at: (assumption_game_inst_name.as_span().start()..assumption_game_inst_name_span.end())
                .into(),
            game_inst_name: assumption_game_inst_name.as_str().to_string(),
        })?;

    let (_, construction_game_inst) = ctx
        .game_instance(construction_game_inst_name.as_str())
        .ok_or(UndefinedGameInstanceError {
            source_code: ctx.named_source(),
            at: (construction_game_inst_name_span.start()..construction_game_inst_name_span.end())
                .into(),
            game_inst_name: construction_game_inst_name.as_str().to_string(),
        })?;

    //dbg!(construction_game_inst);

    let assumption_game_is_really_assumption_game = assumption_game_inst_name.as_str()
        == assumption.left_name
        || assumption_game_inst_name.as_str() == assumption.right_name;

    let construction_game_is_actually_assumption_game = construction_game_inst_name.as_str()
        == assumption.left_name
        || construction_game_inst_name.as_str() == assumption.right_name;

    if !assumption_game_is_really_assumption_game {
        return Err(AssumptionMappingLeftGameInstanceIsNotFromAssumption {
            source_code: ctx.named_source(),
            at: (assumption_game_inst_name_span.start()..assumption_game_inst_name_span.end())
                .into(),
            game_instance_name: assumption_game_inst_name.as_str().to_string(),
            assumption_left_game_instance_name: assumption.left_name.clone(),
            assumption_right_game_instance_name: assumption.right_name.clone(),
        }
        .into());
    }

    if construction_game_is_actually_assumption_game {
        return Err(AssumptionMappingRightGameInstanceIsFromAssumption {
            source_code: ctx.named_source(),
            at: (construction_game_inst_name_span.start()..construction_game_inst_name_span.end())
                .into(),
            game_instance_name: construction_game_inst_name.as_str().to_string(),
            model_left_game_instance_name: assumption_game_inst_name.as_str().to_string(),
            model_right_game_instance_name: construction_game_inst_name.as_str().to_string(),
        }
        .into());
    }

    let mappings: Vec<_> = ast
        .flat_map(|p| p.into_inner().map(PackageInstanceName::from))
        .tuples()
        .collect();

    let mut assumption_game_pkg_inst_names: HashMap<String, &PackageInstanceName> = HashMap::new();
    let mut construction_game_pkg_inst_names: HashMap<String, &PackageInstanceName> =
        HashMap::new();

    let mut pkg_offset_mapping = vec![];

    // check mappings are valid
    for (assumption_game_pkg_inst_name_ast, construction_game_pkg_inst_name_ast) in &mappings {
        let assumption_game_pkg_inst_name = assumption_game_pkg_inst_name_ast.as_str();
        let construction_game_pkg_inst_name = construction_game_pkg_inst_name_ast.as_str();

        // check for duplicates
        if let Some(prev_map) = assumption_game_pkg_inst_names.get(assumption_game_pkg_inst_name) {
            let span_this = assumption_game_pkg_inst_name_ast.as_span();
            let at_this = (span_this.start()..span_this.end()).into();

            let span_prev = prev_map.as_span();
            let at_prev = (span_prev.start()..span_prev.end()).into();

            let pkg_inst_name = assumption_game_pkg_inst_name.to_string();

            return Err(AssumptionMappingDuplicatePackageInstanceError {
                source_code: ctx.named_source(),
                at_this,
                at_prev,
                pkg_inst_name,
            }
            .into());
        }

        if let Some(prev_map) =
            construction_game_pkg_inst_names.get(construction_game_pkg_inst_name)
        {
            let span_this = construction_game_pkg_inst_name_ast.as_span();
            let at_this = (span_this.start()..span_this.end()).into();

            let span_prev = prev_map.as_span();
            let at_prev = (span_prev.start()..span_prev.end()).into();

            let pkg_inst_name = assumption_game_pkg_inst_name.to_string();

            return Err(AssumptionMappingDuplicatePackageInstanceError {
                source_code: ctx.named_source(),
                at_this,
                at_prev,
                pkg_inst_name,
            }
            .into());
        }
        assumption_game_pkg_inst_names.insert(
            assumption_game_pkg_inst_name.to_string(),
            assumption_game_pkg_inst_name_ast,
        );
        construction_game_pkg_inst_names.insert(
            construction_game_pkg_inst_name.to_string(),
            construction_game_pkg_inst_name_ast,
        );

        // get the package instances
        let Some((assumption_game_pkg_inst_offset, assumption_game_pkg_inst)) =
            assumption_game_inst
                .game()
                .pkgs
                .iter()
                .position(|pkg_inst| assumption_game_pkg_inst_name == pkg_inst.name)
                .map(|offs| (offs, &assumption_game_inst.game().pkgs[offs]))
        else {
            let span = assumption_game_pkg_inst_name_ast.as_span();
            let at = (span.start()..span.end()).into();
            return Err(UndefinedPackageInstanceError {
                source_code: ctx.named_source(),
                at,
                pkg_inst_name: assumption_game_pkg_inst_name.to_string(),
                in_game: assumption_game_inst.game().name.clone(),
            }
            .into());
        };

        let Some((construction_game_pkg_inst_offset, construction_game_pkg_inst)) =
            construction_game_inst
                .game()
                .pkgs
                .iter()
                .position(|pkg_inst| construction_game_pkg_inst_name == pkg_inst.name)
                .map(|offs| (offs, &construction_game_inst.game().pkgs[offs]))
        else {
            let span = construction_game_pkg_inst_name_ast.as_span();
            let at = (span.start()..span.end()).into();
            return Err(ParseProofError::from(UndefinedPackageInstanceError {
                source_code: ctx.named_source(),
                at,
                pkg_inst_name: assumption_game_pkg_inst_name.to_string(),
                in_game: construction_game_inst.game().name.clone(),
            }));
        };

        // check that the mapped package instances are equivalent, i.e. same package and same
        // parameters.
        match package_instances_diff(
            ctx,
            assumption_game_pkg_inst,
            assumption_game_inst,
            construction_game_pkg_inst,
            construction_game_inst,
        ) {
            PackageInstanceDiff::DifferentPackage(assumption_pkg_name, construction_pkg_name) => {
                let span_assumption = assumption_game_pkg_inst_name_ast.as_span();
                let at_assumption = (span_assumption.start()..span_assumption.end()).into();

                let span_construction = construction_game_pkg_inst_name_ast.as_span();
                let at_construction = (span_construction.start()..span_construction.end()).into();

                let assumption_pkg_inst_name = assumption_game_pkg_inst_name.to_string();
                let construction_pkg_inst_name = construction_game_pkg_inst_name.to_string();

                return Err(AssumptionMappingContainsDifferentPackagesError {
                    source_code: ctx.named_source(),
                    at_assumption,
                    at_construction,

                    assumption_pkg_inst_name,
                    construction_pkg_inst_name,

                    assumption_pkg_name,
                    construction_pkg_name,
                }
                .into());
            }
            PackageInstanceDiff::DifferentParams(different_params) => {
                let span_assumption = assumption_game_pkg_inst_name_ast.as_span();
                let at_assumption = (span_assumption.start()..span_assumption.end()).into();

                let span_construction = construction_game_pkg_inst_name_ast.as_span();
                let at_construction = (span_construction.start()..span_construction.end()).into();

                let assumption_pkg_inst_name = assumption_game_pkg_inst_name.to_string();
                let construction_pkg_inst_name = construction_game_pkg_inst_name.to_string();

                let param_names = different_params
                    .iter()
                    .map(|(id, _, _)| &id.name)
                    .join(", ");

                return Err(AssumptionMappingParameterMismatchError {
                    source_code: ctx.named_source(),
                    at_assumption,
                    at_construction,

                    assumption_pkg_inst_name,
                    construction_pkg_inst_name,

                    param_names,
                }
                .into());
            }
            PackageInstanceDiff::Same => {}
        }

        pkg_offset_mapping.push((
            construction_game_pkg_inst_offset,
            assumption_game_pkg_inst_offset,
        ));
    }

    // finished reading all mappings. now check we mapped all package instances of the assumption.

    for pkg_inst in &assumption_game_inst.game().pkgs {
        if !assumption_game_pkg_inst_names.contains_key(&pkg_inst.name) {
            let span = mapping_span;
            let at = (span.start()..span.end()).into();
            let pkg_inst_name = pkg_inst.name.to_string();
            return Err(AssumptionMappingMissesPackageInstanceError {
                source_code: ctx.named_source(),
                at,

                pkg_inst_name,
            }
            .into());
        }
    }

    // cross-cut wires: check that all wires from the reduction subgraph into
    // the assumption subgraph point to oracles which the assumption game exports
    for Edge(constr_src, constr_dst, constr_sig) in &construction_game_inst.game().edges {
        let dst_pkginst_mapping = pkg_offset_mapping
            .iter()
            .find(|(constr, _)| *constr == *constr_dst);

        // constr_src is not in the mapping => it's in the reduction part
        let src_is_in_reduction_part = pkg_offset_mapping
            .iter()
            .all(|(constr, _)| *constr != *constr_src);

        if constr_sig.name == "Get" {
            let constr_pkgs: Vec<_> = construction_game_inst
                .game()
                .pkgs
                .iter()
                .enumerate()
                .map(|(i, pkg_inst)| (i, pkg_inst.name(), pkg_inst.pkg_name()))
                .collect();
            let assump_pkgs: Vec<_> = assumption_game_inst
                .game()
                .pkgs
                .iter()
                .enumerate()
                .map(|(i, pkg_inst)| (i, pkg_inst.name(), pkg_inst.pkg_name()))
                .collect();

            dbg!(&construction_game_inst_name);
            dbg!(&construction_game_inst.game_name());
            dbg!(&assumption_game_inst_name);
            dbg!(&assumption_game_inst.game_name());
            dbg!(constr_src);
            dbg!(constr_dst);
            dbg!(constr_sig);
            dbg!(dst_pkginst_mapping);
            dbg!(src_is_in_reduction_part);
            dbg!(assump_pkgs);
            dbg!(constr_pkgs);
        }

        // ignore edges that start in mapped packages, because we are only interested in cross-cut
        // edges
        if !src_is_in_reduction_part {
            continue;
        }

        if let Some((_, assump_dst)) = dst_pkginst_mapping {
            let constr_dst_pkg_inst = &construction_game_inst.game().pkgs[*constr_dst];
            let assump_dst_pkg_inst = &assumption_game_inst.game().pkgs[*assump_dst];

            dbg!(&constr_dst_pkg_inst.pkg.name);
            dbg!(&assump_dst_pkg_inst.pkg.name);

            let assump_dst_export = assumption_game_inst
                .game()
                .exports
                .iter()
                .find(|export| export.to() == *assump_dst && export.sig() == constr_sig);

            dbg!(assump_dst_export);

            // if it's cross-cut, it needs to be exported, else error out
            if assump_dst_export.is_none() {
                let assumption_dst_name = &assumption_game_inst.game().pkgs[*assump_dst].name;
                let (assumption_ast, construction_ast) = mappings
                    .iter()
                    .find(|pair| pair.0.as_str() == assumption_dst_name)
                    .unwrap();

                let assumption_span = assumption_ast.as_span();
                let assumption_at = (assumption_span.start()..assumption_span.end()).into();

                let construction_span = construction_ast.as_span();
                let construction_at = (construction_span.start()..construction_span.end()).into();

                let assumption_pkg_inst_name =
                    assumption_game_inst.game().pkgs[*assump_dst].name.clone();
                let construction_pkg_inst_name =
                    construction_game_inst.game().pkgs[*constr_dst].name.clone();
                let oracle_name = constr_sig.name.clone();

                return Err(AssumptionExportsNotSufficientError {
                    source_code: ctx.named_source(),
                    assumption_at,
                    construction_at,
                    assumption_pkg_inst_name,
                    construction_pkg_inst_name,
                    oracle_name,
                }
                .into());
            }

            // check that the destination packages in construction and assumption are equivalent

            match package_instances_diff(
                ctx,
                constr_dst_pkg_inst,
                construction_game_inst,
                assump_dst_pkg_inst,
                assumption_game_inst,
            ) {
                PackageInstanceDiff::DifferentPackage(_, _) => todo!(),
                PackageInstanceDiff::DifferentParams(vec) => todo!(),
                PackageInstanceDiff::Same => {}
            }
        }
    }

    // assumption wires: check that the assumption subgraph of the
    // construction game is wired just like the assumption game graph
    for (constr_src_pkg_inst_offs, assump_src_pkg_inst_offs) in &pkg_offset_mapping {
        // all edges in the construction game that start at the package instance we are currently
        // looking at. We are only looking at mapped package instances, so we know it is part
        // of the assumption part of the construction game.
        let construction_game_assumptionpart_edges = construction_game_inst
            .game()
            .edges
            .iter()
            .filter(|edge| edge.from() == *constr_src_pkg_inst_offs);

        // edge exists in construction => edge exists in assumption
        for construction_edge in construction_game_assumptionpart_edges {
            dbg!(construction_edge);
            dbg!(
                &construction_game_inst.game().pkgs[*constr_src_pkg_inst_offs]
                    .pkg
                    .name
            );
            dbg!(
                &assumption_game_inst.game().pkgs[*assump_src_pkg_inst_offs]
                    .pkg
                    .name
            );
            let (_, assumption_game_to) = pkg_offset_mapping
                .iter()
                .find(|(constr, _)| *constr == construction_edge.to())
                .unwrap();

            dbg!(
                &construction_game_inst.game().pkgs[construction_edge.to()]
                    .pkg
                    .name
            );

            dbg!(
                &assumption_game_inst.game().pkgs[*assumption_game_to]
                    .pkg
                    .name
            );
            let edge_exists_in_assumption =
                assumption_game_inst
                    .game()
                    .edges
                    .iter()
                    .any(|assumption_game_edge| {
                        assumption_game_edge.from() == *assump_src_pkg_inst_offs
                            && assumption_game_edge.to() == *assumption_game_to
                            && construction_edge.sig() == assumption_game_edge.sig()
                    });

            if !edge_exists_in_assumption {
                panic!("assumption wires 1")
            }
        }

        // all edges in the assumtion game that start at the package instance we are currently
        // looking at.
        let assumption_game_edges = assumption_game_inst
            .game()
            .edges
            .iter()
            .filter(|edge| edge.0 == *assump_src_pkg_inst_offs);

        // edge exists in assumption => edge exists in construction
        for assumption_edge in assumption_game_edges {
            let (construction_game_to, _) = pkg_offset_mapping
                .iter()
                .find(|(_, assumpt)| *assumpt == assumption_edge.1)
                .unwrap();

            let edge_exists_in_construction =
                construction_game_inst
                    .game()
                    .edges
                    .iter()
                    .any(|construction_game_edge| {
                        construction_game_edge.0 == *constr_src_pkg_inst_offs
                            && construction_game_edge.1 == *construction_game_to
                    });

            if !edge_exists_in_construction {
                println!(
                    "couldn't find edge in construction that corresponds to edge in assumption."
                );
                println!("construction game: {}", construction_game_inst.game_name());
                println!("assumption game: {}", assumption_game_inst.game_name());
                println!("assumption edge: {assumption_edge:?}");

                println!(
                    "assumption src pkg inst name: {}",
                    assumption_game_inst.game().pkgs[assumption_edge.0].as_name()
                );
                println!(
                    "assumption dst pkg inst name: {}",
                    assumption_game_inst.game().pkgs[assumption_edge.1].as_name()
                );

                println!(
                    "construction src pkg inst name: {}",
                    construction_game_inst.game().pkgs[*constr_src_pkg_inst_offs].as_name()
                );

                panic!("assumption wires 2")
            }
        }
    }

    let mapping = ReductionMapping {
        assumption: assumption_game_inst_name,
        construction: construction_game_inst_name,
        entries: mappings
            .into_iter()
            .map(|(left, right)| NewReductionMappingEntry {
                assumption: left,
                construction: right,
            })
            .collect_vec(),
    };
    Ok(mapping)
}

fn package_instances_diff(
    ctx: &ParseProofContext,
    left_pkg_inst: &PackageInstance,
    left_game_inst: &GameInstance,
    right_pkg_inst: &PackageInstance,
    right_game_inst: &GameInstance,
) -> PackageInstanceDiff {
    if left_pkg_inst.pkg.name != right_pkg_inst.pkg.name {
        return PackageInstanceDiff::DifferentPackage(
            left_pkg_inst.pkg.name.clone(),
            right_pkg_inst.pkg.name.clone(),
        );
    }

    // if parsing went well so far, all parameters should have been assigned. since they are
    // the same package type, they need to have the same parameters. for the next check we need
    // to be able to rely on that, so we just make sure here.
    assert_eq!(left_pkg_inst.params.len(), right_pkg_inst.params.len());

    let mut different_params = vec![];

    // compare the values of individual parameters
    for (param_ident, left_param_expr) in &left_pkg_inst.params {
        let (_, right_param_expr) = right_pkg_inst
                .params
                .iter()
                .find(|(other_param_ident, _)| param_ident.name == other_param_ident.name)
            .unwrap_or_else(|| panic!("expected both package instances {} and {} in mapping for reduction between games instances {} and {} to contain parameter {}",
                left_pkg_inst.name, right_pkg_inst.name, left_game_inst.name(), right_game_inst.name(), param_ident.ident(),
            ));

        // here we compare whether param_expr and other_param_expr match.
        // problem 1: they have identifiers in them that contain things like game names
        // problem 2: we have the game identifiers here, but we need the proof identifiers,
        //            because otherwise we just compare the name strings used in the game
        //            and ignore the values assigned to the in game instantiation
        //
        // we solve both problems using Expression::map, which both resolves the game identifiers
        // to proof identifiers and redacts game- and package-specific information.

        fn resolve_and_redact_expr(
            game_inst_const_mapping: &[(GameConstIdentifier, Expression)],
            expr: Expression,
        ) -> Expression {
            match expr {
                // redact game and package specific information from proof identifiers
                Expression::Identifier(Identifier::ProofIdentifier(ProofIdentifier::Const(
                    mut proof_const,
                ))) => {
                    proof_const.inst_info = None;
                    Expression::Identifier(proof_const.into())
                }
                Expression::Identifier(Identifier::ProofIdentifier(ProofIdentifier::LoopVar(
                    mut proof_loopvar,
                ))) => {
                    proof_loopvar.inst_info = None;
                    Expression::Identifier(proof_loopvar.into())
                }

                // resolve game const identifiers
                Expression::Identifier(Identifier::GameIdentifier(GameIdentifier::Const(
                    ref game_const,
                ))) => {
                    let (_, assigned_expr) = game_inst_const_mapping
                        .iter()
                        .find(|(game_inst_param, _)| game_inst_param.name == game_const.name)
                        // This should have been caught by the type checker, so we assume it can't
                        // happen and panic
                        .unwrap_or_else(|| panic!("couldn't find identifier {game_const:?} in const mapping {game_inst_const_mapping:?}"));

                    assigned_expr.map(|expr| resolve_and_redact_expr(game_inst_const_mapping, expr))
                }

                // leave the rest
                other => other,
            }
        }

        let redacted_left_param_expr =
            left_param_expr.map(|expr| resolve_and_redact_expr(&left_game_inst.consts, expr));
        let redacted_right_param_expr =
            right_param_expr.map(|expr| resolve_and_redact_expr(&right_game_inst.consts, expr));

        // println!("comparing {}", param_ident.ident_ref());
        // println!("  left:  {left_param_expr:?}");
        // println!("   redacted:  {redacted_left_param_expr:?}");
        // println!("  right: {right_param_expr:?}");
        // println!("   redacted:  {redacted_right_param_expr:?}");

        if redacted_left_param_expr != redacted_right_param_expr {
            different_params.push((
                param_ident.clone(),
                left_param_expr.clone(),
                right_param_expr.clone(),
            ));
        }
    }

    if !different_params.is_empty() {
        PackageInstanceDiff::DifferentParams(different_params)
    } else {
        PackageInstanceDiff::Same
    }
}

enum PackageInstanceDiff {
    DifferentPackage(String, String),
    DifferentParams(Vec<(PackageConstIdentifier, Expression, Expression)>),
    Same,
}

// ----

#[derive(Clone, Debug)]
pub struct ReductionMapping<'a> {
    assumption: GameInstanceName<'a>,
    construction: GameInstanceName<'a>,

    entries: Vec<NewReductionMappingEntry<'a>>,
}

impl<'a> ReductionMapping<'a> {
    pub(crate) fn assumption_game_instance_name(&self) -> &GameInstanceName<'a> {
        &self.assumption
    }

    pub(crate) fn construction_game_instance_name(&self) -> &GameInstanceName<'a> {
        &self.construction
    }

    pub(crate) fn entries(&self) -> &[NewReductionMappingEntry<'a>] {
        &self.entries
    }
}

#[derive(Clone, Debug)]
pub struct NewReductionMappingEntry<'a> {
    assumption: PackageInstanceName<'a>,
    construction: PackageInstanceName<'a>,
}

#[derive(Clone, Debug)]
pub struct NewAssumption<'a> {
    name: AssumptionName<'a>,
    left: GameInstanceName<'a>,
    right: GameInstanceName<'a>,
}

impl<'a> NewReductionMappingEntry<'a> {
    pub(crate) fn assumption(&self) -> &PackageInstanceName<'a> {
        &self.assumption
    }

    pub(crate) fn construction(&self) -> &PackageInstanceName<'a> {
        &self.construction
    }
}
