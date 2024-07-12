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
use std::result::Result as StdResult;

// TODO: identifier is optional
pub fn handle_arglist(arglist: Pair<Rule>) -> Vec<(String, Type)> {
    arglist
        .into_inner()
        .map(|arg| {
            let mut inner = arg.into_inner();
            let id = inner.next().unwrap().as_str();
            let tipe = handle_type(inner.next().unwrap());
            (id.to_string(), tipe)
        })
        .collect()
}

impl From<ParseExpressionError> for Error {
    fn from(value: ParseExpressionError) -> Self {
        match value {
            ParseExpressionError::UndefinedIdentifer(name, file_pos, _owned_span) => {
                println!("lost position at error conversion. The error occurred at {file_pos}");
                Error::UndefinedIdentifer(name)
            }
            ParseExpressionError::ParseIdentifier(parse_ident_err) => match parse_ident_err {
                ParseIdentifierError::ParseExpression(_) => todo!(),
                ParseIdentifierError::ScopeDeclareError(_) => todo!(),
                ParseIdentifierError::Undefined(name) => Error::UndefinedIdentifer(name),
                ParseIdentifierError::TypeMismatch(_) => todo!(),
                ParseIdentifierError::InvalidLeftHandSide(_, _) => todo!(),
            },
        }
    }
}

impl From<ParseExpressionError> for SpanError {
    fn from(value: ParseExpressionError) -> Self {
        println!("lost position at error conversion. The full error is {value}");
        match value {
            ParseExpressionError::UndefinedIdentifer(name, _file_pos, owned_span) => {
                Error::UndefinedIdentifer(name).with_owned_span(owned_span)
            }
            _ => todo!(),
        }
    }
}

#[derive(Debug)]
pub enum ParseExpressionError {
    UndefinedIdentifer(String, FilePosition, OwnedSpan),
    ParseIdentifier(ParseIdentifierError),
}

impl core::fmt::Display for ParseExpressionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseExpressionError::UndefinedIdentifer(name, file_pos, _owned_span) => {
                write!(f, "undefined identifier `{name}` at {file_pos}")
            }

            ParseExpressionError::ParseIdentifier(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for ParseExpressionError {}
impl crate::error::LocationError for ParseExpressionError {
    fn file_pos<'a>(&'a self) -> &'a FilePosition {
        match self {
            ParseExpressionError::UndefinedIdentifer(_, file_pos, _) => file_pos,
            _ => todo!(),
        }
    }
}

// TODO: These actually are not "common", but only for games (and maybe the proofs)
pub fn handle_expression(
    expr: Pair<Rule>,
    scope: &mut Scope,
) -> StdResult<Expression, ParseExpressionError> {
    let expr = match expr.as_rule() {
        // expr_equals | expr_not_equals | fn_call | table_access | identifier
        Rule::expr_add => {
            let mut inner = expr.into_inner();
            let lhs = handle_expression(inner.next().unwrap(), scope)?;
            let rhs = handle_expression(inner.next().unwrap(), scope)?;
            Expression::Add(Box::new(lhs), Box::new(rhs))
        }
        Rule::expr_sub => {
            let mut inner = expr.into_inner();
            let lhs = handle_expression(inner.next().unwrap(), scope)?;
            let rhs = handle_expression(inner.next().unwrap(), scope)?;
            Expression::Sub(Box::new(lhs), Box::new(rhs))
        }
        Rule::expr_mul => {
            let mut inner = expr.into_inner();
            let lhs = handle_expression(inner.next().unwrap(), scope)?;
            let rhs = handle_expression(inner.next().unwrap(), scope)?;
            Expression::Mul(Box::new(lhs), Box::new(rhs))
        }
        Rule::expr_div => {
            let mut inner = expr.into_inner();
            let lhs = handle_expression(inner.next().unwrap(), scope)?;
            let rhs = handle_expression(inner.next().unwrap(), scope)?;
            Expression::Div(Box::new(lhs), Box::new(rhs))
        }
        Rule::expr_and => Expression::And(
            expr.into_inner()
                .map(|e| handle_expression(e, scope))
                .collect::<StdResult<_, _>>()?,
        ),
        Rule::expr_or => Expression::Or(
            expr.into_inner()
                .map(|e| handle_expression(e, scope))
                .collect::<StdResult<_, _>>()?,
        ),
        Rule::expr_xor => Expression::Xor(
            expr.into_inner()
                .map(|e| handle_expression(e, scope))
                .collect::<StdResult<_, _>>()?,
        ),
        Rule::expr_not => {
            let mut inner = expr.into_inner();
            let content = handle_expression(inner.next().unwrap(), scope)?;
            Expression::Not(Box::new(content))
        }
        Rule::expr_equals => Expression::Equals(
            expr.into_inner()
                .map(|e| handle_expression(e, scope))
                .collect::<StdResult<_, _>>()?,
        ),
        Rule::expr_not_equals => Expression::Not(Box::new(Expression::Equals(
            expr.into_inner()
                .map(|e| handle_expression(e, scope))
                .collect::<StdResult<_, _>>()?,
        ))),
        Rule::expr_none => {
            let tipe = handle_type(expr.into_inner().next().unwrap());
            Expression::None(tipe)
        }
        Rule::expr_some => {
            let expr = handle_expression(expr.into_inner().next().unwrap(), scope)?;
            Expression::Some(Box::new(expr))
        }
        Rule::expr_unwrap => {
            let expr = handle_expression(expr.into_inner().next().unwrap(), scope)?;
            Expression::Unwrap(Box::new(expr))
        }
        Rule::expr_newtable => {
            let mut inner = expr.into_inner();
            let idxtipe = handle_type(inner.next().unwrap());
            let valtipe = handle_type(inner.next().unwrap());
            let tabletype = Type::Table(Box::new(idxtipe), Box::new(valtipe));
            Expression::EmptyTable(tabletype)
        }
        Rule::table_access => {
            let mut inner = expr.into_inner();
            let name = inner.next().unwrap().as_str();
            let ident = handle_identifier_in_code_rhs(name, scope)
                .map_err(ParseExpressionError::ParseIdentifier)?;
            let expr = handle_expression(inner.next().unwrap(), scope)?;
            Expression::TableAccess(ident, Box::new(expr))
        }
        Rule::fn_call => {
            let mut inner = expr.into_inner();
            let name = inner.next().unwrap().as_str();
            let ident = handle_identifier_in_code_rhs(name, scope)
                .map_err(ParseExpressionError::ParseIdentifier)?;
            let args = match inner.next() {
                None => vec![],
                Some(inner_args) => inner_args
                    .into_inner()
                    .map(|e| handle_expression(e, scope))
                    .collect::<StdResult<_, _>>()?,
            };
            Expression::FnCall(ident, args)
        }
        Rule::identifier => {
            let span = expr.as_span();
            let file_pos = FilePosition::from_span("???".to_string(), span.clone());
            let name = expr.as_str().to_string();
            let decl = scope
                .lookup(&name)
                .ok_or(ParseExpressionError::UndefinedIdentifer(
                    name.clone(),
                    file_pos,
                    OwnedSpan::new_with_span(span),
                ))?;

            let identifier = match decl {
                crate::util::scope::Declaration::Identifier(ident) => ident,
                crate::util::scope::Declaration::Oracle(_, _) => {
                    todo!("handle error, oracle is not an expression")
                }
            };

            Expression::Identifier(identifier)
        }
        Rule::literal_boolean => {
            let litval = expr.as_str().to_string();
            Expression::BooleanLiteral(litval)
        }
        Rule::literal_integer => {
            let litval = expr.as_str().trim().to_string();
            Expression::IntegerLiteral(i64::from_str_radix(&litval, 10).unwrap())
        }
        Rule::literal_emptyset => Expression::Set(vec![]),
        Rule::expr_list => Expression::List(
            expr.into_inner()
                .map(|e| handle_expression(e, scope))
                .collect::<StdResult<_, _>>()?,
        ),
        Rule::expr_tuple => Expression::Tuple(
            expr.into_inner()
                .map(|e| handle_expression(e, scope))
                .collect::<StdResult<_, _>>()?,
        ),
        Rule::expr_set => Expression::Set(
            expr.into_inner()
                .map(|e| handle_expression(e, scope))
                .collect::<StdResult<_, _>>()?,
        ),
        _ => unreachable!("Unhandled expression {:#?}", expr),
    };

    Ok(expr)
}

pub fn handle_type(tipe: Pair<Rule>) -> Type {
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

pub fn handle_game_params_def_list(
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

pub fn handle_proof_params_def_list(
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
