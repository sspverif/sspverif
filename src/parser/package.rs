use crate::expressions::Expression;
use crate::identifier::game_ident::GameIdentifier;
use crate::identifier::game_ident::GameIdentifier::Const;
use crate::identifier::pkg_ident::PackageConstIdentifier;
use crate::identifier::pkg_ident::PackageIdentifier;
use crate::identifier::pkg_ident::PackageImportsLoopVarIdentifier;
use crate::identifier::pkg_ident::PackageLocalIdentifier;
use crate::identifier::pkg_ident::PackageOracleArgIdentifier;
use crate::identifier::pkg_ident::PackageOracleCodeLoopVarIdentifier;
use crate::identifier::pkg_ident::PackageStateIdentifier;
use crate::identifier::Identifier;
use crate::package::OracleDef;
use crate::package::OracleSig;
use crate::package::Package;
use crate::statement::CodeBlock;
use crate::statement::FilePosition;
use crate::statement::Statement;
use crate::types::Type;
use crate::util::resolver::Named;
use crate::util::scope;
use crate::util::scope::Declaration;
use crate::util::scope::OracleContext;
use crate::util::scope::Scope;
use crate::util::scope::ValidityContext;
use crate::writers::smt::declare::declare_const;
use crate::writers::smt::exprs::SmtAnd;
use crate::writers::smt::exprs::SmtAssert;
use crate::writers::smt::exprs::SmtEq2;
use crate::writers::smt::exprs::SmtExpr;
use crate::writers::smt::exprs::SmtIte;
use crate::writers::smt::exprs::SmtLt;
use crate::writers::smt::exprs::SmtLte;
use crate::writers::smt::exprs::SmtNot;
use crate::writers::smt::sorts::SmtInt;

use super::common::*;
use super::Rule;

use pest::iterators::Pair;

use std::collections::HashMap;
use std::convert::TryInto;
use std::hash::Hash;

pub fn handle_decl_list<F: Fn(String, Type) -> Declaration>(
    decl_list: Pair<Rule>,
    scope: &mut Scope,
    file_name: &str,
    type_to_declaration: F,
) -> Result<Vec<(String, Type, FilePosition)>, scope::Error> {
    decl_list
        .into_inner()
        .map(|entry| {
            let span = entry.as_span();
            let file_pos = FilePosition::from_span(file_name, span);
            let mut inner = entry.into_inner();
            let identifier = inner.next().unwrap().as_str();
            let tipe = handle_type(inner.next().unwrap());
            scope.declare(
                identifier,
                type_to_declaration(identifier.to_string(), tipe.clone()),
            )?;
            Ok((identifier.to_string(), tipe, file_pos))
        })
        .collect()
}

pub fn handle_types_list(types: Pair<Rule>) -> Vec<String> {
    types
        .into_inner()
        .map(|entry| entry.as_str().to_string())
        .collect()
}

// TODO: identifier is optional
pub fn handle_arglist(arglist: Pair<Rule>) -> Vec<(String, Type)> {
    println!("handle_arglist rule: {:?}", arglist.as_rule());
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

#[derive(Debug)]
pub enum ParseExpressionError {
    UndefinedIdentifier(String, FilePosition),
}

impl std::fmt::Display for ParseExpressionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseExpressionError::UndefinedIdentifier(name, pos) => write!(
                f,
                "syntax error at {pos}: use of undefined identifier `{name}`"
            ),
        }
    }
}

impl std::error::Error for ParseExpressionError {}

pub fn handle_expression(
    expr: Pair<Rule>,
    file_name: &str,
    scope: &Scope,
) -> Result<Expression, ParseExpressionError> {
    Ok(match expr.as_rule() {
        // expr_equals | expr_not_equals | fn_call | table_access | identifier
        Rule::expr_add => {
            let mut inner = expr.into_inner();
            let lhs = handle_expression(inner.next().unwrap(), file_name, scope)?;
            let rhs = handle_expression(inner.next().unwrap(), file_name, scope)?;
            Expression::Add(Box::new(lhs), Box::new(rhs))
        }
        Rule::expr_sub => {
            let mut inner = expr.into_inner();
            let lhs = handle_expression(inner.next().unwrap(), file_name, scope)?;
            let rhs = handle_expression(inner.next().unwrap(), file_name, scope)?;
            Expression::Sub(Box::new(lhs), Box::new(rhs))
        }
        Rule::expr_mul => {
            let mut inner = expr.into_inner();
            let lhs = handle_expression(inner.next().unwrap(), file_name, scope)?;
            let rhs = handle_expression(inner.next().unwrap(), file_name, scope)?;
            Expression::Mul(Box::new(lhs), Box::new(rhs))
        }
        Rule::expr_div => {
            let mut inner = expr.into_inner();
            let lhs = handle_expression(inner.next().unwrap(), file_name, scope)?;
            let rhs = handle_expression(inner.next().unwrap(), file_name, scope)?;
            Expression::Div(Box::new(lhs), Box::new(rhs))
        }
        Rule::expr_and => Expression::And(
            expr.into_inner()
                .map(|expr| handle_expression(expr, file_name, scope))
                .collect::<Result<_, _>>()?,
        ),
        Rule::expr_or => Expression::Or(
            expr.into_inner()
                .map(|expr| handle_expression(expr, file_name, scope))
                .collect::<Result<_, _>>()?,
        ),
        Rule::expr_xor => Expression::Xor(
            expr.into_inner()
                .map(|expr| handle_expression(expr, file_name, scope))
                .collect::<Result<_, _>>()?,
        ),
        Rule::expr_not => {
            let mut inner = expr.into_inner();
            let content = handle_expression(inner.next().unwrap(), file_name, scope)?;
            Expression::Not(Box::new(content))
        }
        Rule::expr_equals => Expression::Equals(
            expr.into_inner()
                .map(|expr| handle_expression(expr, file_name, scope))
                .collect::<Result<_, _>>()?,
        ),
        Rule::expr_not_equals => Expression::Not(Box::new(Expression::Equals(
            expr.into_inner()
                .map(|expr| handle_expression(expr, file_name, scope))
                .collect::<Result<_, _>>()?,
        ))),
        Rule::expr_none => {
            let tipe = handle_type(expr.into_inner().next().unwrap());
            Expression::None(tipe)
        }
        Rule::expr_some => {
            let expr = handle_expression(expr.into_inner().next().unwrap(), file_name, scope)?;
            Expression::Some(Box::new(expr))
        }
        Rule::expr_unwrap => {
            let expr = handle_expression(expr.into_inner().next().unwrap(), file_name, scope)?;
            Expression::Unwrap(Box::new(expr))
        }
        Rule::expr_newtable => {
            let mut inner = expr.into_inner();
            let idxtipe = handle_type(inner.next().unwrap());
            let valtipe = handle_type(inner.next().unwrap());
            Expression::EmptyTable(Type::Table(Box::new(idxtipe), Box::new(valtipe)))
        }
        Rule::table_access => {
            let mut inner = expr.into_inner();
            let ident_name = inner.next().unwrap().as_str();
            let ident = handle_identifier_in_code_rhs(ident_name, scope).unwrap();
            let expr = handle_expression(inner.next().unwrap(), file_name, scope)?;
            // TODO properly parse this identifier
            Expression::TableAccess(ident, Box::new(expr))
        }
        Rule::fn_call => {
            let span = expr.as_span();
            let file_pos = FilePosition::from_span(file_name, span);
            let mut inner = expr.into_inner();
            let ident = inner.next().unwrap().as_str();
            let args = match inner.next() {
                None => vec![],
                Some(inner_args) => inner_args
                    .into_inner()
                    .map(|expr| handle_expression(expr, file_name, scope))
                    .collect::<Result<_, _>>()?,
            };
            let decl = scope
                .lookup(ident)
                .ok_or(ParseExpressionError::UndefinedIdentifier(
                    ident.to_string(),
                    file_pos,
                ))?;
            let ident = decl.into_identifier().unwrap();
            Expression::FnCall(ident, args)
        }
        Rule::identifier => {
            let span = expr.as_span();
            let file_pos = FilePosition::from_span(file_name, span);
            let name = expr.as_str().to_string();
            let decl = scope
                .lookup(&name)
                .ok_or(ParseExpressionError::UndefinedIdentifier(
                    name.clone(),
                    file_pos,
                ))?;

            let expect_context = ValidityContext::Package;
            let got_context = decl.validity_context();
            assert_eq!(decl.validity_context(), expect_context, "invariant does not hold! when looking up name `{name}` in scope, we got declaration {decl:?}, which is valid in context {got_context:?} but we are in context {expect_context:?}.");

            let ident = match decl {
                Declaration::Identifier(ident) => ident,
                Declaration::Oracle(_, _) => {
                    todo!("handle error, user tried assigning to an oracle")
                }
            };
            Expression::Identifier(ident)
        }
        Rule::literal_boolean => {
            let litval = expr.as_str().to_string();
            Expression::BooleanLiteral(litval)
        }
        Rule::literal_integer => {
            let litval = expr.as_str().trim().to_string();

            Expression::IntegerLiteral(litval.parse().unwrap_or_else(|_| {
                panic!(
                    "error at position {:?}..{:?}: could not parse as int: {}",
                    expr.as_span().start_pos().line_col(),
                    expr.as_span().end_pos().line_col(),
                    expr.as_str(),
                )
            }))
        }
        Rule::literal_emptyset => {
            let tipe = handle_type(expr.into_inner().next().unwrap());
            Expression::Typed(Type::Set(Box::new(tipe)), Box::new(Expression::Set(vec![])))
        }
        Rule::expr_tuple => Expression::Tuple(
            expr.into_inner()
                .map(|expr| handle_expression(expr, file_name, scope))
                .collect::<Result<_, _>>()?,
        ),
        Rule::expr_list => Expression::List(
            expr.into_inner()
                .map(|expr| handle_expression(expr, file_name, scope))
                .collect::<Result<_, _>>()?,
        ),
        Rule::expr_set => Expression::Set(
            expr.into_inner()
                .map(|expr| handle_expression(expr, file_name, scope))
                .collect::<Result<_, _>>()?,
        ),
        _ => unreachable!("Unhandled expression {:#?}", expr),
    })
}

fn transpose_option_result<T, E>(value: Option<Result<T, E>>) -> Result<Option<T>, E> {
    match value {
        Some(v) => v.map(Some),
        None => Ok(None),
    }
}

fn transpose_result_option<T, E>(value: Result<Option<T>, E>) -> Option<Result<T, E>> {
    match value {
        Err(err) => Some(Err(err)),
        Ok(Some(v)) => Some(Ok(v)),
        Ok(None) => None,
    }
}

#[derive(Debug)]
pub enum ParseCodeError {
    ParseExpression(ParseExpressionError),
    ScopeDeclare(crate::util::scope::Error),
    ParseIdentifier(ParseIdentifierError),
    UndefinedOracle(String),
    InvokingNonOracle(Identifier),
}

impl From<ParseExpressionError> for ParseCodeError {
    fn from(value: ParseExpressionError) -> Self {
        ParseCodeError::ParseExpression(value)
    }
}

impl core::fmt::Display for ParseCodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseCodeError::ParseExpression(err) => write!(f, "error parsing expression: {err}"),
            ParseCodeError::ScopeDeclare(err) => {
                write!(f, "error declaring name in scope: {err}")
            }
            ParseCodeError::ParseIdentifier(err) => write!(f, "error parsing identifier: {err}"),
            ParseCodeError::UndefinedOracle(name) => write!(f, "oracle {name} is not defined"),
            ParseCodeError::InvokingNonOracle(ident) => {
                write!(
                    f,
                    "code invokes identifier {ident:?}, but it is not an oracle"
                )
            }
        }
    }
}

pub fn handle_identifier_in_code_rhs(
    name: &str,
    scope: &Scope,
) -> Result<Identifier, ParseIdentifierError> {
    let ident = scope
        .lookup(name)
        .ok_or(ParseIdentifierError::Undefined(name.to_string()))?
        .into_identifier()
        .unwrap_or_else(|decl| {
            panic!(
                "expected an identifier, got a declaration {decl:?}",
                decl = decl
            )
        });

    Ok(ident)
}

pub fn handle_identifier_in_code_lhs(
    name: &str,
    scope: &mut Scope,
    pkg_name: &str,
    oracle_name: &str,
    file_name: &str,
    expected_type: Type,
) -> Result<Identifier, ParseIdentifierError> {
    let ident = if let Some(decl) = scope.lookup(name) {
        decl.into_identifier()
            .map_err(|decl| ParseIdentifierError::InvalidLeftHandSide(name.to_string(), decl))?
    } else {
        let ident =
            Identifier::PackageIdentifier(PackageIdentifier::Local(PackageLocalIdentifier {
                pkg_name: pkg_name.to_string(),
                oracle_name: oracle_name.to_string(),
                name: name.to_string(),
                tipe: expected_type.clone(),
            }));

        scope
            .declare(name, Declaration::Identifier(ident.clone()))
            .map_err(ParseIdentifierError::ScopeDeclareError)?;

        ident
    };

    if ident.get_type().unwrap() != expected_type {
        Err(ParseIdentifierError::TypeMismatch(ident, expected_type))
    } else {
        Ok(ident)
    }
}

#[derive(Debug)]
pub enum ParseIdentifierError {
    ParseExpression(ParseExpressionError),
    ScopeDeclareError(crate::util::scope::Error),
    Undefined(String),
    TypeMismatch(Identifier, Type),
    InvalidLeftHandSide(String, Declaration),
}

impl core::fmt::Display for ParseIdentifierError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseIdentifierError::InvalidLeftHandSide(name, decl) => {
                write!(f, "error parsing left-hand-side name `{name}`: expected an identifier, got {decl:?}")
            }
            ParseIdentifierError::ParseExpression(err) => {
                write!(f, "error parsing expression: {err}")
            }
            ParseIdentifierError::ScopeDeclareError(err) => {
                write!(f, "error declaring variable in scope: {err}")
            }
            ParseIdentifierError::Undefined(name) => {
                write!(f, "found undefined variable `{name}`.")
            }
            ParseIdentifierError::TypeMismatch(ident, expected_type) => write!(
                f,
                "expected identifier {name} to have type {exp_tipe:?}, but found {real_type:?}",
                name = ident.ident_ref(),
                exp_tipe = expected_type,
                real_type = ident.get_type().unwrap()
            ),
        }
    }
}

pub fn handle_code(
    code: Pair<Rule>,
    scope: &mut Scope,
    pkg_name: &str,
    oracle_name: &str,
    file_name: &str,
) -> Result<CodeBlock, ParseCodeError> {
    code.into_inner()
        .map(|stmt| {
            let span = stmt.as_span();
            let file_pos = FilePosition::from_span(file_name, span);

            let stmt = match stmt.as_rule() {
                // assign | return_stmt | abort | ite
                Rule::ite => {
                    let mut inner = stmt.into_inner();
                    let expr = handle_expression(inner.next().unwrap(), file_name, scope)?;
                    let ifcode = handle_code(
                        inner.next().unwrap(),
                        scope,
                        pkg_name,
                        oracle_name,
                        file_name,
                    )?;
                    let maybe_elsecode = inner.next();
                    let elsecode = match maybe_elsecode {
                        None => CodeBlock(vec![]),
                        Some(c) => handle_code(c, scope, pkg_name, oracle_name, file_name)?,
                    };

                    Statement::IfThenElse(expr, ifcode, elsecode, file_pos)
                }
                Rule::return_stmt => {
                    let mut inner = stmt.into_inner();
                    let maybe_expr = inner.next();
                    let expr = maybe_expr.map(|expr| handle_expression(expr, file_name, scope));
                    let expr = transpose_option_result(expr)?;
                    Statement::Return(expr, file_pos)
                }
                Rule::assert => {
                    let mut inner = stmt.into_inner();
                    let expr = handle_expression(inner.next().unwrap(), file_name, scope)?;
                    Statement::IfThenElse(
                        expr,
                        CodeBlock(vec![]),
                        CodeBlock(vec![Statement::Abort(file_pos.clone())]),
                        file_pos,
                    )
                }
                Rule::abort => Statement::Abort(file_pos),
                Rule::sample => {
                    let mut inner = stmt.into_inner();
                    let name = inner.next().unwrap().as_str();
                    let tipe = handle_type(inner.next().unwrap());
                    let ident = handle_identifier_in_code_lhs(
                        name,
                        scope,
                        pkg_name,
                        oracle_name,
                        file_name,
                        tipe.clone(),
                    )
                    .map_err(ParseCodeError::ParseIdentifier)?;
                    Statement::Sample(ident, None, None, tipe, file_pos)
                }

                Rule::assign => {
                    let mut inner = stmt.into_inner();
                    let name = inner.next().unwrap().as_str();
                    let expr = handle_expression(inner.next().unwrap(), file_name, scope)
                        .map_err(ParseCodeError::ParseExpression)?;

                    let expected_type = expr.get_type();
                    let ident = handle_identifier_in_code_lhs(
                        name,
                        scope,
                        pkg_name,
                        oracle_name,
                        file_name,
                        expected_type,
                    )
                    .map_err(ParseCodeError::ParseIdentifier)?;

                    Statement::Assign(ident, None, expr, file_pos)
                }

                Rule::table_sample => {
                    let mut inner = stmt.into_inner();
                    let name = inner.next().unwrap().as_str();
                    let index = handle_expression(inner.next().unwrap(), file_name, scope)?;
                    let tipe = handle_type(inner.next().unwrap());
                    let ident = handle_identifier_in_code_lhs(
                        name,
                        scope,
                        pkg_name,
                        oracle_name,
                        file_name,
                        Type::Table(Box::new(index.get_type()),  Box::new(tipe.clone())),
                    )
                    .map_err(ParseCodeError::ParseIdentifier)?;
                    Statement::Sample(ident, Some(index), None, tipe, file_pos)
                }

                Rule::table_assign => {
                    let mut inner = stmt.into_inner();
                    println!("DFGERT {inner:#?}");

                    let name = inner.next().unwrap().as_str();
                    let index = handle_expression(inner.next().unwrap(), file_name, scope)?;
                    let expr = handle_expression(inner.next().unwrap(), file_name, scope)
                        .map_err(ParseCodeError::ParseExpression)?;

                    let expected_type = match expr.get_type() {
                        Type::Maybe(t) => Type::Table(Box::new(index.get_type()), t),
                        _ => panic!("assigning non-maybe value to table {expr:?} in {oracle_name}, {pkg_name}", expr = expr, oracle_name=oracle_name, pkg_name=pkg_name),
                    };

                    let ident = handle_identifier_in_code_lhs(
                        name,
                        scope,
                        pkg_name,
                        oracle_name,
                        file_name,
                        expected_type,
                    )
                    .map_err(ParseCodeError::ParseIdentifier)?;

                    Statement::Assign(ident, Some(index), expr, file_pos)
                }

                Rule::invocation => {
                    let mut inner = stmt.into_inner();
                    let target_ident_name = inner.next().unwrap().as_str();
                    let maybe_index = inner.next().unwrap();
                    let (opt_idx, oracle_inv) = if maybe_index.as_rule() == Rule::table_index {
                        let mut inner_index = maybe_index.into_inner();
                        let index =
                            handle_expression(inner_index.next().unwrap(), file_name, scope)?;
                        (Some(index), inner.next().unwrap())
                    } else {
                        (None, maybe_index)
                    };

                    assert!(matches!(oracle_inv.as_rule(), Rule::oracle_call));

                    let mut inner = oracle_inv.into_inner();
                    let oracle_name = inner.next().unwrap().as_str();

                    let mut args = vec![];
                    let mut dst_inst_index = None;

                    for ast in inner {
                        match ast.as_rule() {
                            Rule::oracle_call_index => {
                                let index_expr_ast = ast.into_inner().next().unwrap();
                                dst_inst_index =
                                    Some(handle_expression(index_expr_ast, file_name, scope)?);
                            }
                            Rule::fn_call_arglist => {
                                let arglist: Result<Vec<_>, _> = ast
                                    .into_inner()
                                    .map(|expr| handle_expression(expr, file_name, scope))
                                    .collect();
                                let arglist = arglist?;
                                args.extend(arglist.into_iter())
                            }
                            _ => unreachable!(),
                        }
                    }

                    let oracle_decl = scope
                        .lookup(oracle_name)
                        .ok_or(ParseCodeError::UndefinedOracle(oracle_name.to_string()))?;

                    let expected_type = match oracle_decl {
                        Declaration::Oracle(context, oracle_sig) => Ok(oracle_sig.tipe),
                        Declaration::Identifier(ident) => {
                            Err(ParseCodeError::InvokingNonOracle(ident))
                        }
                        other => panic!("found old-style declaration {other:?}"),
                    }?;

                    let target_ident = handle_identifier_in_code_lhs(
                        target_ident_name,
                        scope,
                        pkg_name,
                        oracle_name,
                        file_name,
                        expected_type,
                    )
                    .map_err(ParseCodeError::ParseIdentifier)?;

                    Statement::InvokeOracle {
                        id: target_ident,
                        opt_idx,
                        opt_dst_inst_idx: dst_inst_index,
                        name: oracle_name.to_owned(),
                        args,
                        target_inst_name: None,
                        tipe: None,
                        file_pos,
                    }
                }
                Rule::parse => {
                    let mut inner = stmt.into_inner();
                    let list = inner.next().unwrap();
                    let expr = inner.next().unwrap();

                    let expr = handle_expression(expr, file_name, scope)?;
                    let tipe = expr.get_type();

                    let tipes = match tipe {
                        Type::Tuple(tipes) => tipes,
                        other => panic!("expected tuple type in parse, but got {other:?} in {file_name}, {pkg_name}, {oracle_name}, {expr:?}", other=other, file_name=file_name, pkg_name=pkg_name, oracle_name=oracle_name, expr=expr)
                    };

                    let idents = list
                        .into_inner()
                        .enumerate()
                        .map(|(i, ident_name)| {
                            handle_identifier_in_code_lhs(ident_name.as_str(), scope, pkg_name, oracle_name, file_name, tipes[i].clone())
                                .map_err(ParseCodeError::ParseIdentifier)
                        })
                        .collect::<Result<_,_>>()?;


                    Statement::Parse(idents, expr, file_pos)
                }
                Rule::for_ => {
                    let mut parsed: Vec<Pair<Rule>> = stmt.into_inner().collect();
                    let decl_var_name = parsed[0].as_str();
                    let lower_bound = handle_expression(parsed.remove(1), file_name, scope)?;
                    let lower_bound_type = parsed[1].as_str();
                    let bound_var_name = parsed[2].as_str();
                    let upper_bound_type = parsed[3].as_str();
                    let upper_bound = handle_expression(parsed.remove(4), file_name, scope)?;

                    if decl_var_name != bound_var_name {
                        todo!("return proper error here")
                    }

                    let lower_bound_type = match lower_bound_type {
                        "<" => ForComp::Lt,
                        "<=" => ForComp::Lte,
                        _ => panic!(),
                    };

                    let upper_bound_type = match upper_bound_type {
                        "<" => ForComp::Lt,
                        "<=" => ForComp::Lte,
                        _ => panic!(),
                    };
                    let loopvar = PackageOracleCodeLoopVarIdentifier {
                        name: decl_var_name.to_string(),
                        pkg_name: pkg_name.to_string(),
                        start: Box::new(lower_bound.clone()),
                        end: Box::new(upper_bound.clone()),
                        start_comp: lower_bound_type,
                        end_comp: upper_bound_type,
                    };
                    let loopvar: Identifier = loopvar.into();

                    scope.enter();
                    scope
                        .declare(decl_var_name, Declaration::Identifier(loopvar.clone()))
                        .unwrap();

                    let body =
                        handle_code(parsed.remove(4), scope, pkg_name, oracle_name, file_name)?;
                    scope.leave();

                    Statement::For(loopvar, lower_bound, upper_bound, body, file_pos)
                }
                _ => {
                    unreachable!("{:#?}", stmt)
                }
            };

            Ok(stmt)
        })
        .collect::<Result<_, _>>()
        .map(CodeBlock)
}

#[derive(Debug)]
pub enum ParseOracleDefError {
    ParseOracleSig(ParseOracleSigError),
    ParseCode(ParseCodeError),
}

impl core::fmt::Display for ParseOracleDefError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseOracleDefError::ParseOracleSig(err) => {
                write!(f, "error parsing oracle signature: {err}")
            }
            ParseOracleDefError::ParseCode(err) => write!(f, "error parsing oracle code: {err}"),
        }
    }
}

pub fn handle_oracle_def(
    oracle_def: Pair<Rule>,
    scope: &mut Scope,
    pkg_name: &str,
    file_name: &str,
) -> Result<OracleDef, ParseOracleDefError> {
    let span = oracle_def.as_span();
    let file_pos = FilePosition::from_span(file_name, span);
    let mut inner = oracle_def.into_inner();

    let sig =
        handle_oracle_sig(inner.next().unwrap()).map_err(ParseOracleDefError::ParseOracleSig)?;

    scope.enter();

    for (name, tipe) in &sig.args {
        scope
            .declare(
                name,
                Declaration::Identifier(Identifier::PackageIdentifier(
                    PackageIdentifier::OracleArg(PackageOracleArgIdentifier {
                        pkg_name: pkg_name.to_string(),
                        oracle_name: sig.name.clone(),
                        name: name.clone(),
                        tipe: tipe.clone(),
                    }),
                )),
            )
            .unwrap();
    }

    let code = handle_code(inner.next().unwrap(), scope, pkg_name, &sig.name, file_name)
        .map_err(ParseOracleDefError::ParseCode)?;

    scope.leave();

    let oracle_def = OracleDef {
        sig,
        code,
        file_pos,
    };

    Ok(oracle_def)
}

#[derive(Debug)]
pub enum ParsePackageError {
    ParseParams(scope::Error),
    ParseState(scope::Error),
    ParseImportOracleSig(ParseImportOraclesError),
    ParseOracleDef(ParseOracleDefError),
}

impl core::fmt::Display for ParsePackageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsePackageError::ParseParams(err) => {
                write!(f, "error parsing package params/consts: {err}")
            }
            ParsePackageError::ParseState(err) => write!(f, "error parsing package state: {err}"),
            ParsePackageError::ParseImportOracleSig(err) => {
                write!(f, "error parsing oracle imports: {err}")
            }
            ParsePackageError::ParseOracleDef(err) => {
                write!(f, "error parsing oracle definition: {err}")
            }
        }
    }
}

pub fn handle_pkg_spec(
    pkg_spec: Pair<Rule>,
    scope: &mut Scope,
    pkg_name: &str,
    file_name: &str,
) -> Result<Package, ParsePackageError> {
    let mut oracles = vec![];
    let mut state = None;
    let mut params = None;
    let mut types = None;
    let mut imported_oracles = HashMap::new();

    // TODO(2024-04-03): get rid of the unwraps in params, state, import_oracles
    for spec in pkg_spec.into_inner() {
        match spec.as_rule() {
            Rule::types => {
                types = spec.into_inner().next().map(handle_types_list);
            }
            Rule::params => {
                scope.enter();

                let ast = spec.into_inner().next();
                let declaration_constructor = |name, tipe| {
                    Declaration::Identifier(Identifier::PackageIdentifier(
                        PackageIdentifier::Const(PackageConstIdentifier {
                            pkg_name: pkg_name.to_string(),
                            name,
                            tipe,
                            game_assignment: None,
                        }),
                    ))
                };
                let params_option_result: Option<Result<_, _>> = ast.map(|params| {
                    handle_decl_list(params, scope, file_name, declaration_constructor)
                        .map_err(ParsePackageError::ParseParams)
                });

                params = transpose_option_result(params_option_result)?;
            }
            Rule::state => {
                scope.enter();
                let ast = spec.into_inner().next();
                let declaration_constructor = |name, tipe| {
                    Declaration::Identifier(Identifier::PackageIdentifier(
                        PackageIdentifier::State(PackageStateIdentifier {
                            pkg_name: pkg_name.to_string(),
                            name,
                            tipe,
                        }),
                    ))
                };
                let state_option_result: Option<Result<_, _>> = ast.map(|state| {
                    handle_decl_list(state, scope, file_name, declaration_constructor)
                        .map_err(ParsePackageError::ParseParams)
                });
                state = transpose_option_result(state_option_result)?;
            }
            Rule::import_oracles => {
                scope.enter();

                let mut loopvar_scope = scope.clone();

                let body_ast = spec.into_inner().next().unwrap();
                handle_import_oracles_body(
                    body_ast,
                    &mut imported_oracles,
                    pkg_name,
                    file_name,
                    scope,
                    &mut loopvar_scope,
                )
                .map_err(ParsePackageError::ParseImportOracleSig)?
            }
            Rule::oracle_def => {
                oracles.push(
                    handle_oracle_def(spec, scope, pkg_name, file_name)
                        .map_err(ParsePackageError::ParseOracleDef)?,
                );
            }
            _ => unreachable!("unhandled ast node in package: {}", spec),
        }
    }

    Ok(Package {
        name: pkg_name.to_string(),
        oracles,
        types: types.unwrap_or_default(),
        params: params.unwrap_or_default(),
        imports: imported_oracles
            .iter()
            .map(|(_k, (v, loc))| (v.clone(), loc.clone()))
            .collect(),
        state: state.unwrap_or_default(),
        split_oracles: vec![],
    })
}

#[derive(Debug)]
pub enum ParseOracleSigError {}

impl core::fmt::Display for ParseOracleSigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {}
    }
}

pub fn handle_oracle_sig(oracle_sig: Pair<Rule>) -> Result<OracleSig, ParseOracleSigError> {
    let mut inner = oracle_sig.into_inner();
    let name = inner.next().unwrap().as_str();
    let maybe_arglist = inner.next().unwrap();
    let args = if maybe_arglist.as_str() == "" {
        vec![]
    } else {
        handle_arglist(maybe_arglist.into_inner().next().unwrap())
    };

    let maybe_tipe = inner.next();
    let tipe = match maybe_tipe {
        None => Type::Empty,
        Some(t) => handle_type(t),
    };

    Ok(OracleSig {
        name: name.to_string(),
        tipe,
        args,
        multi_inst_idx: MultiInstanceIndices::new(vec![]),
    })
}

#[derive(Debug)]
pub enum ParseImportsOracleSigError {
    IndexParseError(ParseExpressionError),
}

impl std::fmt::Display for ParseImportsOracleSigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseImportsOracleSigError::IndexParseError(e) => write!(f, "error parsing index: {e}"),
        }
    }
}

impl std::error::Error for ParseImportsOracleSigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ParseImportsOracleSigError::IndexParseError(e) => Some(e),
        }
    }
}

pub fn handle_oracle_imports_oracle_sig(
    oracle_sig: Pair<Rule>,
    file_name: &str,
    scope: &mut Scope,
    loopvar_scope: &Scope,
) -> Result<OracleSig, ParseImportsOracleSigError> {
    println!("{:?}", oracle_sig.as_rule());

    let _span = oracle_sig.as_span();

    let mut inner = oracle_sig.into_inner();
    let name = inner.next().unwrap().as_str();

    let (multi_inst_idx, args) = {
        let mut multi_inst_idx = vec![];
        let mut arglist = vec![];

        while let Some(next) = inner.peek() {
            match next.as_rule() {
                Rule::indices_expr => {
                    let indices: Vec<_> = next
                        .into_inner()
                        .map(|expr| handle_expression(expr, file_name, loopvar_scope))
                        .collect::<Result<_, _>>()
                        .map_err(ParseImportsOracleSigError::IndexParseError)?;
                    multi_inst_idx.extend_from_slice(&indices);
                    inner.next();
                }
                Rule::fn_maybe_arglist => {
                    if !next.as_str().is_empty() {
                        arglist = handle_arglist(next.into_inner().next().unwrap());
                    }
                    inner.next();
                }
                _ => break,
            }
        }

        (MultiInstanceIndices::new(multi_inst_idx), arglist)
    };

    let maybe_tipe = inner.next();
    let tipe = match maybe_tipe {
        None => Type::Empty,
        Some(t) => handle_type(t),
    };

    Ok(OracleSig {
        name: name.to_string(),
        tipe,
        args,
        multi_inst_idx,
    })
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ForComp {
    Lt,
    Lte,
}

#[derive(Debug)]
pub struct ForCompError(String);

impl std::fmt::Display for ForCompError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "not a valid comparison operator for for loops: {:?}",
            self.0
        )
    }
}

impl std::error::Error for ForCompError {}

impl crate::error::LocationError for ParseImportOraclesError {
    fn file_pos<'a>(&'a self) -> &'a FilePosition {
        match self {
            ParseImportOraclesError::IdentifierMismatch(_, _, file_pos)
            | ParseImportOraclesError::ParseImportOracleSig(_, file_pos)
            | ParseImportOraclesError::ParseStartExpression(_, file_pos)
            | ParseImportOraclesError::ParseEndExpression(_, file_pos)
            | ParseImportOraclesError::InvalidStartComparison(_, file_pos)
            | ParseImportOraclesError::InvalidEndComparison(_, file_pos) => file_pos,
            ParseImportOraclesError::DeclareError(_, file_pos) => file_pos,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MultiInstanceIndices {
    pub(crate) indices: Vec<Expression>,
}
impl MultiInstanceIndices {
    pub(crate) fn new(indices: Vec<Expression>) -> Self {
        Self { indices }
    }

    pub(crate) fn empty() -> Self {
        Self { indices: vec![] }
    }
}

/// A [`MultiInstanceIndicesGroup`] contains a list of [`MultiInstanceIndices`] that all represent
/// the same index, but cover different ranges. The purpose is that the group folds the individual
/// elements into a single one, by merging the ranges specified in the [`ForSpec`] entries.
pub struct MultiInstanceIndicesGroup(Vec<MultiInstanceIndices>);

impl MultiInstanceIndicesGroup {
    pub fn new(v: Vec<MultiInstanceIndices>) -> Self {
        Self(v)
    }

    pub(crate) fn smt_check_total(
        &self,
        assumptions: Vec<SmtExpr>,
        consts: &[&str],
        varname: &str,
    ) -> Vec<SmtExpr> {
        let declares: Vec<_> = consts
            .iter()
            .map(|const_name| declare_const(*const_name, SmtInt))
            .collect();

        let predicate = self.smt_totality_check_function(varname);

        let neg_claim = SmtNot(SmtEq2 {
            lhs: predicate,
            rhs: 1usize,
        })
        .into();

        let and_terms = assumptions
            .into_iter()
            .chain(vec![neg_claim].into_iter())
            .collect();

        let assert = SmtAssert(SmtAnd(and_terms));

        let mut out_statements = declares;
        out_statements.push(assert.into());

        out_statements
        /*
         *
         * declare const n Int
         * ...
         * declare const x Int
         *
         * assert AND(assumptions)  => sum of predicates = 1
         * -> expect unsat
         *
         */
    }

    fn smt_totality_check_function(&self, varname: &str) -> SmtExpr {
        let terms = self.0.iter().map(|indices| {
            SmtIte {
                cond: indices.smt_range_predicate(varname),
                then: 1,
                els: 0,
            }
            .into()
        });

        let add = ["+"].iter().map(|&add| add.into());
        let zero = [0].iter().map(|&zero| zero.into());

        // add the zero term so the operation doesn't fail if there are no terms
        SmtExpr::List(add.chain(terms).chain(zero).collect())
    }
}

impl MultiInstanceIndices {
    /// returns smt code that checks whether a variable with name `varname` is in the range.
    /// currently only works for one-dimensional indices and panics for higher dimensions.
    pub(crate) fn smt_range_predicate(&self, varname: &str) -> SmtExpr {
        //assert!(self.indices.len() == 1);
        match &self.indices[0] {
            Expression::IntegerLiteral(index) => SmtEq2 {
                lhs: *index,
                rhs: varname,
            }
            .into(),
            // I don't think we need to check totality for imports inside the package's import
            // block, so we don't need to handle ImportsLoopVar.
            // Expression::Identifier(Identifier::PackageIdentifier(
            // PackageIdentifier::ImportsLoopVar(loopvar),
            // )) => {
            // let start_comp: SmtExpr = match loopvar.start_comp {
            // ForComp::Lt => SmtLt((*loopvar.start).clone(), varname).into(),
            // ForComp::Lte => SmtLte((*loopvar.start).clone(), varname).into(),
            // };
            //
            // let end_comp: SmtExpr = match loopvar.end_comp {
            // ForComp::Lt => SmtLt(varname, (*loopvar.end).clone()).into(),
            // ForComp::Lte => SmtLte(varname, (*loopvar.end).clone()).into(),
            // };
            //
            // SmtAnd(vec![start_comp, end_comp]).into()
            // }
            Expression::Identifier(Identifier::GameIdentifier(GameIdentifier::Const(
                game_const_ident,
            ))) => SmtEq2 {
                lhs: &game_const_ident.name,
                rhs: varname,
            }
            .into(),
            Expression::Identifier(Identifier::GameIdentifier(GameIdentifier::LoopVar(
                game_loop_var,
            ))) => {
                let lower_comp: SmtExpr = match game_loop_var.start_comp {
                    ForComp::Lt => SmtLt((*game_loop_var.start).clone(), varname).into(),
                    ForComp::Lte => SmtLte((*game_loop_var.start).clone(), varname).into(),
                };

                let upper_comp: SmtExpr = match game_loop_var.end_comp {
                    ForComp::Lt => SmtLt((*game_loop_var.end).clone(), varname).into(),
                    ForComp::Lte => SmtLte((*game_loop_var.end).clone(), varname).into(),
                };

                SmtAnd(vec![lower_comp, upper_comp]).into()
            }
            Expression::Identifier(Identifier::Parameter(pkg_const)) => SmtEq2 {
                lhs: pkg_const.name_in_comp.clone(),
                rhs: varname,
            }
            .into(),
            other => unreachable!(
                "in smt_range_predicate, found unhandled expression variant {expr:?}",
                expr = other
            ),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub struct ForSpec {
    ident: Identifier,
    start: Expression,
    end: Expression,
    start_comp: ForComp,
    end_comp: ForComp,
}

impl Named for ForSpec {
    fn as_name(&self) -> &str {
        self.ident.ident_ref()
    }
}

impl ForSpec {
    pub fn new(
        ident: Identifier,
        start: Expression,
        end: Expression,
        start_comp: ForComp,
        end_comp: ForComp,
    ) -> Self {
        Self {
            ident,
            start,
            end,
            start_comp,
            end_comp,
        }
    }

    pub fn ident(&self) -> &Identifier {
        &self.ident
    }

    pub fn start(&self) -> &Expression {
        &self.start
    }

    pub fn end(&self) -> &Expression {
        &self.end
    }

    pub fn start_comp(&self) -> &ForComp {
        &self.start_comp
    }

    pub fn end_comp(&self) -> &ForComp {
        &self.end_comp
    }

    pub(crate) fn map_identifiers<F: Fn(&Identifier) -> Identifier>(&self, f: F) -> Self {
        Self {
            start: map_ident_expr(&self.start, &f),
            end: map_ident_expr(&self.end, &f),
            ..self.clone()
        }
    }
}

fn map_ident_expr<F: Fn(&Identifier) -> Identifier>(expr: &Expression, f: &F) -> Expression {
    if let Expression::Identifier(id) = expr {
        Expression::Identifier(f(id))
    } else {
        expr.clone()
    }
}

impl std::convert::TryFrom<&str> for ForComp {
    type Error = ForCompError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "<=" => Ok(ForComp::Lte),
            "<" => Ok(ForComp::Lt),
            _ => Err(ForCompError(value.to_string())),
        }
    }
}

#[derive(Debug)]
pub enum ParseImportOraclesError {
    IdentifierMismatch(String, String, FilePosition),
    ParseStartExpression(ParseExpressionError, FilePosition),
    ParseEndExpression(ParseExpressionError, FilePosition),
    InvalidStartComparison(ForCompError, FilePosition),
    InvalidEndComparison(ForCompError, FilePosition),
    ParseImportOracleSig(ParseImportsOracleSigError, FilePosition),
    DeclareError(crate::util::scope::Error, FilePosition),
}

impl std::fmt::Display for ParseImportOraclesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseImportOraclesError::IdentifierMismatch(fst, snd, _filepos) => {
                write!(
                    f,
                    "for loop spec uses different identifiers: {fst:?} != {snd:?}"
                )
            }
            ParseImportOraclesError::InvalidStartComparison(comp_err, _filepos) => {
                write!(f, "first loop comparison is invalid: {comp_err}")
            }
            ParseImportOraclesError::InvalidEndComparison(comp_err, _filepos) => {
                write!(f, "second loop comparison is invalid: {comp_err}")
            }
            ParseImportOraclesError::ParseImportOracleSig(err, _filepos) => {
                write!(f, "error parsing import oracle signature: {err}")
            }
            ParseImportOraclesError::ParseStartExpression(err, _) => {
                write!(f, "error parsing loop start expression: {err}")
            }
            ParseImportOraclesError::ParseEndExpression(err, _) => {
                write!(f, "error parsing loop end expression: {err}")
            }
            ParseImportOraclesError::DeclareError(err, _) => {
                write!(f, "erroring declaring identifier: {err}")
            }
        }
    }
}

impl std::error::Error for ParseImportOraclesError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ParseImportOraclesError::IdentifierMismatch(_, _, _) => None,
            ParseImportOraclesError::ParseImportOracleSig(e, _) => Some(e),
            ParseImportOraclesError::DeclareError(e, _) => Some(e),
            ParseImportOraclesError::ParseStartExpression(e, _) => Some(e),
            ParseImportOraclesError::ParseEndExpression(e, _) => Some(e),
            ParseImportOraclesError::InvalidStartComparison(e, _)
            | ParseImportOraclesError::InvalidEndComparison(e, _) => Some(e),
        }
    }
}

pub fn handle_import_oracles_body(
    ast: Pair<Rule>,
    imported_oracles: &mut HashMap<String, (OracleSig, FilePosition)>,
    pkg_name: &str,
    file_name: &str,
    scope: &mut Scope,
    loopvar_scope: &mut Scope,
) -> Result<(), ParseImportOraclesError> {
    assert_eq!(ast.as_rule(), Rule::import_oracles_body);

    for entry in ast.into_inner() {
        match entry.as_rule() {
            Rule::import_oracles_oracle_sig => {
                let file_pos = FilePosition::from_span(file_name, entry.as_span());
                let sig = handle_oracle_imports_oracle_sig(entry, file_name, scope, loopvar_scope)
                    .map_err(|e| {
                        ParseImportOraclesError::ParseImportOracleSig(e, file_pos.clone())
                    })?;
                imported_oracles.insert(sig.name.clone(), (sig.clone(), file_pos.clone()));
                scope
                    .declare(
                        &sig.name,
                        Declaration::Oracle(
                            OracleContext::Package {
                                pkg_name: pkg_name.to_string(),
                            },
                            sig.clone(),
                        ),
                    )
                    .map_err(|err| ParseImportOraclesError::DeclareError(err, file_pos))?;
            }

            Rule::import_oracles_for => {
                let for_span = entry.as_span();
                let mut for_ast = entry.into_inner();

                let ident_ast = for_ast.next().unwrap();
                let ident = ident_ast.as_str().to_string();

                let for_start_ast = for_ast.next().unwrap();
                let for_start_file_pos =
                    FilePosition::from_span(file_name, for_start_ast.as_span());
                let for_start =
                    handle_expression(for_start_ast, file_name, scope).map_err(|e| {
                        ParseImportOraclesError::ParseStartExpression(e, for_start_file_pos)
                    })?;

                let start_comp_ast = for_ast.next().unwrap();
                let start_comp_filepos =
                    FilePosition::from_span(file_name, start_comp_ast.as_span());
                let start_comp: ForComp = start_comp_ast.as_str().try_into().map_err(|e| {
                    ParseImportOraclesError::InvalidStartComparison(e, start_comp_filepos)
                })?;

                let ident2_ast = for_ast.next().unwrap();
                let ident2_span = ident2_ast.as_span();
                let ident2 = ident2_ast.as_str().to_string();

                let end_comp_ast = for_ast.next().unwrap();
                let end_comp_filepos = FilePosition::from_span(file_name, end_comp_ast.as_span());
                let end_comp: ForComp = end_comp_ast.as_str().try_into().map_err(|e| {
                    ParseImportOraclesError::InvalidEndComparison(e, end_comp_filepos)
                })?;

                let for_end_ast = for_ast.next().unwrap();
                let for_end_file_pos = FilePosition::from_span(file_name, for_end_ast.as_span());
                let for_end = handle_expression(for_end_ast, file_name, scope).map_err(|e| {
                    ParseImportOraclesError::ParseEndExpression(e, for_end_file_pos)
                })?;

                if ident != ident2 {
                    return Err(ParseImportOraclesError::IdentifierMismatch(
                        ident,
                        ident2,
                        FilePosition::from_span(file_name, ident2_span),
                    ));
                }

                let ident_data = PackageImportsLoopVarIdentifier {
                    pkg_name: pkg_name.to_string(),
                    name: ident.clone(),
                    start: Box::new(for_start.clone()),
                    end: Box::new(for_end.clone()),
                    start_comp,
                    end_comp,
                };

                let identifier =
                    Identifier::PackageIdentifier(PackageIdentifier::ImportsLoopVar(ident_data));

                loopvar_scope.enter();

                loopvar_scope
                    .declare(&ident, Declaration::Identifier(identifier))
                    .map_err(|e| {
                        ParseImportOraclesError::DeclareError(
                            e,
                            FilePosition::from_span(file_name, for_span),
                        )
                    })?;

                handle_import_oracles_body(
                    for_ast.next().unwrap(),
                    imported_oracles,
                    pkg_name,
                    file_name,
                    scope,
                    loopvar_scope,
                )?;
                loopvar_scope.leave();
            }

            _ => unreachable!(),
        }
    }
    Ok(())
}

pub fn handle_pkg(
    pkg: Pair<Rule>,
    file_name: &str,
) -> Result<(String, Package), ParsePackageError> {
    let mut inner = pkg.into_inner();
    let mut scope = crate::util::scope::Scope::new();
    let pkg_name = inner.next().unwrap().as_str();
    let spec = inner.next().unwrap();
    let pkg = handle_pkg_spec(spec, &mut scope, pkg_name, file_name)?;

    Ok((pkg_name.to_owned(), pkg))
}

#[cfg(test)]
mod tests2 {
    use pest::Parser;

    use crate::{
        expressions::Expression,
        identifier::{pkg_ident::PackageIdentifier, Identifier},
        parser::{common::handle_expression, package::ForComp, Rule, SspParser},
        types::Type,
        util::scope::Scope,
        writers::smt::exprs::{SmtLt, SmtLte},
    };

    use super::{MultiInstanceIndices, MultiInstanceIndicesGroup};

    #[test]
    fn example_smt_stuff() {
        let pkg_name = || "Foo".to_string();
        let mut scope = Scope::new();
        scope.enter();
        scope
            .declare(
                "n",
                crate::util::scope::Declaration::Identifier(Identifier::PackageIdentifier(
                    crate::identifier::pkg_ident::PackageIdentifier::Const(
                        crate::identifier::pkg_ident::PackageConstIdentifier {
                            pkg_name: pkg_name(),
                            name: "n".to_string(),
                            tipe: Type::Integer,
                            game_assignment: None,
                        },
                    ),
                )),
            )
            .unwrap();

        let mut parse_expr = |text: &str| {
            handle_expression(
                SspParser::parse(Rule::expression, text)
                    .unwrap()
                    .next()
                    .unwrap(),
                &mut scope,
            )
            .unwrap()
        };

        let end = parse_expr("(n - 1)").map(|expr| match expr {
            Expression::Identifier(Identifier::Scalar(x)) => {
                Expression::Identifier(Identifier::Local(x))
            }
            x => x,
        });

        let indices_group = MultiInstanceIndicesGroup(vec![MultiInstanceIndices::new(vec![
            Expression::Identifier(Identifier::PackageIdentifier(
                PackageIdentifier::ImportsLoopVar(
                    crate::identifier::pkg_ident::PackageImportsLoopVarIdentifier {
                        pkg_name: pkg_name(),
                        name: "i".to_string(),
                        start: Box::new(parse_expr("0")),
                        end: Box::new(end),
                        start_comp: ForComp::Lte,
                        end_comp: ForComp::Lt,
                    },
                ),
            )),
        ])]);

        let smt = indices_group.smt_check_total(
            vec![SmtLte(0, "x").into(), SmtLt("x", "n").into()],
            &["n"],
            "x",
        );

        for expr in smt {
            println!("{expr}")
        }

        println!("(check-sat)");
        println!("(get-model)")
    }
}
