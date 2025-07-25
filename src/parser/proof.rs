use std::collections::HashMap;

use crate::{
    expressions::Expression,
    gamehops::equivalence::Equivalence,
    gamehops::reduction::Assumption,
    gamehops::GameHop,
    identifier::{
        game_ident::GameConstIdentifier,
        proof_ident::{ProofConstIdentifier, ProofIdentifier::Const},
        Identifier,
    },
    package::{Composition, Package},
    parser::{
        error::{
            AssumptionMappingContainsDifferentPackagesError,
            AssumptionMappingDuplicatePackageInstanceError,
            AssumptionMappingLeftGameInstanceIsNotFromAssumption,
            ReductionContainsDifferentPackagesError,
        },
        Rule,
    },
    proof::{Claim, GameInstance, Proof},
    types::Type,
    util::scope::{Declaration, Error as ScopeError, Scope},
};

use miette::{Diagnostic, NamedSource};
use pest::{
    iterators::{Pair, Pairs},
    Span,
};
use thiserror::Error;

use super::{
    common::{self, HandleTypeError},
    error::{
        AssumptionExportsNotSufficientError, AssumptionMappingMissesPackageInstanceError,
        AssumptionMappingParameterMismatchError,
        AssumptionMappingRightGameInstanceIsFromAssumption, DuplicateGameParameterDefinitionError,
        InvalidGameInstanceInReductionError, MissingGameParameterDefinitionError,
        NoSuchGameParameterError, ReductionInconsistentAssumptionBoundaryError,
        ReductionPackageInstanceParameterMismatchError, UndefinedAssumptionError,
        UndefinedGameError, UndefinedGameInstanceError, UndefinedPackageInstanceError,
    },
    package::ParseExpressionError,
    ParseContext,
};

#[derive(Debug)]
pub(crate) struct ParseProofContext<'a> {
    pub file_name: &'a str,
    pub file_content: &'a str,
    pub scope: Scope,

    pub types: Vec<String>,

    pub proof_name: &'a str,

    pub consts: HashMap<String, Type>,
    pub instances: Vec<GameInstance>,
    pub instances_table: HashMap<String, (usize, GameInstance)>,
    pub assumptions: Vec<Assumption>,
    pub game_hops: Vec<GameHop<'a>>,
}

impl<'a> ParseContext<'a> {
    fn proof_context(self, proof_name: &'a str) -> ParseProofContext<'a> {
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

impl ParseProofContext<'_> {
    fn declare(&mut self, name: &str, clone: Declaration) -> Result<(), ScopeError> {
        self.scope.declare(name, clone)
    }
    // TODO: check dupes here?

    fn add_game_instance(&mut self, game_inst: GameInstance) {
        let offset = self.instances.len();
        self.instances.push(game_inst.clone());
        self.instances_table
            .insert(game_inst.name().to_string(), (offset, game_inst));
    }

    pub(crate) fn game_instance(&self, name: &str) -> Option<(usize, &GameInstance)> {
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
    HandleType(#[from] HandleTypeError),

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
    ReductionInconsistentAssumptionBoundary(#[from] ReductionInconsistentAssumptionBoundaryError),

    #[diagnostic(transparent)]
    #[error(transparent)]
    ReductionPackageInstanceParameterMismatch(
        #[from] ReductionPackageInstanceParameterMismatchError,
    ),

    #[diagnostic(transparent)]
    #[error(transparent)]
    AssumptionExportsNotSufficient(#[from] AssumptionExportsNotSufficientError),

    #[diagnostic(transparent)]
    #[error(transparent)]
    InvalidGameInstanceInReduction(#[from] InvalidGameInstanceInReductionError),
}

pub fn handle_proof<'a>(
    file_name: &'a str,
    file_content: &'a str,
    ast: Pair<'a, Rule>,
    pkgs: HashMap<String, Package>,
    games: HashMap<String, Composition>,
) -> Result<Proof<'a>, ParseProofError> {
    let mut iter = ast.into_inner();
    let proof_name = iter.next().unwrap().as_str();
    let proof_ast = iter.next().unwrap();

    let ctx = ParseContext::new(file_name, file_content);
    let mut ctx = ctx.proof_context(proof_name);
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
                handle_instance_decl(&mut ctx, ast, &games)?;
            }
            otherwise => unreachable!("found {:?} in proof", otherwise),
        }
    }

    let ParseProofContext {
        proof_name,
        consts,
        instances,
        assumptions,
        game_hops,
        ..
    } = ctx;

    Ok(Proof {
        name: proof_name.to_string(),
        consts: consts.into_iter().collect(),
        instances,
        assumptions,
        game_hops,
        pkgs: pkgs.into_values().collect(),
    })
}

fn handle_instance_decl<'a>(
    ctx: &mut ParseProofContext<'a>,
    ast: Pair<'a, Rule>,
    games: &HashMap<String, Composition>,
) -> Result<(), ParseProofError> {
    let mut ast = ast.into_inner();

    let game_inst_name = ast.next().unwrap().as_str().to_string();
    let game_name_ast = ast.next().unwrap();
    let game_name_span = game_name_ast.as_span();
    let game_name = game_name_ast.as_str();
    let body_ast = ast.next().unwrap();

    let game = games.get(game_name).ok_or(UndefinedGameError {
        source_code: ctx.named_source(),
        at: (game_name_span.start()..game_name_span.end()).into(),
        game_name: game_name.to_string(),
    })?;

    let (types, consts) = handle_instance_assign_list(ctx, &game_inst_name, game, body_ast)?;

    let consts_as_ident = consts
        .iter()
        .map(|(ident, expr)| (ident.clone(), expr.clone()))
        .collect();

    // println!("printing constant assignment in the parser:");
    // println!("  {consts_as_ident:#?}");

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
                            assigned_value: Some(Box::new(value.clone())),
                            inst_info: None,
                            game_inst_name: Some(game_inst_name.to_string()),
                            proof_name: Some(ctx.proof_name.to_string()),
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

        ctx.game_instance(&left_name)
            .ok_or(UndefinedGameInstanceError {
                source_code: ctx.named_source(),
                at: (left_name_span.start()..left_name_span.end()).into(),
                game_inst_name: left_name.clone(),
            })?;

        if ctx.game_instance(&right_name).is_none() {
            return Err(UndefinedGameInstanceError {
                source_code: ctx.named_source(),
                at: (right_name_span.start()..right_name_span.end()).into(),
                game_inst_name: right_name.clone(),
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

fn handle_game_hops<'a>(
    ctx: &mut ParseProofContext<'a>,
    ast: Pairs<'a, Rule>,
) -> Result<(), ParseProofError> {
    for hop_ast in ast {
        let game_hop = match hop_ast.as_rule() {
            Rule::equivalence => handle_equivalence(ctx, hop_ast)?,
            Rule::reduction => super::reduction::handle_reduction(ctx, hop_ast)?,
            otherwise => unreachable!("found {:?} in game_hops", otherwise),
        };
        ctx.game_hops.push(game_hop)
    }

    Ok(())
}

fn handle_equivalence<'a>(
    ctx: &mut ParseProofContext,
    ast: Pair<'a, Rule>,
) -> Result<GameHop<'a>, ParseProofError> {
    let mut ast = ast.into_inner();
    let (left_name, right_name) = handle_string_pair(&mut ast);

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

    if ctx.game_instance(left_name.as_str()).is_none() {
        return Err(UndefinedGameInstanceError {
            source_code: ctx.named_source(),
            at: (left_name.as_span().start()..left_name.as_span().end()).into(),
            game_inst_name: left_name.as_str().to_string(),
        }
        .into());
    }
    if ctx.game_instance(right_name.as_str()).is_none() {
        return Err(UndefinedGameInstanceError {
            source_code: ctx.named_source(),
            at: (right_name.as_span().start()..right_name.as_span().end()).into(),
            game_inst_name: right_name.as_str().to_string(),
        }
        .into());
    }

    let eq = Equivalence::new(
        left_name.as_str().to_string(),
        right_name.as_str().to_string(),
        invariants,
        trees,
    );

    Ok(GameHop::Equivalence(eq))
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

fn handle_string_triplet<'a>(
    ast: &mut Pairs<'a, Rule>,
) -> ((String, Span<'a>), (String, Span<'a>), (String, Span<'a>)) {
    let mut strs: Vec<_> = ast
        .take(3)
        .map(|str| (str.as_str().to_string(), str.as_span()))
        .collect();

    (strs.remove(0), strs.remove(0), strs.remove(0))
}

pub(crate) fn handle_identifiers<'a, T: crate::parser::ast::Identifier<'a>, const N: usize>(
    ast: &mut Pairs<'a, Rule>,
) -> [T; N] {
    ast.take(N)
        .map(T::from)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

fn handle_string_pair<'a>(ast: &mut Pairs<'a, Rule>) -> (Pair<'a, Rule>, Pair<'a, Rule>) {
    let [left, right] = ast.take(2).collect::<Vec<_>>().try_into().unwrap();

    (left, right)
}

fn next_pairs<'a>(ast: &'a mut Pairs<Rule>) -> Pairs<'a, Rule> {
    ast.next().unwrap().into_inner()
}

fn next_str<'a>(ast: &'a mut Pairs<Rule>) -> &'a str {
    ast.next().unwrap().as_str()
}
