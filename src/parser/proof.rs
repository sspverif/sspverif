use std::collections::HashMap;

use crate::{
    expressions::Expression,
    identifier::{
        game_ident::GameConstIdentifier,
        proof_ident::{ProofConstIdentifier, ProofIdentifier::Const},
        Identifier,
    },
    package::{Composition, Package},
    parser::{error::AssumptionMappingLeftGameInstanceIsNotFromAssumption, Rule},
    proof::{Assumption, Claim, Equivalence, GameHop, GameInstance, Mapping, Proof, Reduction},
    types::Type,
    util::{
        resolver::{Resolver, SliceResolver},
        scope::{Declaration, Scope},
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
    error::{MissingGameParameterDefinitionError, NoSuchTypeError, UndefinedGameError},
    ParseContext,
};
use super::{
    error::{
        AssumptionMappingRightGameInstanceIsFromAssumption, DuplicateGameParameterDefinitionError,
        NoSuchGameParameterError, UndefinedAssumptionError, UndefinedGameInstanceError,
    },
    package::ParseExpressionError,
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
    fn declare(&mut self, name: &str, clone: Declaration) -> Result<(), ()> {
        self.scope.declare(name, clone)
    }
    // TODO: check dupes here?

    fn add_game_instance(&mut self, game_inst: GameInstance) {
        let offset = self.instances.len();
        self.instances.push(game_inst.clone());
        self.instances_table
            .insert(game_inst.name().to_string(), (offset, game_inst));
    }

    fn has_game_instance(&self, name: &str) -> bool {
        self.instances_table.contains_key(name)
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

    fn get_const(&self, name: &str) -> Option<&Type> {
        self.consts.get(name)
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

    let inst_name = ast.next().unwrap().as_str().to_string();
    let game_name_ast = ast.next().unwrap();
    let game_name_span = game_name_ast.as_span();
    let game_name = game_name_ast.as_str();
    let body_ast = ast.next().unwrap();

    let game = ctx.games.get(game_name).ok_or(UndefinedGameError {
        source_code: ctx.named_source(),
        at: (game_name_span.start()..game_name_span.end()).into(),
        game_name: game_name.to_string(),
    })?;

    let (types, consts) = handle_instance_assign_list(ctx, &inst_name, game, body_ast)?;

    let consts_as_ident = consts
        .iter()
        .map(|(ident, expr)| (ident.clone().into(), expr.clone()))
        .collect();

    let game_inst = GameInstance::new(inst_name, game.clone(), types, consts_as_ident);
    ctx.instances.push(game_inst);

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
    let mut ast = body.into_inner();
    let assumptions_ast = ast.next().unwrap();
    let assumptions_span = assumptions_ast.as_span();
    let mut assumptions_ast = assumptions_ast.into_inner();
    let assumption_name = next_str(&mut assumptions_ast);

    // check that assumption_name turns up in the assumptions list
    // and fetch the assumption definition
    let assumption_resolver = SliceResolver(&ctx.assumptions);
    let assumption = assumption_resolver
        .resolve_value(assumption_name)
        .ok_or(UndefinedAssumptionError {
            source_code: ctx.named_source(),
            at: (assumptions_span.start()..assumptions_span.end()).into(),
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

    let game1_is_left = mapping1.as_game_inst_name() == left_name.as_str();
    let (left, right) = if game1_is_left {
        (mapping1, mapping2)
    } else {
        (mapping2, mapping1)
    };

    let reduction = Reduction::new(left, right, assumption_name.to_string());

    Ok(reduction)
}

fn handle_mapspec<'a>(
    ctx: &mut ParseProofContext,
    ast: Pair<Rule>,
    assumption: &Assumption,
) -> Result<Mapping, ParseProofError> {
    let mut ast = ast.into_inner();

    let (
        (first_game_inst_name, first_game_inst_name_span),
        (second_game_inst_name, second_game_inst_name_span),
    ) = handle_string_pair(&mut ast);

    // check that game instance names can be resolved
    SliceResolver(&ctx.instances)
        .resolve_value(&first_game_inst_name)
        .ok_or(UndefinedGameInstanceError {
            source_code: ctx.named_source(),
            at: (first_game_inst_name_span.start()..first_game_inst_name_span.end()).into(),
            game_inst_name: first_game_inst_name.clone(),
        })?;
    SliceResolver(&ctx.instances)
        .resolve_value(&second_game_inst_name)
        .ok_or(UndefinedGameInstanceError {
            source_code: ctx.named_source(),
            at: (second_game_inst_name_span.start()..second_game_inst_name_span.end()).into(),
            game_inst_name: second_game_inst_name.clone(),
        })?;

    let is_left_assumption_game = first_game_inst_name.clone() == assumption.left_name;
    let is_right_assumption_game = first_game_inst_name == assumption.right_name;

    if !(is_left_assumption_game || is_right_assumption_game) {
        println!("{assumption:?}");
        return Err(AssumptionMappingLeftGameInstanceIsNotFromAssumption {
            source_code: ctx.named_source(),
            at: (first_game_inst_name_span.start()..first_game_inst_name_span.end()).into(),
            game_instance_name: first_game_inst_name.to_string(),
            assumption_left_game_instance_name: assumption.left_name.clone(),
            assumption_right_game_instance_name: assumption.right_name.clone(),
        }
        .into());
    }

    let is_left_assumption_game = second_game_inst_name == assumption.left_name;
    let is_right_assumption_game = second_game_inst_name == assumption.right_name;

    if !(is_left_assumption_game || is_right_assumption_game) {
        println!("{assumption:?}");
        return Err(AssumptionMappingRightGameInstanceIsFromAssumption {
            source_code: ctx.named_source(),
            at: (second_game_inst_name_span.start()..second_game_inst_name_span.end()).into(),
            game_instance_name: second_game_inst_name.to_string(),
            model_left_game_instance_name: first_game_inst_name.clone(),
            model_right_game_instance_name: second_game_inst_name.clone(),
        }
        .into());
    }

    let mappings: Vec<(String, String)> = ast
        .flat_map(Pair::into_inner)
        .map(|pair| pair.as_str())
        .map(str::to_string)
        .tuples()
        .collect();

    // TODO check mappings are valid
    for (_assumption_const, _game_const) in &mappings {}

    let mapping = Mapping::new(first_game_inst_name, second_game_inst_name, mappings);
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
