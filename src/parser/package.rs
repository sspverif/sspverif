use crate::expressions::Expression;
use crate::identifier::ComposeLoopVar;
use crate::identifier::Identifier;
use crate::package::OracleDef;
use crate::package::OracleSig;
use crate::package::Package;
use crate::statement::CodeBlock;
use crate::statement::FilePosition;
use crate::statement::Statement;
use crate::types::Type;
use crate::util::resolver::Named;
use crate::util::resolver::Resolver;
use crate::util::resolver::SliceResolver;
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
            let litval = expr.as_str().trim().to_string();

            Expression::IntegerLiteral(litval.parse().expect(&format!(
                "error at position {:?}..{:?}: could not parse as int: {}",
                expr.as_span().start_pos().line_col(),
                expr.as_span().end_pos().line_col(),
                expr.as_str(),
            )))
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
                        let target_ident = Identifier::new_scalar(inner.next().unwrap().as_str());
                        let maybe_index = inner.next().unwrap();
                        let (opt_idx, oracle_inv) = if maybe_index.as_rule() == Rule::table_index {
                            let mut inner_index = maybe_index.into_inner();
                            let index = handle_expression(inner_index.next().unwrap());
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
                                    dst_inst_index = Some(handle_expression(index_expr_ast));
                                }
                                Rule::fn_call_arglist => {
                                    args.extend(ast.into_inner().map(handle_expression))
                                }
                                _ => unreachable!(),
                            }
                        }

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
                                Box::new(Expression::IntegerLiteral(1)),
                            ),
                            "<=" => lower_bound,
                            _ => panic!(),
                        };

                        let upper_bound = match upper_bound_type {
                            "<" => upper_bound,
                            "<=" => Expression::Add(
                                Box::new(upper_bound),
                                Box::new(Expression::IntegerLiteral(1)),
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
                let body_ast = spec.into_inner().next().unwrap();
                handle_import_oracles_body(
                    body_ast,
                    &mut imported_oracles,
                    pkg_name,
                    file_name,
                    &vec![],
                )
                .unwrap();
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
        multi_inst_idx: None,
    }
}

pub fn handle_oracle_imports_oracle_sig(
    oracle_sig: Pair<Rule>,
    for_stack: &Vec<ForSpec>,
) -> OracleSig {
    println!("{:?}", oracle_sig.as_rule());

    let _span = oracle_sig.as_span();

    let mut inner = oracle_sig.into_inner();
    let name = inner.next().unwrap().as_str();

    let (multi_inst_idx, args) = {
        let mut multi_inst_idx = None;
        let mut arglist = vec![];

        while let Some(next) = inner.peek() {
            match next.as_rule() {
                Rule::indices_expr => {
                    let indices = next.into_inner().map(handle_expression).collect();
                    multi_inst_idx = Some(MultiInstanceIndices::new(indices, for_stack.to_owned()));
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

        (multi_inst_idx, arglist)
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
        multi_inst_idx,
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
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
            | ParseImportOraclesError::InvalidStartComparison(_, file_pos)
            | ParseImportOraclesError::InvalidEndComparison(_, file_pos) => file_pos,
        }
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialOrd, Ord)]
pub struct MultiInstanceIndices {
    pub(crate) indices: Vec<Expression>,
    pub(crate) forspecs: Vec<ForSpec>,
}

impl MultiInstanceIndices {
    pub(crate) fn new(indices: Vec<Expression>, forspecs: Vec<ForSpec>) -> Self {
        Self { indices, forspecs }
    }

    pub(crate) fn from_strings(indices: &[String], forspecs: Vec<ForSpec>) -> Self {
        MultiInstanceIndices {
            indices: indices
                .iter()
                .cloned()
                .map(Identifier::Scalar)
                .map(Expression::Identifier)
                .collect(),
            forspecs,
        }
    }
    pub(crate) fn from_strs(indices: &[&str], forspecs: Vec<ForSpec>) -> Self {
        MultiInstanceIndices {
            indices: indices
                .iter()
                .cloned()
                .map(str::to_string)
                .map(Identifier::Scalar)
                .map(Expression::Identifier)
                .collect(),
            forspecs,
        }
    }
}

impl PartialEq for MultiInstanceIndices {
    fn eq(&self, other: &Self) -> bool {
        if self.indices.len() != other.indices.len() {
            return false;
        }

        let left_resolver = SliceResolver(&self.forspecs);
        let right_resolver = SliceResolver(&other.forspecs);

        let left = self.indices.iter();
        let right = other.indices.iter();

        for (i, (left, right)) in left.zip(right).enumerate() {
            match (left, right) {
                (Expression::IntegerLiteral(left), Expression::IntegerLiteral(right)) => {
                    if left != right {
                        return false;
                    }
                }
                (Expression::Identifier(left), Expression::Identifier(right)) => {
                    let left = left_resolver
                        .resolve_value(left.ident_ref())
                        .unwrap_or_else(|| {
                            panic!(
                                "invalid input: index #{i} is not specified in {indices:?}",
                                i = i,
                                indices = self
                            )
                        });

                    let right = right_resolver
                        .resolve_value(right.ident_ref())
                        .unwrap_or_else(|| {
                            panic!(
                                "invalid input: index #{i} is not specified in {indices:?}",
                                i = i,
                                indices = other
                            )
                        });
                    if left.start != right.start
                        || left.end != right.end
                        || left.start_comp != right.end_comp
                        || left.end_comp != right.end_comp
                    {
                        return false;
                    }
                }
                _ => return false,
            }
        }

        true
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
        let resolver = SliceResolver(&self.forspecs);
        //assert!(self.indices.len() == 1);
        for index in &self.indices {
            let does_match: SmtExpr = match index {
                Expression::IntegerLiteral(index) => SmtEq2 {
                    lhs: *index,
                    rhs: varname,
                }
                .into(),
                Expression::Identifier(Identifier::Scalar(var))
                | Expression::Identifier(Identifier::ComposeLoopVar(ComposeLoopVar {
                    name_in_comp: var,
                    ..
                })) => {
                    let forspec = resolver.resolve_value(var).unwrap();
                    SmtAnd(vec![
                        SmtLte(forspec.start.clone(), varname).into(),
                        SmtLt(varname, forspec.end.clone()).into(),
                    ])
                    .into()
                }
                Expression::Identifier(Identifier::Parameter(pkg_const)) => SmtEq2 {
                    lhs: pkg_const.name_in_comp.clone(),
                    rhs: varname,
                }
                .into(),
                _ => unreachable!(),
            };

            return does_match;
        }

        unreachable!()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        expressions::Expression,
        identifier::{GameInstanceConst, Identifier, PackageConst},
    };

    use super::{ForComp, ForSpec, MultiInstanceIndices};

    #[test]
    fn multi_instance_indices_equality() {
        let ident_loop_left = Identifier::Local("left_idx".to_string());
        let ident_loop_right = Identifier::Local("right_idx".to_string());
        let ident_end = Identifier::GameInstanceConst(GameInstanceConst {
            game_inst_name: "the_game_inst".to_string(),
            name_in_comp: "n".to_string(),
            name_in_proof: "n".to_string(),
        });

        let lit_0 = Expression::IntegerLiteral(0);
        let lit_1 = Expression::IntegerLiteral(1);

        let left = MultiInstanceIndices::new(
            vec![Expression::Identifier(ident_loop_left.clone())],
            vec![ForSpec {
                ident: ident_loop_left.clone(),
                start: lit_0.clone(),
                end: Expression::Identifier(ident_end.clone()),
                start_comp: ForComp::Lte,
                end_comp: ForComp::Lte,
            }],
        );

        let right = MultiInstanceIndices::new(
            vec![Expression::Identifier(ident_loop_right.clone())],
            vec![ForSpec {
                ident: ident_loop_right.clone(),
                start: lit_0.clone(),
                end: Expression::Identifier(ident_end.clone()),
                start_comp: ForComp::Lte,
                end_comp: ForComp::Lte,
            }],
        );

        assert_eq!(left, right);

        let left = MultiInstanceIndices::new(
            vec![Expression::Identifier(ident_loop_left.clone())],
            vec![ForSpec {
                ident: ident_loop_left.clone(),
                start: lit_0.clone(),
                end: Expression::Identifier(ident_end.clone()),
                start_comp: ForComp::Lte,
                end_comp: ForComp::Lte,
            }],
        );

        let right = MultiInstanceIndices::new(
            vec![Expression::Identifier(ident_loop_right.clone())],
            vec![ForSpec {
                ident: ident_loop_right.clone(),
                start: lit_1.clone(),
                end: Expression::Identifier(ident_end.clone()),
                start_comp: ForComp::Lte,
                end_comp: ForComp::Lte,
            }],
        );

        assert!(left != right);

        let ident_end_left = Identifier::GameInstanceConst(GameInstanceConst {
            game_inst_name: "the_game_inst".to_string(),
            name_in_comp: "n".to_string(),
            name_in_proof: "n".to_string(),
        });
        let ident_end_right = Identifier::Parameter(PackageConst {
            game_inst_name: "the_game_inst".to_string(),
            name_in_comp: "n".to_string(),
            name_in_proof: "n".to_string(),
            name_in_pkg: "anything".to_string(),
            pkgname: "anything".to_string(),
        });

        let left = MultiInstanceIndices::new(
            vec![Expression::Identifier(ident_loop_left.clone())],
            vec![ForSpec {
                ident: ident_loop_left.clone(),
                start: lit_0.clone(),
                end: Expression::Identifier(ident_end_left.clone()),
                start_comp: ForComp::Lte,
                end_comp: ForComp::Lte,
            }],
        );

        let right = MultiInstanceIndices::new(
            vec![Expression::Identifier(ident_loop_right.clone())],
            vec![ForSpec {
                ident: ident_loop_right.clone(),
                start: lit_0.clone(),
                end: Expression::Identifier(ident_end_right.clone()),
                start_comp: ForComp::Lte,
                end_comp: ForComp::Lte,
            }],
        );

        assert_eq!(left, right);
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
    InvalidStartComparison(ForCompError, FilePosition),
    InvalidEndComparison(ForCompError, FilePosition),
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
        }
    }
}

impl std::error::Error for ParseImportOraclesError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ParseImportOraclesError::IdentifierMismatch(_, _, _) => None,
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
    for_stack: &Vec<ForSpec>,
) -> Result<(), ParseImportOraclesError> {
    assert_eq!(ast.as_rule(), Rule::import_oracles_body);

    for entry in ast.into_inner() {
        match entry.as_rule() {
            Rule::import_oracles_oracle_sig => {
                let file_pos = FilePosition::from_span(file_name, entry.as_span());
                let sig = handle_oracle_imports_oracle_sig(entry, for_stack);
                imported_oracles.insert(sig.name.clone(), (sig, file_pos));
            }

            Rule::import_oracles_for => {
                let mut for_ast = entry.into_inner();

                let ident_ast = for_ast.next().unwrap();
                let ident = ident_ast.as_str().to_string();

                let for_start = handle_expression(for_ast.next().unwrap());

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

                let end = handle_expression(for_ast.next().unwrap());

                if ident != ident2 {
                    return Err(ParseImportOraclesError::IdentifierMismatch(
                        ident,
                        ident2,
                        FilePosition::from_span(file_name, ident2_span),
                    ));
                }

                let for_spec = ForSpec {
                    ident: Identifier::Scalar(ident),
                    start: for_start,
                    end,
                    start_comp,
                    end_comp,
                };

                let mut new_for_stack = for_stack.clone();
                new_for_stack.push(for_spec);

                handle_import_oracles_body(
                    for_ast.next().unwrap(),
                    imported_oracles,
                    pkg_name,
                    file_name,
                    &new_for_stack,
                )?;
            }

            _ => unreachable!(),
        }
    }

    Ok(())
}

pub fn handle_pkg(pkg: Pair<Rule>, file_name: &str) -> (String, Package) {
    let mut inner = pkg.into_inner();
    let pkg_name = inner.next().unwrap().as_str();
    let spec = inner.next().unwrap();
    let pkg = handle_pkg_spec(spec, pkg_name, file_name);

    (pkg_name.to_owned(), pkg)
}

#[cfg(test)]
mod tests2 {
    use pest::Parser;

    use crate::{
        expressions::Expression,
        identifier::Identifier,
        parser::{
            common::handle_expression,
            package::{ForComp, ForSpec},
            Rule, SspParser,
        },
        writers::smt::exprs::{SmtLt, SmtLte},
    };

    use super::{MultiInstanceIndices, MultiInstanceIndicesGroup};

    #[test]
    fn example_smt_stuff() {
        let parse_expr = |text: &str| {
            handle_expression(
                SspParser::parse(Rule::expression, text)
                    .unwrap()
                    .next()
                    .unwrap(),
            )
        };

        let end = parse_expr("(n - 1)").map(|expr| match expr {
            Expression::Identifier(Identifier::Scalar(x)) => {
                Expression::Identifier(Identifier::Local(x))
            }
            x => x,
        });

        let indices_group = MultiInstanceIndicesGroup(vec![MultiInstanceIndices::from_strs(
            &["i"],
            vec![ForSpec {
                ident: Identifier::Scalar("i".to_string()),
                start: parse_expr("0"),
                end,
                start_comp: ForComp::Lte,
                end_comp: ForComp::Lt,
            }],
        )]);

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
