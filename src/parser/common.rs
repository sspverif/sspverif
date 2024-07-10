use crate::identifier::game_ident::GameIdentifier;
use crate::identifier::proof_ident::ProofIdentifier;
use crate::statement::FilePosition;
use crate::util::scope::Scope;
use crate::{expressions::Expression, identifier::Identifier, types::Type};

use super::error::{Error, OwnedSpan, Result, SpanError};
use super::package::{handle_identifier_in_code_rhs, ParseIdentifierError};
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
    ast: Pair<Rule>,
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
                | Expression::Identifier(Identifier::GameIdentifier(GameIdentifier::LoopVar(_)))
                | Expression::Identifier(Identifier::GameIdentifier(GameIdentifier::Const(_))) => {}

                _ => {
                    return Err(Error::IllegalExpression(right.clone()).with_span(right_span));
                }
            }

            Ok((left.to_owned(), right.clone()))
        })
        .collect()
}
pub fn handle_proof_params_def_list(
    ast: Pair<Rule>,
    _defined_consts: &[(String, Type)],
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
                | Expression::Identifier(Identifier::ProofIdentifier(ProofIdentifier::LoopVar(_)))
                | Expression::Identifier(Identifier::ProofIdentifier(ProofIdentifier::Const(_))) => {}
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
) -> Result<Vec<(String, Type)>> {
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

pub fn handle_const_decl(ast: Pair<Rule>) -> (String, Type) {
    let mut inner = ast.into_inner();
    let name = inner.next().unwrap().as_str().to_owned();
    let tipe = handle_type(inner.next().unwrap());

    (name, tipe)
}
