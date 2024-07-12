use crate::identifier::game_ident::GameIdentifier;
use crate::identifier::proof_ident::ProofIdentifier;
use crate::package::{Composition, Package};
use crate::statement::FilePosition;
use crate::util::scope::Scope;
use crate::{expressions::Expression, identifier::Identifier, types::Type};

use super::composition::{ParseGameContext, ParseGameError};
use super::error::{
    DuplicateGameParameterDefinitionError, DuplicatePackageParameterDefinitionError, Error,
    MissingGameParameterDefinitionError, MissingPackageParameterDefinitionError,
    NoSuchGameParameterError, NoSuchPackageParameterError, OwnedSpan, SpanError,
};
use super::package::{handle_identifier_in_code_rhs, ParseIdentifierError};
use super::proof::{ParseProofContext, ParseProofError};
use super::Rule;

use itertools::Itertools;
use miette::SourceSpan;
use pest::iterators::Pair;

use std::collections::{HashMap, HashSet};

pub(crate) fn handle_type(tipe: Pair<Rule>) -> Type {
    match tipe.as_rule() {
        Rule::type_empty => Type::Empty,
        Rule::type_integer => Type::Integer,
        Rule::type_bool => Type::Boolean,
        Rule::type_string => Type::String,
        Rule::type_maybe => Type::Maybe(Box::new(handle_type(tipe.into_inner().next().unwrap()))),
        Rule::type_bits => Type::Bits(tipe.into_inner().next().unwrap().as_str().to_string()),
        Rule::type_tuple => Type::Tuple(tipe.into_inner().map(handle_type).collect()),
        Rule::type_table => {
            let mut inner = tipe.into_inner();
            let indextype = handle_type(inner.next().unwrap());
            let valuetype = handle_type(inner.next().unwrap());
            Type::Table(Box::new(indextype), Box::new(valuetype))
        }
        Rule::type_fn => {
            let mut inner = tipe.into_inner();
            let argtipes = inner
                .next()
                .unwrap()
                .into_inner()
                .map(|spec| handle_type(spec.into_inner().next().unwrap()))
                .collect();
            let tipe = handle_type(inner.next().unwrap());
            Type::Fn(argtipes, Box::new(tipe))
        }
        Rule::type_userdefined => Type::UserDefined(tipe.as_str().to_string()),
        _ => {
            unreachable!("{:#?}", tipe)
        }
    }
}

pub(crate) fn handle_game_params_def_list(
    ctx: &ParseGameContext,
    pkg: &Package,
    pkg_inst_name: &str,
    ast: Pair<Rule>,
) -> std::result::Result<Vec<(String, Expression)>, super::composition::ParseGameError> {
    let params = &pkg.params;
    let mut defined_params = HashMap::<String, SourceSpan>::new();
    let block_span = ast.as_span();
    let result = ast
        .into_inner()
        // loop over the parameter definitions
        .map(|inner| {
            let pair_span = inner.as_span();
            let mut inner = inner.into_inner();
            let name_ast = inner.next().unwrap();
            let value_ast = inner.next().unwrap();

            let name_span = name_ast.as_span();
            let name = name_ast.as_str();

            // check that the defined parameter hasn't been defined before
            // (insert returns Some(x) if x has been written before)
            if let Some(prev_span) = defined_params.insert(
                name.to_string(),
                (pair_span.start()..pair_span.end()).into(),
            ) {
                Err(DuplicatePackageParameterDefinitionError {
                    source_code: ctx.named_source(),
                    at: (name_span.start()..name_span.end()).into(),
                    other: prev_span,
                    param_name: name.to_string(),
                    pkg_inst_name: pkg_inst_name.to_string(),
                })?;
            }

            // look up the parameter declaration from the package
            let maybe_param_info = params.iter().find(|(name_, _, _)| name == name_);

            // if it desn't exist, return an error
            let (_, expected_type, _) = maybe_param_info.ok_or(NoSuchPackageParameterError {
                source_code: ctx.named_source(),
                at: (name_span.start()..name_span.end()).into(),
                param_name: name.to_string(),
                pkg_name: pkg.name.clone(),
            })?;

            // parse the assigned value, and set the expected type to what the declaration
            // prescribes.
            let value = super::package::handle_expression(
                &ctx.parse_ctx(),
                value_ast,
                Some(expected_type),
            )?;

            Ok((name.to_string(), value))
        })
        .collect::<std::result::Result<Vec<_>, super::composition::ParseGameError>>()
        .and_then(|list| {
            let definied_names: HashSet<String> = defined_params.keys().cloned().collect();
            let declared_names: HashSet<String> =
                params.iter().map(|(name, _, _)| name).cloned().collect();

            let missing_params_vec: Vec<_> = declared_names
                .difference(&definied_names)
                .cloned()
                .collect();
            let missing_params = missing_params_vec.iter().join(", ");

            if !missing_params_vec.is_empty() {
                Err(MissingPackageParameterDefinitionError {
                    source_code: ctx.named_source(),
                    at: (block_span.start()..block_span.end()).into(),
                    pkg_name: pkg.name.clone(),
                    pkg_inst_name: pkg_inst_name.to_string(),
                    missing_params_vec,
                    missing_params,
                }
                .into())
            } else {
                Ok(list)
            }
        });

    result
}

pub(crate) fn handle_proof_params_def_list(
    ctx: &ParseProofContext,
    game: &Composition,
    game_inst_name: &str,
    ast: Pair<Rule>,
) -> Result<Vec<(String, Expression)>, ParseProofError> {
    let params = &game.consts;
    let mut defined_params = HashMap::<String, SourceSpan>::new();
    let block_span = ast.as_span();
    ast.into_inner()
        .map(|inner| {
            let pair_span = inner.as_span();
            let mut inner = inner.into_inner();
            let name_ast = inner.next().unwrap();
            let value_ast = inner.next().unwrap();

            let name_span = name_ast.as_span();
            let name = name_ast.as_str();

            // check that the defined parameter hasn't been defined before
            // (insert returns Some(x) if x has been written before)
            if let Some(prev_span) = defined_params.insert(
                name.to_string(),
                (pair_span.start()..pair_span.end()).into(),
            ) {
                return Err(DuplicateGameParameterDefinitionError {
                    source_code: ctx.named_source(),
                    at: (name_span.start()..name_span.end()).into(),
                    other: prev_span,
                    param_name: name.to_string(),
                    game_inst_name: game_inst_name.to_string(),
                }
                .into());
            }

            // look up the parameter declaration from the game
            let maybe_param_info = params.iter().find(|(name_, _)| name == name_);

            // if it desn't exist, return an error
            let (_, expected_type) = maybe_param_info.ok_or(NoSuchGameParameterError {
                source_code: ctx.named_source(),
                at: (name_span.start()..name_span.end()).into(),
                param_name: name.to_string(),
                game_name: game.name.clone(),
            })?;

            let value = super::package::handle_expression(
                &ctx.parse_ctx(),
                value_ast,
                Some(expected_type),
            )?;

            Ok((name.to_owned(), value.clone()))
        })
        .collect::<std::result::Result<Vec<_>, ParseProofError>>()
        .and_then(|list| {
            let definied_names: HashSet<String> = defined_params.keys().cloned().collect();
            let declared_names: HashSet<String> =
                params.iter().map(|(name, _)| name).cloned().collect();

            let missing_params_vec: Vec<_> = declared_names
                .difference(&definied_names)
                .cloned()
                .collect();
            let missing_params = missing_params_vec.iter().join(", ");

            if !missing_params_vec.is_empty() {
                Err(MissingGameParameterDefinitionError {
                    source_code: ctx.named_source(),
                    at: (block_span.start()..block_span.end()).into(),
                    game_name: game.name.clone(),
                    game_inst_name: game_inst_name.to_string(),
                    missing_params_vec,
                    missing_params,
                }
                .into())
            } else {
                Ok(list)
            }
        })
}

/*
pub fn handle_types_def_list(
    ast: Pair<Rule>,
    inst_name: &str,
    file_name: &str,
) -> std::result::Result<Vec<(String, Type)>, ParseGameError> {
    ast.into_inner()
        .map(|def_spec| handle_types_def_spec(def_spec, inst_name, file_name))
        .collect()
}

pub fn handle_types_def_spec(
    ast: Pair<Rule>,
    inst_name: &str,
    _file_name: &str,
) -> Result<(String, Type)> {
    let span = ast.as_span();
    let mut iter = ast.into_inner();

    let fst = iter.next().unwrap();

    let snd = iter.next().unwrap();

    let snd_span = snd.as_span();
    let snd_type = handle_type(snd);

        let place = crate::transforms::resolvetypes::Place::Types {
            inst_name: inst_name.to_string(),
            type_name: format!("{:?}", fst.as_str()),
        };

        let tf = crate::transforms::resolvetypes::ResolveTypesTypeTransform::new(place);

        if let Err(err) = tf.transform_type(&snd_type, &(span.start()..span.end()).into()) {
            return Err(error::Error::from(err).with_span(snd_span));
        }

    Ok((fst.as_str().to_string(), snd_type))
}
    */

pub fn handle_const_decl(ast: Pair<Rule>) -> (String, Type) {
    let mut inner = ast.into_inner();
    let name = inner.next().unwrap().as_str().to_owned();
    let tipe = handle_type(inner.next().unwrap());

    (name, tipe)
}
