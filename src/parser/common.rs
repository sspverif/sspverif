use crate::identifier::game_ident::{GameConstIdentifier, GameIdentifier, GameLoopVarIdentifier};
use crate::statement::FilePosition;
use crate::util::scope::Scope;
use crate::{expressions::Expression, identifier::Identifier, types::Type};

use super::error::{Error, OwnedSpan, Result, SpanError};
use super::{error, Rule};

use pest::iterators::Pair;

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
            ParseExpressionError::UndefinedIdentifer(name, file_pos, owned_span) => {
                println!("lost position at error conversion. The error occurred at {file_pos}");
                Error::UndefinedIdentifer(name)
            }
        }
    }
}

impl From<ParseExpressionError> for SpanError {
    fn from(value: ParseExpressionError) -> Self {
        println!("lost position at error conversion. The full error is {value}");
        match value {
            ParseExpressionError::UndefinedIdentifer(name, file_pos, owned_span) => {
                Error::UndefinedIdentifer(name).with_owned_span(owned_span)
            }
        }
    }
}

#[derive(Debug)]
pub enum ParseExpressionError {
    UndefinedIdentifer(String, FilePosition, OwnedSpan),
}

impl core::fmt::Display for ParseExpressionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseExpressionError::UndefinedIdentifer(name, file_pos, owned_span) => {
                write!(f, "undefined identifier `{name}` at {file_pos}")
            }
        }
    }
}

impl std::error::Error for ParseExpressionError {}
impl crate::error::LocationError for ParseExpressionError {
    fn file_pos<'a>(&'a self) -> &'a FilePosition {
        match self {
            ParseExpressionError::UndefinedIdentifer(_, file_pos, _) => file_pos,
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
            Expression::Typed(
                Type::Table(Box::new(idxtipe), Box::new(valtipe)),
                Box::new(Expression::EmptyTable),
            )
        }
        Rule::table_access => {
            let mut inner = expr.into_inner();
            let ident = inner.next().unwrap().as_str();
            let expr = handle_expression(inner.next().unwrap(), scope)?;
            Expression::TableAccess(Identifier::new_scalar(ident), Box::new(expr))
        }
        Rule::fn_call => {
            let mut inner = expr.into_inner();
            let ident = inner.next().unwrap().as_str();
            let args = match inner.next() {
                None => vec![],
                Some(inner_args) => inner_args
                    .into_inner()
                    .map(|e| handle_expression(e, scope))
                    .collect::<StdResult<_, _>>()?,
            };
            Expression::FnCall(Identifier::new_scalar(ident), args)
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
                crate::util::scope::Declaration::CompositionConst { tipe, game_name } => {
                    Identifier::GameIdentifier(GameIdentifier::Const(GameConstIdentifier {
                        game_name,
                        name,
                        tipe,
                    }))
                }
                crate::util::scope::Declaration::CompositionForSpec {
                    game_name,
                    start,
                    end,
                    start_comp,
                    end_comp,
                } => Identifier::GameIdentifier(GameIdentifier::LoopVar(GameLoopVarIdentifier {
                    game_name,
                    name,
                    start: Box::new(start),
                    end: Box::new(end),
                    start_comp,
                    end_comp,
                })),
                crate::util::scope::Declaration::PackageConst { .. }
                | crate::util::scope::Declaration::PackageState { .. }
                | crate::util::scope::Declaration::PackageOracleArg { .. }
                | crate::util::scope::Declaration::PackageOracleLocal { .. }
                | crate::util::scope::Declaration::PackageOracleImport { .. }
                | crate::util::scope::Declaration::PackageOracleImportsForSpec { .. } => {
                    unreachable!()
                }
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
        Rule::literal_emptyset => {
            let tipe = handle_type(expr.into_inner().next().unwrap());
            Expression::Typed(Type::Set(Box::new(tipe)), Box::new(Expression::Set(vec![])))
        }
        Rule::expr_tuple => Expression::Tuple(
            expr.into_inner()
                .map(|e| handle_expression(e, scope))
                .collect::<StdResult<_, _>>()?,
        ),
        Rule::expr_list => Expression::List(
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

pub fn handle_params_def_list(
    ast: Pair<Rule>,
    defined_consts: &[(String, Type)],
    scope: &mut Scope,
) -> Result<Vec<(String, Expression)>> {
    ast.into_inner()
        .map(|inner| {
            //let inner = inner.into_inner().next().unwrap();

            let mut inner = inner.into_inner();
            let left_ast = inner.next().unwrap();
            let left = left_ast.as_str();

            let right_ast = inner.next().unwrap();
            let right_span = right_ast.as_span();

            let right = handle_expression(right_ast, scope)?;

            match &right {
                // TODO: also allow proof constant identifiers, once we have them
                Expression::BooleanLiteral(_)
                | Expression::StringLiteral(_)
                | Expression::IntegerLiteral(_)
                | Expression::Identifier(Identifier::GameIdentifier(GameIdentifier::Const(_))) => {}
                Expression::Identifier(Identifier::Scalar(ident)) => {
                    if !defined_consts
                        .iter()
                        .any(|(defd_name, _)| ident == defd_name)
                    {
                        return Err(Error::UndefinedIdentifer(ident.clone()).with_span(right_span));
                    }
                }
                _ => {
                    return Err(Error::IllegalExpression(right.clone()).with_span(right_span));
                }
            }

            Ok((left.to_owned(), right.clone()))
        })
        .collect()
}

pub fn handle_types_def_list(
    ast: Pair<Rule>,
    inst_name: &str,
    file_name: &str,
) -> Result<Vec<(Type, Type)>> {
    ast.into_inner()
        .map(|def_spec| handle_types_def_spec(def_spec, inst_name, file_name))
        .collect()
}

pub fn handle_types_def_spec(
    ast: Pair<Rule>,
    inst_name: &str,
    file_name: &str,
) -> Result<(Type, Type)> {
    let span = ast.as_span();
    let file_pos = FilePosition::from_span(file_name, span);
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

    if let Err(err) = tf.transform_type(&snd_type, &file_pos) {
        return Err(error::Error::from(err).with_span(snd_span));
    }

    Ok((handle_type(fst), snd_type))
}

pub fn handle_const_decl(ast: Pair<Rule>) -> (String, Type) {
    let mut inner = ast.into_inner();
    let name = inner.next().unwrap().as_str().to_owned();
    let tipe = handle_type(inner.next().unwrap());

    (name, tipe)
}
