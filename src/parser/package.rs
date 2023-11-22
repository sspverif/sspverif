use crate::expressions::Expression;
use crate::identifier::Identifier;
use crate::package::OracleDef;
use crate::package::OracleSig;
use crate::package::Package;
use crate::statement::CodeBlock;
use crate::statement::FilePosition;
use crate::statement::Statement;
use crate::types::Type;

use super::common::*;
use super::Rule;

use pest::iterators::Pair;

use std::collections::HashMap;

pub fn handle_decl_list(state: Pair<Rule>, file_name: &str) -> Vec<(String, Type, FilePosition)> {
    state
        .into_inner()
        .map(|entry| {
            let span = entry.as_span();
            let file_pos = FilePosition::from_span(file_name, span);
            let mut inner = entry.into_inner();
            let identifier = inner.next().unwrap().as_str();
            let tipe = handle_type(inner.next().unwrap());
            (identifier.to_string(), tipe, file_pos)
        })
        .collect()
}

pub fn handle_types_list(types: Pair<Rule>) -> Vec<Type> {
    types
        .into_inner()
        .map(|entry| Type::UserDefined(entry.as_str().to_string()))
        .collect()
}

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

pub fn handle_oracle_imports(oracle_sig: Pair<Rule>, file_path: &str) -> (OracleSig, FilePosition) {
    let span = oracle_sig.as_span();
    let file_pos = FilePosition::from_span(file_path, span);

    (handle_oracle_sig(oracle_sig), file_pos)
}

pub fn handle_oracle_sig(oracle_sig: Pair<Rule>) -> OracleSig {
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

    OracleSig {
        name: name.to_string(),
        tipe,
        args,
    }
}

pub fn handle_expression(expr: Pair<Rule>) -> Expression {
    match expr.as_rule() {
        // expr_equals | expr_not_equals | fn_call | table_access | identifier
        Rule::expr_add => {
            let mut inner = expr.into_inner();
            let lhs = handle_expression(inner.next().unwrap());
            let rhs = handle_expression(inner.next().unwrap());
            Expression::Add(Box::new(lhs), Box::new(rhs))
        }
        Rule::expr_sub => {
            let mut inner = expr.into_inner();
            let lhs = handle_expression(inner.next().unwrap());
            let rhs = handle_expression(inner.next().unwrap());
            Expression::Sub(Box::new(lhs), Box::new(rhs))
        }
        Rule::expr_mul => {
            let mut inner = expr.into_inner();
            let lhs = handle_expression(inner.next().unwrap());
            let rhs = handle_expression(inner.next().unwrap());
            Expression::Mul(Box::new(lhs), Box::new(rhs))
        }
        Rule::expr_div => {
            let mut inner = expr.into_inner();
            let lhs = handle_expression(inner.next().unwrap());
            let rhs = handle_expression(inner.next().unwrap());
            Expression::Div(Box::new(lhs), Box::new(rhs))
        }
        Rule::expr_and => Expression::And(expr.into_inner().map(handle_expression).collect()),
        Rule::expr_or => Expression::Or(expr.into_inner().map(handle_expression).collect()),
        Rule::expr_xor => Expression::Xor(expr.into_inner().map(handle_expression).collect()),
        Rule::expr_not => {
            let mut inner = expr.into_inner();
            let content = handle_expression(inner.next().unwrap());
            Expression::Not(Box::new(content))
        }
        Rule::expr_equals => Expression::Equals(expr.into_inner().map(handle_expression).collect()),
        Rule::expr_not_equals => Expression::Not(Box::new(Expression::Equals(
            expr.into_inner().map(handle_expression).collect(),
        ))),
        Rule::expr_none => {
            let tipe = handle_type(expr.into_inner().next().unwrap());
            Expression::None(tipe)
        }
        Rule::expr_some => {
            let expr = handle_expression(expr.into_inner().next().unwrap());
            Expression::Some(Box::new(expr))
        }
        Rule::expr_unwrap => {
            let expr = handle_expression(expr.into_inner().next().unwrap());
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
            let expr = handle_expression(inner.next().unwrap());
            Expression::TableAccess(Identifier::new_scalar(ident), Box::new(expr))
        }
        Rule::fn_call => {
            let mut inner = expr.into_inner();
            let ident = inner.next().unwrap().as_str();
            let args = match inner.next() {
                None => vec![],
                Some(inner_args) => inner_args.into_inner().map(handle_expression).collect(),
            };
            Expression::FnCall(Identifier::new_scalar(ident), args)
        }
        Rule::identifier => Identifier::new_scalar(expr.as_str()).to_expression(),
        Rule::literal_boolean => {
            let litval = expr.as_str().to_string();
            Expression::BooleanLiteral(litval)
        }
        Rule::literal_integer => {
            let litval = expr.as_str().to_string();
            Expression::IntegerLiteral(litval)
        }
        Rule::literal_emptyset => {
            let tipe = handle_type(expr.into_inner().next().unwrap());
            Expression::Typed(Type::Set(Box::new(tipe)), Box::new(Expression::Set(vec![])))
        }
        Rule::expr_tuple => Expression::Tuple(expr.into_inner().map(handle_expression).collect()),
        Rule::expr_list => Expression::List(expr.into_inner().map(handle_expression).collect()),
        Rule::expr_set => Expression::Set(expr.into_inner().map(handle_expression).collect()),
        _ => unreachable!("Unhandled expression {:#?}", expr),
    }
}

pub fn handle_code(code: Pair<Rule>, file_name: &str) -> CodeBlock {
    CodeBlock(
        code.into_inner()
            .map(|stmt| {
                let span = stmt.as_span();
                let file_pos = FilePosition::from_span(file_name, span);

                match stmt.as_rule() {
                    // assign | return_stmt | abort | ite
                    Rule::ite => {
                        let mut inner = stmt.into_inner();
                        let expr = handle_expression(inner.next().unwrap());
                        let ifcode = handle_code(inner.next().unwrap(), file_name);
                        let maybe_elsecode = inner.next();
                        let elsecode = match maybe_elsecode {
                            None => CodeBlock(vec![]),
                            Some(c) => handle_code(c, file_name),
                        };

                        Statement::IfThenElse(expr, ifcode, elsecode, file_pos)
                    }
                    Rule::return_stmt => {
                        let mut inner = stmt.into_inner();
                        let maybe_expr = inner.next();
                        let expr = maybe_expr.map(handle_expression);
                        Statement::Return(expr, file_pos)
                    }
                    Rule::assert => {
                        let mut inner = stmt.into_inner();
                        let expr = handle_expression(inner.next().unwrap());
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
                        let ident = Identifier::new_scalar(inner.next().unwrap().as_str());
                        let tipe = handle_type(inner.next().unwrap());
                        Statement::Sample(ident, None, None, tipe, file_pos)
                        //Statement::Assign(ident, Expression::Sample(tipe))
                    }
                    Rule::assign => {
                        let mut inner = stmt.into_inner();
                        let ident = Identifier::new_scalar(inner.next().unwrap().as_str());
                        let expr = handle_expression(inner.next().unwrap());
                        Statement::Assign(ident, None, expr, file_pos)
                    }
                    Rule::table_sample => {
                        let mut inner = stmt.into_inner();
                        let ident = Identifier::new_scalar(inner.next().unwrap().as_str());
                        let index = handle_expression(inner.next().unwrap());
                        let tipe = handle_type(inner.next().unwrap());
                        Statement::Sample(ident, Some(index), None, tipe, file_pos)
                    }
                    Rule::table_assign => {
                        let mut inner = stmt.into_inner();
                        let ident = Identifier::new_scalar(inner.next().unwrap().as_str());
                        let index = handle_expression(inner.next().unwrap());
                        let expr = handle_expression(inner.next().unwrap());
                        Statement::Assign(ident, Some(index), expr, file_pos)
                    }
                    Rule::invocation => {
                        let mut inner = stmt.into_inner();
                        let ident = Identifier::new_scalar(inner.next().unwrap().as_str());
                        let maybe_index = inner.next().unwrap();
                        let (opt_idx, oracle_inv) = if maybe_index.as_rule() == Rule::table_index {
                            let mut inner_index = maybe_index.into_inner();
                            let index = handle_expression(inner_index.next().unwrap());
                            (Some(index), inner.next().unwrap())
                        } else {
                            (None, maybe_index)
                        };

                        let mut inner = oracle_inv.into_inner();
                        let oracle_name = inner.next().unwrap().as_str();
                        let args = match inner.next() {
                            None => vec![],
                            Some(inner_args) => {
                                inner_args.into_inner().map(handle_expression).collect()
                            }
                        };

                        Statement::InvokeOracle {
                            id: ident,
                            opt_idx,
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

                        let idents = list
                            .into_inner()
                            .map(|ident_name| Identifier::new_scalar(ident_name.as_str()))
                            .collect();

                        let expr = handle_expression(expr);

                        Statement::Parse(idents, expr, file_pos)
                    }
                    Rule::for_ => {
                        let mut parsed: Vec<Pair<Rule>> = stmt.into_inner().collect();
                        let decl_var_name = parsed[0].as_str();
                        let lower_bound = handle_expression(parsed.remove(1));
                        let lower_bound_type = parsed[1].as_str();
                        let bound_var_name = parsed[2].as_str();
                        let upper_bound_type = parsed[3].as_str();
                        let upper_bound = handle_expression(parsed.remove(4));
                        let body = handle_code(parsed.remove(4), file_name);

                        if decl_var_name != bound_var_name {
                            todo!("return proper error here")
                        }

                        let lower_bound = match lower_bound_type {
                            "<" => Expression::Add(
                                Box::new(lower_bound),
                                Box::new(Expression::IntegerLiteral("1".to_string())),
                            ),
                            "<=" => lower_bound,
                            _ => panic!(),
                        };

                        let upper_bound = match upper_bound_type {
                            "<" => upper_bound,
                            "<=" => Expression::Add(
                                Box::new(upper_bound),
                                Box::new(Expression::IntegerLiteral("1".to_string())),
                            ),
                            _ => panic!(),
                        };

                        let ident = Identifier::Scalar(decl_var_name.to_string());
                        Statement::For(ident, lower_bound, upper_bound, body, file_pos)
                    }
                    _ => {
                        unreachable!("{:#?}", stmt)
                    }
                }
            })
            .collect(),
    )
}

pub fn handle_oracle_def(oracle_def: Pair<Rule>, file_name: &str) -> OracleDef {
    let span = oracle_def.as_span();
    let file_pos = FilePosition::from_span(file_name, span);
    let mut inner = oracle_def.into_inner();
    let sig = handle_oracle_sig(inner.next().unwrap());
    let code = handle_code(inner.next().unwrap(), file_name);

    OracleDef {
        sig,
        code,
        file_pos,
    }
}

pub fn handle_pkg_spec(pkg_spec: Pair<Rule>, pkg_name: &str, file_name: &str) -> Package {
    let mut oracles = vec![];
    let mut state = None;
    let mut params = None;
    let mut types = None;
    let mut imported_oracles = HashMap::new();

    for spec in pkg_spec.into_inner() {
        match spec.as_rule() {
            Rule::types => {
                types = spec.into_inner().next().map(handle_types_list);
            }
            Rule::state => {
                state = spec
                    .into_inner()
                    .next()
                    .map(|state| handle_decl_list(state, file_name));
            }
            Rule::params => {
                params = spec
                    .into_inner()
                    .next()
                    .map(|params| handle_decl_list(params, file_name));
            }
            Rule::import_oracles => {
                for sig_ast in spec.into_inner() {
                    let (sig, file_pos) = handle_oracle_imports(sig_ast, file_name);
                    imported_oracles.insert(sig.name.clone(), (sig, file_pos));
                }
            }
            Rule::oracle_def => {
                oracles.push(handle_oracle_def(spec, file_name));
            }
            _ => unreachable!("unhandled ast node in package: {}", spec),
        }
    }

    Package {
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
    }
}

pub fn handle_pkg(pkg: Pair<Rule>, file_name: &str) -> (String, Package) {
    let mut inner = pkg.into_inner();
    let pkg_name = inner.next().unwrap().as_str();
    let spec = inner.next().unwrap();
    let pkg = handle_pkg_spec(spec, pkg_name, file_name);

    (pkg_name.to_owned(), pkg)
}
