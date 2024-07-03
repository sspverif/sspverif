use crate::{
    expressions::Expression,
    identifier::{
        proof_ident::{ProofConstIdentifier, ProofIdentifier::Const},
        Identifier,
    },
    package::{Composition, Package},
    parser::Rule,
    proof::{Assumption, Claim, Equivalence, GameHop, GameInstance, Mapping, Proof, Reduction},
    types::Type,
    util::{
        resolver::{Resolver, SliceResolver},
        scope::{Declaration, Scope},
    },
};

use itertools::Itertools;
use pest::{
    iterators::{Pair, Pairs},
    Span,
};

use super::common;
use super::error::{Error, Result};

pub fn handle_proof<'a>(
    ast: Pair<Rule>,
    scope: &mut Scope,
    pkgs: &[Package],
    games: &[Composition],
    file_name: &str,
) -> Result<Proof> {
    let mut iter = ast.into_inner();
    let proof_name = iter.next().unwrap().as_str();
    let proof_ast = iter.next().unwrap();

    let mut assumptions = vec![];
    let mut game_hops = vec![];
    let mut instances = vec![];
    let mut consts = vec![];

    scope.enter();

    for ast in proof_ast.into_inner() {
        match ast.as_rule() {
            Rule::const_decl => {
                let (const_name, tipe) = common::handle_const_decl(ast);
                let declaration = Declaration::Identifier(Identifier::ProofIdentifier(Const(
                    ProofConstIdentifier {
                        proof_name: proof_name.to_string(),
                        name: const_name.clone(),
                        tipe: tipe.clone(),
                    },
                )));
                scope.declare(&const_name, declaration);
                consts.push((const_name, tipe));
            }
            Rule::assumptions => {
                let more_assumptions = handle_assumptions(ast.into_inner(), &instances)?;
                assumptions.extend(more_assumptions);
            }
            Rule::game_hops => {
                let more_game_hops =
                    handle_game_hops(ast.into_inner(), &assumptions, games, &instances)?;
                game_hops.extend(more_game_hops);
            }
            Rule::instance_decl => {
                instances.push(handle_instance_decl(
                    ast, scope, &consts, pkgs, games, file_name,
                )?);
            }
            otherwise => unreachable!("found {:?} in proof", otherwise),
        }
    }

    Ok(Proof::new(
        proof_name.to_string(),
        consts,
        instances,
        assumptions,
        game_hops,
        //games.to_vec(),
        pkgs.to_vec(),
    ))
}

fn handle_instance_decl(
    ast: Pair<Rule>,
    scope: &mut Scope,
    proof_consts: &[(String, Type)],
    pkgs: &[Package],
    games: &[Composition],
    file_name: &str,
) -> Result<GameInstance> {
    let span = ast.as_span();

    let mut ast = ast.into_inner();

    let inst_name = ast.next().unwrap().as_str().to_string();
    let game_ast = ast.next().unwrap();
    let game_span = game_ast.as_span();
    let game_name = game_ast.as_str().to_string();
    let body_ast = ast.next().unwrap();

    let (types, consts) =
        handle_instance_assign_list(scope, &inst_name, file_name, proof_consts, body_ast)?;

    let game_resolver = SliceResolver(games);
    let game = match game_resolver.resolve_value(&game_name) {
        Some(game) => game,
        None => return Err(Error::UndefinedGame(game_name.to_string()).with_span(game_span)),
    };

    let game_inst = GameInstance::new(inst_name, game.clone(), types, consts);

    check_consts(&game_inst, span, games)?;

    Ok(game_inst)
}

pub fn handle_params_def_list(
    ast: Pair<Rule>,
    scope: &mut Scope,
    inst_name: &str,
    game: &Composition,
) -> Result<Vec<(String, Expression)>> {
    ast.into_inner()
        .map(|inner| {
            //let inner = inner.into_inner().next().unwrap();

            let mut inner = inner.into_inner();
            let name_ast = inner.next().unwrap();
            let name_span = name_ast.as_span();
            let name = name_ast.as_str();

            if !game.consts.iter().any(|(const_name, _)| const_name == name) {
                return Err(Error::GameConstParameterUndeclared {
                    game_name: game.name.clone(),
                    inst_name: inst_name.to_string(),
                    param_name: name.to_string(),
                }
                .with_span(name_span));
            }

            let left = inner.next().unwrap().as_str();
            let right = inner.next().unwrap();
            let right = common::handle_expression(right, scope)?;

            Ok((left.to_owned(), right))
        })
        .collect()
}

fn check_consts(game_inst: &GameInstance, span: Span, games: &[Composition]) -> Result<()> {
    let mut inst_const_names: Vec<_> = game_inst
        .consts()
        .iter()
        .map(|(name, _)| name.to_string())
        .collect();
    inst_const_names.sort();

    let game_resolver = SliceResolver(games);
    let game = game_resolver
        .resolve_value(game_inst.game_name())
        .ok_or(Error::UndefinedGame(game_inst.game_name().to_string()).with_span(span.clone()))?;

    let mut game_const_names: Vec<_> = game
        .consts
        .iter()
        .map(|(name, _)| name.to_string())
        .collect();
    game_const_names.sort();

    if inst_const_names != game_const_names {
        return Err(Error::GameConstParameterMismatch {
            game_name: game.name.clone(),
            inst_name: game_inst.name().to_string(),
            bound_params: game_inst.consts().to_vec(),
            game_params: game.consts.clone(),
        }
        .with_span(span));
    }

    Ok(())
}

fn handle_instance_assign_list(
    scope: &mut Scope,
    inst_name: &str,
    file_name: &str,
    proof_consts: &[(String, Type)],
    ast: Pair<Rule>,
) -> Result<(Vec<(String, Type)>, Vec<(String, Expression)>)> {
    let ast = ast.into_inner();

    let mut types = vec![];
    let mut consts = vec![];

    for ast in ast {
        match ast.as_rule() {
            Rule::types_def => {
                let ast = ast.into_inner().next().unwrap();
                types.extend(common::handle_types_def_list(ast, inst_name, file_name)?);
            }
            Rule::params_def => {
                let ast = ast.into_inner().next().unwrap();
                consts.extend(common::handle_proof_params_def_list(
                    ast,
                    proof_consts,
                    scope,
                )?);
            }
            otherwise => {
                unreachable!("unexpected {:?} at {:?}", otherwise, ast.as_span())
            }
        }
    }

    Ok((types, consts))
}

fn handle_assumptions(ast: Pairs<Rule>, instances: &[GameInstance]) -> Result<Vec<Assumption>> {
    let mut out = vec![];

    for pair in ast {
        let span = pair.as_span();
        let (name, left_name, right_name) = handle_string_triplet(&mut pair.into_inner());

        let inst_resolver = SliceResolver(instances);
        if inst_resolver.resolve_value(&left_name).is_none() {
            return Err(Error::UndefinedGameInstance(left_name).with_span(span.clone()));
        }
        if inst_resolver.resolve_value(&right_name).is_none() {
            return Err(Error::UndefinedGameInstance(right_name).with_span(span.clone()));
        }

        out.push(Assumption {
            name,
            left_name,
            right_name,
        });
    }

    Ok(out)
}

fn handle_game_hops(
    ast: Pairs<Rule>,
    assumptions: &[Assumption],
    games: &[Composition],
    game_instances: &[GameInstance],
) -> Result<Vec<GameHop>> {
    let mut out = vec![];

    for hop_ast in ast {
        let game_hop = match hop_ast.as_rule() {
            Rule::equivalence => handle_equivalence(hop_ast, game_instances)?,
            Rule::reduction => handle_reduction(hop_ast, assumptions, game_instances)?,
            otherwise => unreachable!("found {:?} in game_hops", otherwise),
        };
        out.extend(game_hop);
    }

    Ok(out)
}

fn handle_equivalence<'a>(
    ast: Pair<Rule>,
    game_instances: &[GameInstance],
) -> Result<Vec<GameHop>> {
    let span = ast.as_span();
    let mut ast = ast.into_inner();
    let (left_name, right_name) = handle_string_pair(&mut ast);

    let mut equivalences = vec![];

    let equivalence_data: Vec<_> = ast.map(handle_equivalence_oracle).collect();

    let trees: Vec<_> = equivalence_data
        .iter()
        .cloned()
        .map(|(oracle_name, _, lemmas)| {
            (
                oracle_name,
                lemmas.into_iter().map(Claim::from_tuple).collect(),
            )
        })
        .collect();

    let invariants: Vec<_> = equivalence_data
        .iter()
        .cloned()
        .map(|(oracle_name, inv_paths, _)| (oracle_name, inv_paths))
        .collect();

    if SliceResolver(game_instances)
        .resolve_value(&left_name)
        .is_none()
    {
        return Err(Error::UndefinedGameInstance(left_name).with_span(span));
    }
    if SliceResolver(game_instances)
        .resolve_value(&right_name)
        .is_none()
    {
        return Err(Error::UndefinedGameInstance(right_name).with_span(span));
    }

    let eq = GameHop::Equivalence(Equivalence::new(left_name, right_name, invariants, trees));

    equivalences.push(eq);

    Ok(equivalences)
}

fn handle_equivalence_oracle(ast: Pair<Rule>) -> (String, Vec<String>, Vec<(String, Vec<String>)>) {
    let mut ast = ast.into_inner();
    let oracle_name = ast.next().unwrap().as_str().to_string();
    let invariant_paths = handle_invariant_spec(next_pairs(&mut ast));
    let lemmas = handle_lemmas_spec(next_pairs(&mut ast));

    (oracle_name, invariant_paths, lemmas)
}

fn handle_invariant_spec(ast: Pairs<Rule>) -> Vec<String> {
    ast.map(|ast| ast.as_str().to_string()).collect()
}

fn handle_lemmas_spec(ast: Pairs<Rule>) -> Vec<(String, Vec<String>)> {
    ast.map(handle_lemma_line).collect()
}

fn handle_lemma_line(ast: Pair<Rule>) -> (String, Vec<String>) {
    let mut ast = ast.into_inner();
    let name = next_str(&mut ast).to_string();
    let deps = ast.map(|dep| dep.as_str().to_string()).collect();

    (name, deps)
}

fn handle_reduction(
    ast: Pair<Rule>,
    assumptions: &[Assumption],
    game_instances: &[GameInstance],
) -> Result<Vec<GameHop>> {
    let mut ast = ast.into_inner();
    let (left_name, right_name) = handle_string_pair(&mut ast);
    let reduction = handle_reduction_body(
        next_pair(&mut ast),
        assumptions,
        game_instances,
        &left_name,
        &right_name,
    )?;

    Ok(vec![GameHop::Reduction(reduction)])
}

fn handle_reduction_body(
    ast: Pair<Rule>,
    assumptions: &[Assumption],
    game_instances: &[GameInstance],
    left_name: &str,
    right_name: &str,
) -> Result<Reduction> {
    let mut ast = ast.into_inner();
    let assumptions_ast = ast.next().unwrap();
    let assumptions_span = assumptions_ast.as_span();
    let mut assumptions_ast = assumptions_ast.into_inner();
    let assumption_name = next_str(&mut assumptions_ast);

    // check that assumption_name turns up in the assumptions list
    // and fetch the assumption definition
    let assumption_resolver = SliceResolver(assumptions);
    let assumption = assumption_resolver.resolve_value(assumption_name).ok_or(
        Error::ReductionMappingMismatch {
            place: "assumptions definition".to_string(),
        }
        .with_span(assumptions_span),
    )?;

    let map1_ast = ast.next().unwrap();
    let map2_ast = ast.next().unwrap();

    let mapping1 = handle_mapspec(map1_ast, &assumption, game_instances)?;
    let mapping2 = handle_mapspec(map2_ast, &assumption, game_instances)?;

    if mapping1.as_game_inst_name() == mapping2.as_game_inst_name() {
        panic!();
        // TODO reutrn err
    }

    if mapping1.as_assumption_game_inst_name() == mapping2.as_assumption_game_inst_name() {
        panic!();
        // TODO reutrn err
    }

    let game1_is_left = mapping1.as_game_inst_name() == left_name;
    let (left, right) = if game1_is_left {
        (mapping1, mapping2)
    } else {
        (mapping2, mapping1)
    };

    let reduction = Reduction::new(left, right, assumption_name.to_string());

    Ok(reduction)
}

fn handle_mapspec<'a>(
    ast: Pair<Rule>,
    assumption: &Assumption,
    game_instances: &'a [GameInstance],
) -> Result<Mapping> {
    let span = ast.as_span();

    let mut ast = ast.into_inner();

    let (assumption_game_inst_name, game_inst_name) = handle_string_pair(&mut ast);

    // check that game instance names can be resolved
    SliceResolver(game_instances)
        .resolve_value(&game_inst_name)
        .ok_or(Error::UndefinedGame(game_inst_name.clone()).with_span(span.clone()))?;
    SliceResolver(game_instances)
        .resolve_value(&assumption_game_inst_name)
        .ok_or(Error::UndefinedGame(assumption_game_inst_name.clone()).with_span(span.clone()))?;

    let is_left_assumption_game = assumption_game_inst_name == assumption.left_name;
    let is_right_assumption_game = assumption_game_inst_name == assumption.right_name;

    if !(is_left_assumption_game || is_right_assumption_game) {
        println!("{assumption:?}");
        return Err(Error::InvalidAssumptionMapping(format!(
            "neither of {} and {} are the assumption game name {}",
			assumption.left_name, assumption.right_name, assumption_game_inst_name
        ))
        .with_span(span));
    }

    if is_left_assumption_game && is_right_assumption_game {
        println!("{assumption:?}");
        return Err(
            Error::InvalidAssumptionMapping(format!("both are assumption game names"))
                .with_span(span),
        );
    }

    let mappings: Vec<(String, String)> = ast
        .map(Pair::into_inner)
        .flatten()
        .map(|pair| pair.as_str())
        .map(str::to_string)
        .tuples()
        .collect();

    // TODO check mappings are valid
    for (assumption_const, game_const) in &mappings {}

    if assumption.left_name != assumption_game_inst_name
        && assumption.right_name != assumption_game_inst_name
    {
        let assumption_name = &assumption.name;
        return Err(Error::InvalidAssumptionMapping(format!("assumption game {assumption_game_inst_name:?} does not exist in assumption {assumption_name:?}")).with_span(span));
    };

    let mapping = Mapping::new(assumption_game_inst_name, game_inst_name, mappings);
    Ok(mapping)
}

fn handle_string_triplet(ast: &mut Pairs<Rule>) -> (String, String, String) {
    let strs: Vec<_> = ast.take(3).map(|str| str.as_str()).collect();

    (
        strs[0].to_string(),
        strs[1].to_string(),
        strs[2].to_string(),
    )
}

fn handle_string_pair(ast: &mut Pairs<Rule>) -> (String, String) {
    let strs: Vec<_> = ast.take(2).map(|str| str.as_str()).collect();
    (strs[0].to_string(), strs[1].to_string())
}

fn next_pairs<'a>(ast: &'a mut Pairs<Rule>) -> Pairs<'a, Rule> {
    ast.next().unwrap().into_inner()
}

fn next_pair<'a>(ast: &'a mut Pairs<Rule>) -> Pair<'a, Rule> {
    ast.next().unwrap()
}

fn next_pairs_with_span<'a>(ast: &'a mut Pairs<Rule>) -> (Pairs<'a, Rule>, Span<'a>) {
    let pair = ast.next().unwrap();
    (pair.clone().into_inner(), pair.as_span())
}

fn next_str<'a>(ast: &'a mut Pairs<Rule>) -> &'a str {
    ast.next().unwrap().as_str()
}
