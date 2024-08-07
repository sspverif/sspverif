use std::collections::{HashMap, HashSet};

use crate::{
    expressions::Expression,
    identifier::{
        game_ident::{GameConstIdentifier, GameIdentifier},
        pkg_ident::PackageConstIdentifier,
        proof_ident::{
            ProofConstIdentifier,
            ProofIdentifier::{self, Const},
        },
        Identifier,
    },
    package::{Composition, Edge, Export, Package, PackageInstance},
    parser::{
        error::{
            AssumptionMappingContainsDifferentPackagesError,
            AssumptionMappingDuplicatePackageInstanceError,
            AssumptionMappingLeftGameInstanceIsNotFromAssumption,
            ReductionContainsDifferentPackagesError,
        },
        Rule,
    },
    proof::{Assumption, Claim, Equivalence, GameHop, GameInstance, Mapping, Proof, Reduction},
    types::Type,
    util::{
        resolver::{Resolver, SliceResolver},
        scope::{AlreadyDefinedError, Declaration, Scope},
    },
};

use itertools::Itertools;
use miette::{Diagnostic, NamedSource};
use pest::{
    iterators::{Pair, Pairs},
    Span,
};
use thiserror::Error;

use super::{
    common,
    error::{
        AssumptionExportsNotSufficientError, AssumptionMappingMissesPackageInstanceError,
        AssumptionMappingParameterMismatchError,
        AssumptionMappingRightGameInstanceIsFromAssumption, DuplicateGameParameterDefinitionError,
        MissingGameParameterDefinitionError, NoSuchGameParameterError, NoSuchTypeError,
        ReductionPackageInstanceParameterMismatchError, UndefinedAssumptionError,
        UndefinedGameError, UndefinedGameInstanceError, UndefinedPackageInstanceError,
    },
    package::ParseExpressionError,
    ParseContext,
};

#[derive(Debug)]
pub struct ParseProofContext<'a> {
    pub file_name: &'a str,
    pub file_content: &'a str,
    pub scope: Scope,

    pub types: Vec<String>,

    pub proof_name: &'a str,
    pub packages: &'a HashMap<String, Package>,
    pub games: &'a HashMap<String, Composition>,

    pub consts: HashMap<String, Type>,
    pub instances: Vec<GameInstance>,
    pub instances_table: HashMap<String, (usize, GameInstance)>,
    pub assumptions: Vec<Assumption>,
    pub game_hops: Vec<GameHop>,
}

impl<'a> ParseContext<'a> {
    fn proof_context(
        self,
        proof_name: &'a str,
        packages: &'a HashMap<String, Package>,
        games: &'a HashMap<String, Composition>,
    ) -> ParseProofContext<'a> {
        let Self {
            file_name,
            file_content,
            scope,
            types,
        } = self;

        ParseProofContext {
            file_name,
            file_content,
            proof_name,
            packages,
            games,

            scope,

            consts: HashMap::new(),
            types,

            instances: vec![],
            instances_table: HashMap::new(),
            assumptions: vec![],
            game_hops: vec![],
        }
    }
}

impl<'a> ParseProofContext<'a> {
    pub fn named_source(&self) -> NamedSource<String> {
        NamedSource::new(self.file_name, self.file_content.to_string())
    }

    pub fn parse_ctx(&self) -> ParseContext<'a> {
        ParseContext {
            file_name: self.file_name,
            file_content: self.file_content,
            scope: self.scope.clone(),
            types: self.types.clone(),
        }
    }
}

impl<'a> ParseProofContext<'a> {
    fn into_proof(self) -> Proof {
        let Self {
            proof_name,
            packages,
            consts,
            instances,
            assumptions,
            game_hops,
            ..
        } = self;

        Proof {
            name: proof_name.to_string(),
            consts: consts.into_iter().collect(),
            instances,
            assumptions,
            game_hops,
            pkgs: packages.values().cloned().collect(),
        }
    }
    fn declare(&mut self, name: &str, clone: Declaration) -> Result<(), AlreadyDefinedError> {
        self.scope.declare(name, clone)
    }
    // TODO: check dupes here?

    fn add_game_instance(&mut self, game_inst: GameInstance) {
        let offset = self.instances.len();
        self.instances.push(game_inst.clone());
        self.instances_table
            .insert(game_inst.name().to_string(), (offset, game_inst));
    }

    fn get_game_instance(&self, name: &str) -> Option<(usize, &GameInstance)> {
        self.instances_table
            .get(name)
            .map(|(offset, game_inst)| (*offset, game_inst))
    }

    // TODO: check dupes here?
    fn add_const(&mut self, name: String, ty: Type) {
        self.consts.insert(name, ty);
    }
}

#[derive(Debug, Error, Diagnostic)]
pub enum ParseProofError {
    #[diagnostic(transparent)]
    #[error(transparent)]
    ParseExpression(#[from] ParseExpressionError),

    #[diagnostic(transparent)]
    #[error(transparent)]
    UndefinedGame(#[from] UndefinedGameError),

    #[diagnostic(transparent)]
    #[error(transparent)]
    UndefinedPackageInstance(#[from] UndefinedPackageInstanceError),

    #[diagnostic(transparent)]
    #[error(transparent)]
    UndefinedGameInstance(#[from] UndefinedGameInstanceError),

    #[diagnostic(transparent)]
    #[error(transparent)]
    UndefinedAssumption(#[from] UndefinedAssumptionError),

    #[diagnostic(transparent)]
    #[error(transparent)]
    AssumptionMappingLeftGameInstanceIsNotFromAssumption(
        #[from] AssumptionMappingLeftGameInstanceIsNotFromAssumption,
    ),

    #[diagnostic(transparent)]
    #[error(transparent)]
    AssumptionMappingRightGameInstanceIsFromAssumption(
        #[from] AssumptionMappingRightGameInstanceIsFromAssumption,
    ),

    #[diagnostic(transparent)]
    #[error(transparent)]
    DuplicateGameParameterDefinition(#[from] DuplicateGameParameterDefinitionError),

    #[diagnostic(transparent)]
    #[error(transparent)]
    MissingGameParameterDefinition(#[from] MissingGameParameterDefinitionError),

    #[diagnostic(transparent)]
    #[error(transparent)]
    NoSuchGameParameter(#[from] NoSuchGameParameterError),

    #[diagnostic(transparent)]
    #[error(transparent)]
    NoSuchType(#[from] NoSuchTypeError),

    #[diagnostic(transparent)]
    #[error(transparent)]
    AssumptionMappingContainsDifferentPackages(
        #[from] AssumptionMappingContainsDifferentPackagesError,
    ),

    #[diagnostic(transparent)]
    #[error(transparent)]
    AssumptionMappingParameterMismatch(#[from] AssumptionMappingParameterMismatchError),

    #[diagnostic(transparent)]
    #[error(transparent)]
    AssumptionMappingDuplicatePackageInstance(
        #[from] AssumptionMappingDuplicatePackageInstanceError,
    ),

    #[diagnostic(transparent)]
    #[error(transparent)]
    AssumptionMappingMissesPackageInstance(#[from] AssumptionMappingMissesPackageInstanceError),

    #[diagnostic(transparent)]
    #[error(transparent)]
    ReductionContainsDifferentPackages(#[from] ReductionContainsDifferentPackagesError),

    #[diagnostic(transparent)]
    #[error(transparent)]
    ReductionPackageInstanceParameterMismatch(
        #[from] ReductionPackageInstanceParameterMismatchError,
    ),

    #[diagnostic(transparent)]
    #[error(transparent)]
    AssumptionExportsNotSufficient(#[from] AssumptionExportsNotSufficientError),
}

pub fn handle_proof(
    file_name: &str,
    file_content: &str,
    ast: Pair<Rule>,
    pkgs: &HashMap<String, Package>,
    games: &HashMap<String, Composition>,
) -> Result<Proof, ParseProofError> {
    let mut iter = ast.into_inner();
    let proof_name = iter.next().unwrap().as_str();
    let proof_ast = iter.next().unwrap();

    let ctx = ParseContext::new(file_name, file_content);
    let mut ctx = ctx.proof_context(proof_name, pkgs, games);
    ctx.scope.enter();

    for ast in proof_ast.into_inner() {
        match ast.as_rule() {
            Rule::const_decl => {
                let (const_name, tipe) = common::handle_const_decl(&ctx.parse_ctx(), ast)?;
                let clone = Declaration::Identifier(Identifier::ProofIdentifier(Const(
                    ProofConstIdentifier {
                        proof_name: proof_name.to_string(),
                        name: const_name.clone(),
                        tipe: tipe.clone(),
                        inst_info: None,
                    },
                )));
                ctx.declare(&const_name, clone).unwrap();
                ctx.add_const(const_name, tipe);
            }
            Rule::assumptions => {
                handle_assumptions(&mut ctx, ast.into_inner())?;
            }
            Rule::game_hops => {
                handle_game_hops(&mut ctx, ast.into_inner())?;
            }
            Rule::instance_decl => {
                handle_instance_decl(&mut ctx, ast)?;
            }
            otherwise => unreachable!("found {:?} in proof", otherwise),
        }
    }

    Ok(ctx.into_proof())
}

fn handle_instance_decl(
    ctx: &mut ParseProofContext,
    ast: Pair<Rule>,
) -> Result<(), ParseProofError> {
    let mut ast = ast.into_inner();

    let game_inst_name = ast.next().unwrap().as_str().to_string();
    let game_name_ast = ast.next().unwrap();
    let game_name_span = game_name_ast.as_span();
    let game_name = game_name_ast.as_str();
    let body_ast = ast.next().unwrap();

    let game = ctx.games.get(game_name).ok_or(UndefinedGameError {
        source_code: ctx.named_source(),
        at: (game_name_span.start()..game_name_span.end()).into(),
        game_name: game_name.to_string(),
    })?;

    let (types, consts) = handle_instance_assign_list(ctx, &game_inst_name, game, body_ast)?;

    let consts_as_ident = consts
        .iter()
        .map(|(ident, expr)| (ident.clone(), expr.clone()))
        .collect();

    let game_inst = GameInstance::new(
        game_inst_name,
        ctx.proof_name.to_string(),
        game.clone(),
        types,
        consts_as_ident,
    );
    ctx.add_game_instance(game_inst);

    Ok(())
}

fn handle_instance_assign_list(
    ctx: &ParseProofContext,
    game_inst_name: &str,
    game: &Composition,
    ast: Pair<Rule>,
) -> Result<(Vec<(String, Type)>, Vec<(GameConstIdentifier, Expression)>), ParseProofError> {
    let ast = ast.into_inner();

    let types = vec![];
    let mut consts = vec![];

    for ast in ast {
        match ast.as_rule() {
            Rule::types_def => {
                //let ast = ast.into_inner().next().unwrap();
                //types.extend(common::handle_types_def_list(ast, inst_name, file_name)?);
            }
            Rule::params_def => {
                let ast = ast.into_inner().next().unwrap();
                let defs = common::handle_proof_params_def_list(ctx, game, game_inst_name, ast)?;

                consts.extend(defs.into_iter().map(|(name, value)| {
                    (
                        GameConstIdentifier {
                            game_name: game.name.to_string(),
                            name,
                            tipe: value.get_type(),
                            inst_info: None,
                        },
                        value,
                    )
                }));
            }
            otherwise => {
                unreachable!("unexpected {:?} at {:?}", otherwise, ast.as_span())
            }
        }
    }

    Ok((types, consts))
}

fn handle_assumptions(
    ctx: &mut ParseProofContext,
    ast: Pairs<Rule>,
) -> Result<(), ParseProofError> {
    for pair in ast {
        let ((name, _), (left_name, left_name_span), (right_name, right_name_span)) =
            handle_string_triplet(&mut pair.into_inner());

        let inst_resolver = SliceResolver(&ctx.instances);

        if inst_resolver.resolve_value(&left_name).is_none() {
            return Err(UndefinedGameInstanceError {
                source_code: ctx.named_source(),
                at: (left_name_span.start()..left_name_span.end()).into(),
                game_inst_name: left_name,
            }
            .into());
        }

        if inst_resolver.resolve_value(&right_name).is_none() {
            return Err(UndefinedGameInstanceError {
                source_code: ctx.named_source(),
                at: (right_name_span.start()..right_name_span.end()).into(),
                game_inst_name: right_name,
            }
            .into());
        }

        ctx.assumptions.push(Assumption {
            name,
            left_name,
            right_name,
        })
    }

    Ok(())
}

fn handle_game_hops(ctx: &mut ParseProofContext, ast: Pairs<Rule>) -> Result<(), ParseProofError> {
    for hop_ast in ast {
        let game_hop = match hop_ast.as_rule() {
            Rule::equivalence => handle_equivalence(ctx, hop_ast)?,
            Rule::reduction => handle_reduction(ctx, hop_ast)?,
            otherwise => unreachable!("found {:?} in game_hops", otherwise),
        };
        ctx.game_hops.extend(game_hop)
    }

    Ok(())
}

fn handle_equivalence(
    ctx: &mut ParseProofContext,
    ast: Pair<Rule>,
) -> Result<Vec<GameHop>, ParseProofError> {
    let mut ast = ast.into_inner();
    let ((left_name, left_name_span), (right_name, right_name_span)) = handle_string_pair(&mut ast);

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

    if SliceResolver(&ctx.instances)
        .resolve_value(&left_name)
        .is_none()
    {
        return Err(UndefinedGameInstanceError {
            source_code: ctx.named_source(),
            at: (left_name_span.start()..left_name_span.end()).into(),
            game_inst_name: left_name,
        }
        .into());
    }
    if SliceResolver(&ctx.instances)
        .resolve_value(&right_name)
        .is_none()
    {
        return Err(UndefinedGameInstanceError {
            source_code: ctx.named_source(),
            at: (right_name_span.start()..right_name_span.end()).into(),
            game_inst_name: right_name,
        }
        .into());
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
    ctx: &mut ParseProofContext,
    ast: Pair<Rule>,
) -> Result<Vec<GameHop>, ParseProofError> {
    let mut ast = ast.into_inner();

    let left_name_ast = ast.next().unwrap();
    let right_name_ast = ast.next().unwrap();
    let body_ast = ast.next().unwrap();

    let reduction = handle_reduction_body(ctx, left_name_ast, right_name_ast, body_ast)?;

    Ok(vec![GameHop::Reduction(reduction)])
}

fn handle_reduction_body(
    ctx: &mut ParseProofContext,
    left_name: Pair<Rule>,
    _right_name: Pair<Rule>,
    body: Pair<Rule>,
) -> Result<Reduction, ParseProofError> {
    let reduction_span = body.as_span();
    let mut ast = body.into_inner();
    let assumptions_spec_ast = ast.next().unwrap();
    let assumptions_name_ast = assumptions_spec_ast.into_inner().next().unwrap();
    let assumptions_name_span = assumptions_name_ast.as_span();
    let assumption_name = assumptions_name_ast.as_str();

    // check that assumption_name turns up in the assumptions list
    // and fetch the assumption definition
    let assumption_resolver = SliceResolver(&ctx.assumptions);
    let assumption = assumption_resolver
        .resolve_value(assumption_name)
        .ok_or(UndefinedAssumptionError {
            source_code: ctx.named_source(),
            at: (assumptions_name_span.start()..assumptions_name_span.end()).into(),
            assumption_name: assumption_name.to_string(),
        })?
        .clone();

    let map1_ast = ast.next().unwrap();
    let map2_ast = ast.next().unwrap();

    let mapping1 = handle_mapspec(ctx, map1_ast, &assumption)?;
    let mapping2 = handle_mapspec(ctx, map2_ast, &assumption)?;

    if mapping1.as_game_inst_name() == mapping2.as_game_inst_name() {
        panic!();
        // TODO reutrn err
    }

    if mapping1.as_assumption_game_inst_name() == mapping2.as_assumption_game_inst_name() {
        panic!();
        // TODO reutrn err
    }

    let (_, left_game_inst) = ctx.get_game_instance(mapping1.as_game_inst_name()).unwrap();
    let (_, right_game_inst) = ctx.get_game_instance(mapping2.as_game_inst_name()).unwrap();

    let mut unmapped_pkg_insts1: Vec<_> = {
        let mapped_bigger_pkg_insts = mapping1
            .pkg_maps()
            .iter()
            .map(|(_, bigger_pkg_inst_name)| bigger_pkg_inst_name)
            .collect::<HashSet<_>>();
        left_game_inst
            .game()
            .pkgs
            .iter()
            .filter(|pkg_inst| !mapped_bigger_pkg_insts.contains(&pkg_inst.name))
            .collect()
    };

    unmapped_pkg_insts1.sort_by(|left, right| left.name.cmp(&right.name));

    let mut unmapped_pkg_insts2: Vec<_> = {
        let mapped_bigger_pkg_insts = mapping2
            .pkg_maps()
            .iter()
            .map(|(_, bigger_pkg_inst_name)| bigger_pkg_inst_name)
            .collect::<HashSet<_>>();
        right_game_inst
            .game()
            .pkgs
            .iter()
            .filter(|pkg_inst| !mapped_bigger_pkg_insts.contains(&pkg_inst.name))
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

    let game1_is_left = mapping1.as_game_inst_name() == left_name.as_str();
    let (left, right) = if game1_is_left {
        (mapping1, mapping2)
    } else {
        (mapping2, mapping1)
    };

    let reduction = Reduction::new(left, right, assumption_name.to_string());

    Ok(reduction)
}

fn handle_mapspec(
    ctx: &mut ParseProofContext,
    ast: Pair<Rule>,
    assumption: &Assumption,
) -> Result<Mapping, ParseProofError> {
    let mapping_span = ast.as_span();
    let mut ast = ast.into_inner();

    let (
        (assumption_game_inst_name, assumption_game_inst_name_span),
        (construction_game_inst_name, construction_game_inst_name_span),
    ) = handle_string_pair(&mut ast);

    // check that game instance names can be resolved
    let assumption_game_inst = SliceResolver(&ctx.instances)
        .resolve_value(&assumption_game_inst_name)
        .ok_or(UndefinedGameInstanceError {
            source_code: ctx.named_source(),
            at: (assumption_game_inst_name_span.start()..assumption_game_inst_name_span.end())
                .into(),
            game_inst_name: assumption_game_inst_name.clone(),
        })?;
    let construction_game_inst = SliceResolver(&ctx.instances)
        .resolve_value(&construction_game_inst_name)
        .ok_or(UndefinedGameInstanceError {
            source_code: ctx.named_source(),
            at: (construction_game_inst_name_span.start()..construction_game_inst_name_span.end())
                .into(),
            game_inst_name: construction_game_inst_name.clone(),
        })?;

    let assumption_game_is_really_assumption_game = assumption_game_inst_name
        == assumption.left_name
        || assumption_game_inst_name == assumption.right_name;

    let construction_game_is_actually_assumption_game = construction_game_inst_name
        == assumption.left_name
        || construction_game_inst_name == assumption.right_name;

    if !assumption_game_is_really_assumption_game {
        println!("{assumption:?}");
        return Err(AssumptionMappingLeftGameInstanceIsNotFromAssumption {
            source_code: ctx.named_source(),
            at: (assumption_game_inst_name_span.start()..assumption_game_inst_name_span.end())
                .into(),
            game_instance_name: assumption_game_inst_name.to_string(),
            assumption_left_game_instance_name: assumption.left_name.clone(),
            assumption_right_game_instance_name: assumption.right_name.clone(),
        }
        .into());
    }

    if construction_game_is_actually_assumption_game {
        println!("{assumption:?}");
        return Err(AssumptionMappingRightGameInstanceIsFromAssumption {
            source_code: ctx.named_source(),
            at: (construction_game_inst_name_span.start()..construction_game_inst_name_span.end())
                .into(),
            game_instance_name: construction_game_inst_name.to_string(),
            model_left_game_instance_name: assumption_game_inst_name.clone(),
            model_right_game_instance_name: construction_game_inst_name.clone(),
        }
        .into());
    }

    let mappings: Vec<_> = ast.flat_map(Pair::into_inner).tuples().collect();

    let mut assumption_game_pkg_inst_names: HashMap<String, &Pair<'_, Rule>> = HashMap::new();
    let mut construction_game_pkg_inst_names: HashMap<String, &Pair<'_, Rule>> = HashMap::new();

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
            return Err(UndefinedPackageInstanceError {
                source_code: ctx.named_source(),
                at,
                pkg_inst_name: assumption_game_pkg_inst_name.to_string(),
                in_game: construction_game_inst.game().name.clone(),
            }
            .into());
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
            assumption_game_pkg_inst_offset,
            construction_game_pkg_inst_offset,
        ));
    }

    // read all mappings. now check we mapped all package instances of the assumption.

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

    // check that packages that are mapped only call packages that are also mapped

    let bigger_game = construction_game_inst.game();
    for Edge(from, to, _sig) in &bigger_game.edges {
        let from_name = &bigger_game.pkgs[*from].name;
        let to_name = &bigger_game.pkgs[*to].name;
        if construction_game_pkg_inst_names.contains_key(from_name)
            && !construction_game_pkg_inst_names.contains_key(to_name)
        {
            todo!("write error message that mapping isn't a clean cut and mapped packages call unmapped packages")
        }
    }

    // check that if an oracle can be called by adversary or reduction in the construction game,
    // then it must be exported in the assumption game.
    // specifically, check that
    // for all mapped construction packa or goes to the adversaryge instances mapped_pkg_inst
    //   for every incoming export into mapped_pkg_inst
    //     require that the oracle at the edge is exported.
    //   for every incoming edge e into mapped_pkg_inst
    //     if e.src is not maapped
    //       require that the oracle at the edge is exported.
    //
    // said differently,
    // for all edges `(from, to, osig)` in the construction game
    //   if `to` is mapped to `assumption_to` and `from` isn't
    //     require that in the assumption game, there is an export `(assumption_to, osig)`
    // for all exports `(to, osig)` in the construction game
    //   if `to` is mapped to `assumption_to`
    //     require that in the assumption game, there is an export `(assumption_to, osig)`
    //
    for Edge(src, dst, osig) in &construction_game_inst.game().edges {
        let src_is_mapped = pkg_offset_mapping
            .iter()
            .any(|(_, constr_src)| constr_src == src);
        let dst_mapping =
            pkg_offset_mapping
                .iter()
                .find_map(|(assumption_dst, construction_dst)| {
                    if dst == construction_dst {
                        Some(*assumption_dst)
                    } else {
                        None
                    }
                });

        if let Some(assumption_dst) = dst_mapping {
            let export_exists =
                assumption_game_inst
                    .game()
                    .exports
                    .iter()
                    .any(|Export(found_dst, found_osig)| {
                        *found_dst == assumption_dst && found_osig == osig
                    });

            if !src_is_mapped && !export_exists {
                let assumption_dst_name = &assumption_game_inst.game().pkgs[assumption_dst].name;
                let (assumption_ast, construction_ast) = mappings
                    .iter()
                    .find(|pair| pair.0.as_str() == assumption_dst_name)
                    .unwrap();

                let assumption_span = assumption_ast.as_span();
                let assumption_at = (assumption_span.start()..assumption_span.end()).into();

                let construction_span = construction_ast.as_span();
                let construction_at = (construction_span.start()..construction_span.end()).into();

                let assumption_pkg_inst_name = assumption_game_inst.game().pkgs[assumption_dst]
                    .name
                    .clone();
                let construction_pkg_inst_name =
                    construction_game_inst.game().pkgs[*dst].name.clone();
                let oracle_name = osig.name.clone();

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
        }
    }

    for Export(dst, osig) in &construction_game_inst.game().exports {
        let dst_mapping =
            pkg_offset_mapping
                .iter()
                .find_map(|(assumption_dst, construction_dst)| {
                    if dst == construction_dst {
                        Some(*assumption_dst)
                    } else {
                        None
                    }
                });

        if let Some(assumption_dst) = dst_mapping {
            let export_exists =
                assumption_game_inst
                    .game()
                    .exports
                    .iter()
                    .any(|Export(found_dst, found_osig)| {
                        *found_dst == assumption_dst && found_osig == osig
                    });

            if !export_exists {
                let assumption_dst_name = &assumption_game_inst.game().pkgs[assumption_dst].name;
                let (assumption_ast, construction_ast) = mappings
                    .iter()
                    .find(|pair| pair.0.as_str() == assumption_dst_name)
                    .unwrap();

                let assumption_span = assumption_ast.as_span();
                let assumption_at = (assumption_span.start()..assumption_span.end()).into();

                let construction_span = construction_ast.as_span();
                let construction_at = (construction_span.start()..construction_span.end()).into();

                let assumption_pkg_inst_name = assumption_game_inst.game().pkgs[assumption_dst]
                    .name
                    .clone();
                let construction_pkg_inst_name =
                    construction_game_inst.game().pkgs[*dst].name.clone();
                let oracle_name = osig.name.clone();

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
        }
    }

    let mapping = Mapping::new(
        assumption_game_inst_name,
        construction_game_inst_name,
        mappings
            .into_iter()
            .map(|(ast1, ast2)| (ast1.as_str().to_string(), ast2.as_str().to_string()))
            .collect_vec(),
    );
    Ok(mapping)
}

fn handle_string_triplet<'a>(
    ast: &mut Pairs<'a, Rule>,
) -> ((String, Span<'a>), (String, Span<'a>), (String, Span<'a>)) {
    let mut strs: Vec<_> = ast
        .take(3)
        .map(|str| (str.as_str().to_string(), str.as_span()))
        .collect();

    (strs.remove(0), strs.remove(0), strs.remove(0))
}

fn handle_string_pair<'a>(ast: &mut Pairs<'a, Rule>) -> ((String, Span<'a>), (String, Span<'a>)) {
    let mut strs: Vec<_> = ast
        .take(2)
        .map(|str| (str.as_str().to_string(), str.as_span()))
        .collect();
    (strs.remove(0), strs.remove(0))
}

fn next_pairs<'a>(ast: &'a mut Pairs<Rule>) -> Pairs<'a, Rule> {
    ast.next().unwrap().into_inner()
}

fn next_str<'a>(ast: &'a mut Pairs<Rule>) -> &'a str {
    ast.next().unwrap().as_str()
}

fn package_instances_diff(
    _ctx: &ParseProofContext,
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
        // problem: they have identifiers in them that contain things like game names
        // we see two possible solutions:
        //
        // - recurse over both expressions at the same time and compare every subexpression.
        //   when we hit identifiers, compare everything except the problematic bits.
        // - use Expression::map to find identifiers are redact some information in there, so
        //   we can just compare the expressions
        //
        // the latter seems easier, so we'll go with that.

        let redact_ident = |ident: Identifier| -> Identifier {
            match ident {
                Identifier::GameIdentifier(GameIdentifier::LoopVar(mut game_loopvar)) => {
                    game_loopvar.game_inst_name = None;
                    game_loopvar.proof_name = None;
                    game_loopvar.inst_info = None;

                    game_loopvar.into()
                }
                Identifier::ProofIdentifier(ProofIdentifier::Const(mut proof_const)) => {
                    proof_const.inst_info = None;

                    proof_const.into()
                }
                Identifier::ProofIdentifier(ProofIdentifier::LoopVar(mut proof_loopvar)) => {
                    proof_loopvar.inst_info = None;

                    proof_loopvar.into()
                }
                Identifier::GameIdentifier(GameIdentifier::Const(_)) => unreachable!(),
                Identifier::PackageIdentifier(_) => unreachable!(),
                Identifier::Generated(_, _) => unreachable!(),
            }
        };

        let redact_expr = |expr: Expression| -> Expression {
            match expr {
                Expression::Identifier(ident) => Expression::Identifier(redact_ident(ident)),
                other => other,
            }
        };

        let redacted_left_param_expr = left_param_expr.map(redact_expr);
        let redacted_right_param_expr = right_param_expr.map(redact_expr);

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
