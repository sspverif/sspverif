use super::{
    common::*,
    error,
    package::{ForComp, MultiInstanceIndices},
    ParseContext, Rule,
};
use crate::{
    expressions::Expression,
    identifier::{
        game_ident::{GameConstIdentifier, GameIdentifier},
        pkg_ident::PackageConstIdentifier,
        Identifier,
    },
    package::{
        Composition, Edge, Export, MultiInstanceEdge, MultiInstanceExport,
        NotSingleInstanceEdgeError, NotSingleInstanceExportError, OracleSig, Package,
        PackageInstance,
    },
    statement::FilePosition,
    transforms::{resolvetypes::ResolveTypesPackageInstanceTransform, PackageInstanceTransform},
    types::Type,
};
use pest::iterators::{Pair, Pairs};
use std::collections::HashMap;
use std::convert::TryInto;
use std::iter::FromIterator as _;

impl<'a> ParseContext<'a> {
    fn game_context(
        self,
        game_name: &'a str,
        pkgs: &'a HashMap<String, Package>,
    ) -> ParseGameContext<'a> {
        let Self {
            file_name,
            file_content,
            scope,
        } = self;

        ParseGameContext {
            file_name,
            file_content,
            game_name,
            pkgs,

            scope,

            consts: HashMap::new(),

            instances: vec![],
            instances_table: HashMap::new(),

            edges: vec![],
            exports: vec![],

            multi_inst_edges: vec![],
            multi_inst_exports: vec![],
        }
    }
}

#[derive(Debug)]
pub struct ParseGameContext<'a> {
    pub file_name: &'a str,
    pub file_content: &'a str,
    pub game_name: &'a str,
    pub pkgs: &'a HashMap<String, Package>,

    pub scope: Scope,

    pub consts: HashMap<String, Type>,

    pub instances: Vec<PackageInstance>,
    pub instances_table: HashMap<String, (usize, PackageInstance)>,

    pub edges: Vec<Edge>,
    pub exports: Vec<Export>,

    pub multi_inst_edges: Vec<MultiInstanceEdge>,
    pub multi_inst_exports: Vec<MultiInstanceExport>,
}

impl<'a> ParseGameContext<'a> {
    fn into_game(self) -> Composition {
        let mut consts = Vec::from_iter(self.consts);
        consts.sort();

        Composition {
            name: self.game_name.to_string(),
            consts,
            pkgs: self.instances,
            edges: self.edges,
            exports: self.exports,
            multi_inst_edges: self.multi_inst_edges,
            multi_inst_exports: self.multi_inst_exports,

            // this one will be populated in a transform, not in the parser
            split_exports: vec![],
        }
    }

    fn declare(
        &mut self,
        name: &str,
        declaration: Declaration,
    ) -> Result<(), crate::util::scope::Error> {
        self.scope.declare(name, declaration)
    }

    // TODO: check dupes here?
    fn add_pkg_instance(&mut self, pkg_inst: PackageInstance) {
        let offset = self.instances.len();
        self.instances.push(pkg_inst.clone());
        self.instances_table
            .insert(pkg_inst.name.clone(), (offset, pkg_inst));
    }

    fn has_pkg_instance(&self, name: &str) -> bool {
        self.instances_table.contains_key(name)
    }
    fn get_pkg_instance(&self, name: &str) -> Option<(usize, &PackageInstance)> {
        self.instances_table
            .get(name)
            .map(|(offset, pkg_inst)| (*offset, pkg_inst))
    }

    // TODO: check dupes here?
    fn add_const(&mut self, name: String, ty: Type) {
        self.consts.insert(name, ty);
    }

    fn get_const(&self, name: &str) -> Option<&Type> {
        self.consts.get(name)
    }

    fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge)
    }
    fn add_export(&mut self, export: Export) {
        self.exports.push(export)
    }

    fn add_multi_inst_edge(&mut self, edge: MultiInstanceEdge) {
        self.multi_inst_edges.push(edge)
    }
    fn add_multi_inst_export(&mut self, export: MultiInstanceExport) {
        self.multi_inst_exports.push(export)
    }
}

pub fn handle_composition(
    file_name: &str,
    file_content: &str,
    ast: Pair<Rule>,
    pkg_map: &HashMap<String, Package>,
) -> error::Result<Composition> {
    let mut inner = ast.into_inner();
    let game_name = inner.next().unwrap().as_str();

    let mut scope = Scope::new();
    scope.enter();

    let ctx = ParseContext::new(file_name, file_content).game_context(game_name, pkg_map);

    let spec = inner.next().unwrap();
    handle_comp_spec_list(ctx, spec)
}

/// Parses the main body of a game (aka composition).
/// This function takes ownership of the context because it needs to move all the information stored in there into the game.
pub fn handle_comp_spec_list(
    mut ctx: ParseGameContext,
    ast: Pair<Rule>,
) -> error::Result<Composition> {
    for comp_spec in ast.into_inner() {
        match comp_spec.as_rule() {
            Rule::const_decl => {
                let (name, tipe) = handle_const_decl(comp_spec);
                ctx.add_const(name.clone(), tipe.clone());
                ctx.declare(
                    &name,
                    Declaration::Identifier(
                        GameConstIdentifier {
                            game_name: ctx.game_name.to_string(),
                            name: name.clone(),
                            tipe,
                            inst_info: None,
                        }
                        .into(),
                    ),
                )
                .unwrap();
            }
            Rule::game_for => {
                handle_for_loop(&mut ctx, comp_spec)?;
            }
            Rule::instance_decl => {
                handle_instance_decl(&mut ctx, comp_spec)?;
            }
            Rule::compose_decl => {
                let comp_spec_span = comp_spec.as_span();
                handle_compose_assign_body_list(&mut ctx, comp_spec)
                    .map_err(|e| error::Error::ParseGameError(e).with_span(comp_spec_span))?;
            }
            _ => {
                unreachable!();
            }
        }
    }

    Ok(ctx.into_game())
}

pub fn handle_compose_assign_list_multi_inst<'a>(
    ctx: &mut ParseGameContext<'a>,
    ast: Pairs<Rule>,
) -> Result<Vec<(String, String, Vec<Expression>)>, ParseGameError> {
    ast.map(|assignment| -> Result<_, ParseGameError> {
        let mut line_builder = (None, None, vec![]);
        for piece in assignment.into_inner() {
            match piece.as_rule() {
                Rule::identifier if line_builder.0.is_none() => {
                    line_builder.0 = Some(piece.as_str().to_string())
                }
                Rule::identifier if line_builder.1.is_none() => {
                    if !ctx.has_pkg_instance(piece.as_str()) {
                        return Err(ParseGameError::UndeclaredInstance(
                            piece.as_str().to_string(),
                            FilePosition::from_span(ctx.file_name, piece.as_span()),
                        ));
                    }

                    line_builder.1 = Some(piece.as_str().to_string())
                }
                Rule::compose_assign_modifier_with_index => line_builder.2.append(
                    &mut piece
                        .into_inner()
                        .next()
                        .unwrap()
                        .into_inner()
                        .map(|e| {
                            handle_expression(e, &mut ctx.scope)
                                .map_err(ParseGameError::ParseExpressionError)
                        })
                        .collect::<Result<Vec<Expression>, ParseGameError>>()?,
                ),
                _ => unreachable!(),
            }
        }

        Ok((
            line_builder.0.unwrap(),
            line_builder.1.unwrap(),
            line_builder.2,
        ))
        // let mut inner = assignment.into_inner();
        // let oracle_name = inner.next().unwrap().as_str();
        // let dst_inst_name = inner.next().unwrap().as_str();
        //
        // (oracle_name.to_owned(), dst_inst_name.to_owned())
    })
    .collect::<Result<Vec<_>, _>>()
}

#[derive(Debug)]
pub enum ParseGameError {
    UndeclaredInstance(String, FilePosition),
    NoSuchOracleInInstance(String, String, FilePosition),
    ParseExpressionError(super::common::ParseExpressionError),
}

impl std::fmt::Display for ParseGameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseGameError::UndeclaredInstance(inst_name, _) => {
                write!(f, "instance {inst_name:?} not declared")
            }
            ParseGameError::NoSuchOracleInInstance(oracle_name, inst_name, _) => {
                write!(
                    f,
                    "instace {inst_name:?} does not have oracle {oracle_name:?}"
                )
            }
            ParseGameError::ParseExpressionError(err) => {
                write!(f, "error parsing expression: {err}")
            }
        }
    }
}

impl std::error::Error for ParseGameError {}
impl crate::error::LocationError for ParseGameError {
    fn file_pos<'a>(&'a self) -> &'a FilePosition {
        match self {
            ParseGameError::ParseExpressionError(err) => err.file_pos(),
            ParseGameError::UndeclaredInstance(_, file_pos) => file_pos,
            ParseGameError::NoSuchOracleInInstance(_, _, file_pos) => file_pos,
        }
    }
}

pub fn handle_for_loop_body<'a>(
    ctx: &mut ParseGameContext<'a>,
    ast: Pair<Rule>,
) -> error::Result<()> {
    for comp_spec in ast.into_inner() {
        match comp_spec.as_rule() {
            Rule::game_for => handle_for_loop(ctx, comp_spec)?,

            Rule::instance_decl => handle_instance_decl_multi_inst(ctx, comp_spec)?,

            Rule::compose_decl_multi_inst => {
                let comp_spec_span = comp_spec.as_span();
                handle_compose_assign_body_list_multi_inst(ctx, comp_spec)
                    .map_err(|e| error::Error::ParseGameError(e).with_span(comp_spec_span))?;
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}

/*
This functions parses the body of a compose block. It returns internal edges and exports.
 */
pub fn handle_compose_assign_body_list(
    ctx: &mut ParseGameContext,
    ast: Pair<Rule>,
) -> Result<(), ParseGameError> {
    handle_compose_assign_body_list_multi_inst(ctx, ast)
}

pub fn handle_compose_assign_body_list_multi_inst(
    ctx: &mut ParseGameContext,
    ast: Pair<Rule>,
) -> Result<(), ParseGameError> {
    for body in ast.into_inner() {
        let mut inner = body.into_inner();
        let src_inst_name = inner.next().unwrap().as_str();

        let maybe_src_indices = inner.peek().unwrap();
        let source_instance_idx = if matches!(maybe_src_indices.as_rule(), Rule::indices_ident) {
            inner.next();
            maybe_src_indices
                .into_inner()
                .map(
                    // TODO: Error handling
                    |idx_ident| match ctx.scope.lookup(idx_ident.as_str()).unwrap() {
                        Declaration::Oracle(_, _) => panic!("expected an identifier"),
                        Declaration::Identifier(ident) => ident,
                    },
                )
                .collect()
        } else {
            vec![]
        };

        match src_inst_name {
            "adversary" => {
                for multi_inst_export in handle_export_compose_assign_list_multi_inst(ctx, inner)? {
                    match multi_inst_export.try_into() {
                        Ok(export) => ctx.add_export(export),
                        Err(NotSingleInstanceExportError(multi_export)) => {
                            ctx.add_multi_inst_export(multi_export)
                        }
                    }
                }
            }
            _ => {
                let source_pkgidx = ctx
                    .instances_table
                    .get(src_inst_name)
                    .unwrap_or_else(|| {
                        panic!(
                            "instance {} not found in composition {}",
                            src_inst_name, ctx.file_name
                        )
                    })
                    .0;

                for multi_instance_edge in handle_edges_compose_assign_list_multi_inst(
                    ctx,
                    inner,
                    &source_instance_idx,
                    source_pkgidx,
                )? {
                    match multi_instance_edge.try_into() {
                        Ok(edge) => ctx.add_edge(edge),
                        Err(NotSingleInstanceEdgeError(multi_edge)) => {
                            ctx.add_multi_inst_edge(multi_edge)
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn handle_export_compose_assign_list_multi_inst(
    ctx: &mut ParseGameContext,
    ast: Pairs<Rule>,
) -> Result<Vec<MultiInstanceExport>, ParseGameError> {
    let mut exports = vec![];

    for assignment in ast {
        let mut oracle_name = None;
        let mut dst_inst_name = None;
        let mut dst_instance_idx = vec![];
        let span = assignment.as_span();
        for piece in assignment.into_inner() {
            match piece.as_rule() {
                Rule::identifier if oracle_name.is_none() => {
                    oracle_name = Some(piece.as_str());
                }
                Rule::identifier if dst_inst_name.is_none() => {
                    if !ctx.has_pkg_instance(piece.as_str()) {
                        return Err(ParseGameError::UndeclaredInstance(
                            piece.as_str().to_string(),
                            FilePosition::from_span(ctx.file_name, piece.as_span()),
                        ));
                    }

                    dst_inst_name = Some(piece.as_str());
                }
                Rule::compose_assign_modifier_with_index => dst_instance_idx.append(
                    &mut piece
                        .into_inner()
                        .next()
                        .unwrap()
                        .into_inner()
                        .map(|e| {
                            handle_expression(e, &mut ctx.scope)
                                .map_err(ParseGameError::ParseExpressionError)
                        })
                        .collect::<Result<_, _>>()?,
                ),
                _ => unreachable!(""),
            }
        }

        let oracle_name = oracle_name.expect("expected there to be an oracle name");
        let dst_inst_name = dst_inst_name.expect("expected there to be a dst instance name");

        let (dst_offset, dst_inst) = match ctx.get_pkg_instance(dst_inst_name) {
            None => {
                let pos = FilePosition::new(
                    ctx.file_name.to_string(),
                    span.start_pos().line_col().0,
                    span.end_pos().line_col().0,
                );
                panic!(
                    "instance {} not declared but used at {}",
                    dst_inst_name, pos
                );
            }
            Some(inst) => inst,
        };
        let mut oracle_sig = match dst_inst
            .pkg
            .oracles
            .iter()
            .find(|oracle_def| oracle_def.sig.name == oracle_name)
        {
            None => {
                panic!(
                    "oracle {oracle_name} not found in package instance {dst_inst_name} in composition {file_name}",
                    file_name = ctx.file_name
                );
            }
            Some(def) => def.sig.clone(),
        };

        assert!(oracle_sig.multi_inst_idx.indices.is_empty());

        if !dst_instance_idx.is_empty() {
            oracle_sig.multi_inst_idx = MultiInstanceIndices {
                indices: dst_instance_idx,
            };
        }

        exports.push(MultiInstanceExport {
            dest_pkgidx: dst_offset,
            oracle_sig,
        });
    }

    Ok(exports)
}

fn handle_edges_compose_assign_list_multi_inst(
    ctx: &mut ParseGameContext,
    ast: Pairs<Rule>,
    source_instance_idx: &[Identifier],
    source_pkgidx: usize,
) -> Result<Vec<MultiInstanceEdge>, ParseGameError> {
    let mut edges = vec![];

    for assignment in ast {
        let mut oracle_name = None;
        let mut dst_inst_name = None;
        let mut dst_instance_idx = vec![];
        let span = assignment.as_span();
        for piece in assignment.into_inner() {
            match piece.as_rule() {
                Rule::identifier if oracle_name.is_none() => {
                    oracle_name = Some(piece.as_str());
                }
                Rule::identifier if dst_inst_name.is_none() => {
                    if !ctx.has_pkg_instance(piece.as_str()) {
                        return Err(ParseGameError::UndeclaredInstance(
                            piece.as_str().to_string(),
                            FilePosition::from_span(ctx.file_name, piece.as_span()),
                        ));
                    }

                    dst_inst_name = Some(piece.as_str());
                }
                Rule::compose_assign_modifier_with_index => dst_instance_idx.append(
                    &mut piece
                        .into_inner()
                        .next()
                        .unwrap()
                        .into_inner()
                        .map(|e| {
                            handle_expression(e, &mut ctx.scope)
                                .map_err(ParseGameError::ParseExpressionError)
                        })
                        .collect::<Result<_, _>>()?,
                ),
                other_rule => unreachable!("found rule {rule:?}", rule = other_rule),
            }
        }

        let oracle_name = oracle_name.expect("expected there to be an oracle name");
        let dst_inst_name = dst_inst_name.expect("expected there to be a dst instance name");

        println!("mi found oracle {oracle_name}");

        let Some((dst_offset, dst_inst)) = ctx.get_pkg_instance(dst_inst_name) else {
            let pos = FilePosition::new(
                ctx.file_name.to_string(),
                span.start_pos().line_col().0,
                span.end_pos().line_col().0,
            );
            panic!(
                "instance {} not declared but used at {}",
                dst_inst_name, pos
            )
        };

        let oracle_sig = match dst_inst
            .pkg
            .oracles
            .iter()
            .find(|oracle_def| oracle_def.sig.name == oracle_name)
        {
            None => {
                panic!(
                    "oracle {oracle_name} not found in package instance {dst_inst_name}, file name {file_name}",
                    file_name = ctx.file_name
                );
            }
            Some(def) => def.sig.clone(),
        };

        let multi_inst_idx = MultiInstanceIndices {
            indices: dst_instance_idx,
        };

        let oracle_sig = OracleSig {
            multi_inst_idx,
            ..oracle_sig
        };

        edges.push(MultiInstanceEdge {
            dest_pkgidx: dst_offset,
            oracle_sig,
            source_pkgidx,
            source_instance_idx: source_instance_idx.to_vec(),
        });
    }

    Ok(edges)
}

pub fn handle_for_loop<'a>(ctx: &mut ParseGameContext<'a>, ast: Pair<Rule>) -> error::Result<()> {
    let mut parsed: Vec<Pair<Rule>> = ast.into_inner().collect();
    let decl_var_name = parsed[0].as_str();
    let lower_bound = handle_expression(parsed.remove(1), &mut ctx.scope)?;
    let lower_bound_type = parsed[1].as_str();
    let bound_var_name = parsed[2].as_str();
    let upper_bound_type = parsed[3].as_str();
    let upper_bound = handle_expression(parsed.remove(4), &mut ctx.scope)?;
    let body_ast = parsed.remove(4);

    if decl_var_name != bound_var_name {
        todo!("return proper error here")
    }

    let start_comp = match lower_bound_type {
        "<" => ForComp::Lt,
        "<=" => ForComp::Lte,
        _ => unreachable!(),
    };
    let end_comp = match upper_bound_type {
        "<" => ForComp::Lt,
        "<=" => ForComp::Lte,
        _ => unreachable!(),
    };

    let loopvar = crate::identifier::game_ident::GameLoopVarIdentifier {
        game_name: ctx.game_name.to_string(),
        name: decl_var_name.to_string(),
        start: Box::new(lower_bound),
        end: Box::new(upper_bound),
        start_comp,
        end_comp,
        game_inst_name: None,
        proof_name: None,
        inst_info: None,
    };
    let loopvar = GameIdentifier::LoopVar(loopvar);
    let loopvar = Identifier::GameIdentifier(loopvar);
    let decl = Declaration::Identifier(loopvar);

    ctx.scope.enter();
    ctx.declare(decl_var_name, decl).unwrap();

    let result = handle_for_loop_body(ctx, body_ast);

    ctx.scope.leave();

    result
}

use crate::util::scope::{Declaration, Scope};

pub fn handle_instance_assign_list(
    ctx: &mut ParseGameContext,
    ast: Pair<Rule>,
    inst_name: &str,
    pkg_name: &str,
) -> error::Result<(
    Vec<(PackageConstIdentifier, Expression)>,
    Vec<(String, Type)>,
)> {
    let mut params = vec![];
    let mut types = vec![];

    for elem in ast.into_inner() {
        match elem.as_rule() {
            Rule::params_def => {
                let defs =
                    handle_game_params_def_list(elem.into_inner().next().unwrap(), &mut ctx.scope)?;
                params.extend(defs.into_iter().map(|(name, value)| {
                    (
                        PackageConstIdentifier {
                            pkg_name: pkg_name.to_string(),
                            name,
                            tipe: value.get_type(),
                            // we don't resolve it here yet, so we can easily find it when
                            // searching this list when we don't have the value yet.
                            game_name: None,
                            game_assignment: None,
                            pkg_inst_name: None,
                            game_inst_name: None,
                            proof_name: None,
                        },
                        value,
                    )
                }))
            }
            Rule::types_def => {
                let mut defs = handle_types_def_list(
                    elem.into_inner().next().unwrap(),
                    inst_name,
                    ctx.file_name,
                )?;
                types.append(&mut defs);
            }
            _ => unreachable!("{:#?}", elem),
        }
    }

    Ok((params, types))
}

pub fn handle_instance_decl_multi_inst(
    ctx: &mut ParseGameContext,
    ast: Pair<Rule>,
) -> error::Result<()> {
    let span = ast.as_span();

    println!(">>>> {:#?}:{:?}", ast, ast.as_rule());

    let mut inner = ast.into_inner();
    let inst_name = inner.next().unwrap().as_str();

    println!("inst_name: {:?}", inst_name);

    let indices_ast = inner.next().unwrap();
    println!("indices: {:?}", indices_ast);
    let indices = indices_ast
        .into_inner()
        .map(|index_ast| handle_expression(index_ast, &mut ctx.scope))
        .collect::<Result<Vec<_>, _>>()?;

    println!("indices: {:?}", indices);

    let pkg_name = inner.next().unwrap().as_str();
    let data = inner.next().unwrap();

    let pkg = match ctx.pkgs.get(pkg_name) {
        None => {
            panic!("package {} is unknown", pkg_name);
        }
        Some(pkg) => error::Result::Ok(pkg),
    }?;

    let (mut param_list, type_list) = handle_instance_assign_list(ctx, data, inst_name, pkg_name)?;

    param_list.sort();

    // check that const param lists match
    let mut typed_params: Vec<_> = param_list
        .iter()
        .map(|(pkg_param, comp_param)| (pkg_param.name.clone(), comp_param.get_type()))
        .collect();
    typed_params.sort();

    let mut pkg_params: Vec<_> = pkg
        .params
        .iter()
        .cloned()
        .map(|(name, tipe, _)| (name, tipe))
        .collect();
    pkg_params.sort();

    if typed_params != pkg_params {
        // TODO: include the difference in here
        return Err(error::Error::PackageConstParameterMismatch {
            pkg_name: pkg_name.to_string(),
            inst_name: inst_name.to_string(),
            bound_params: typed_params,
            pkg_params,
        }
        .with_span(span));
    }

    // check that type param lists match
    let mut assigned_types: Vec<_> = type_list
        .iter()
        .map(|(pkg_type, _)| pkg_type)
        .cloned()
        .collect();
    assigned_types.sort();

    let mut pkg_types: Vec<_> = pkg.types.iter().map(|(ty, _span)| ty.clone()).collect();
    pkg_types.sort();

    if assigned_types != pkg_types {
        println!("assigned: {assigned_types:?}");
        println!("pkg:      {pkg_types:?}");
        // TODO include the difference in here
        return Err(error::SpanError::new_with_span(
            error::Error::TypeParameterMismatch {
                game_name: ctx.game_name.to_string(),
                pkg_name: pkg_name.to_string(),
                pkg_inst_name: inst_name.to_string(),
            },
            span,
        ));
    }

    let multi_instance_indices = MultiInstanceIndices::new(indices);

    let inst = PackageInstance {
        name: inst_name.to_owned(),
        params: param_list,
        types: type_list,
        pkg: pkg.clone(),
        multi_instance_indices,
    };

    let resolved_inst = match ResolveTypesPackageInstanceTransform.transform_package_instance(&inst)
    {
        Ok((inst, _)) => Ok(inst),
        Err(err) => Err(error::Error::from(err).with_span(span)),
    }?;

    ctx.add_pkg_instance(resolved_inst);

    Ok(())
}

pub fn handle_instance_decl(ctx: &mut ParseGameContext, ast: Pair<Rule>) -> error::Result<()> {
    let span = ast.as_span();

    let mut inner = ast.into_inner();
    let pkg_inst_name = inner.next().unwrap().as_str();

    let index_or_pkgname = inner.next().unwrap();
    let (multi_instance_indices, pkg_name) = if index_or_pkgname.as_rule() == Rule::index_id_list {
        let indices_ast = index_or_pkgname.into_inner();
        let indices: Vec<_> = indices_ast
            .map(|index| handle_expression(index, &mut ctx.scope))
            .collect::<Result<_, _>>()?;
        (
            MultiInstanceIndices::new(indices),
            inner.next().unwrap().as_str(),
        )
    } else {
        (MultiInstanceIndices::empty(), index_or_pkgname.as_str())
    };

    let pkg = match ctx.pkgs.get(pkg_name) {
        None => {
            panic!(
                "package {pkg_name} is unknown in composition {file_name}",
                file_name = ctx.file_name
            );
        }
        Some(pkg) => error::Result::Ok(pkg),
    }?;

    let instance_assign_ast = inner.next().unwrap();
    let (mut param_list, type_list) =
        handle_instance_assign_list(ctx, instance_assign_ast, pkg_inst_name, pkg_name)?;

    param_list.sort();

    // check that const param lists match
    let mut typed_params: Vec<_> = param_list
        .iter()
        .map(|(pkg_param, comp_param)| match comp_param {
            Expression::Identifier(id) => {
                let maybe_type = ctx.get_const(id.ident_ref());

                assert!(
                    maybe_type.is_some(),
                    "constant not specified: {} at {:?}",
                    id.ident(),
                    span
                );
                (pkg_param.ident(), maybe_type.unwrap().clone())
            }
            Expression::BooleanLiteral(_) => (pkg_param.ident(), Type::Boolean),
            Expression::IntegerLiteral(_) => (pkg_param.ident(), Type::Integer),
            otherwise => {
                panic!("unhandled expression: {:?}", otherwise)
            }
        })
        .collect();
    typed_params.sort();

    let mut pkg_params: Vec<_> = pkg
        .params
        .iter()
        .cloned()
        .map(|(name, tipe, _)| (name, tipe))
        .collect();
    pkg_params.sort();

    if typed_params != pkg_params {
        // TODO: include the difference in here
        return Err(error::Error::PackageConstParameterMismatch {
            pkg_name: pkg_name.to_string(),
            inst_name: pkg_inst_name.to_string(),
            bound_params: typed_params,
            pkg_params,
        }
        .with_span(span));
    }

    // check that type param lists match
    let mut assigned_types: Vec<_> = type_list
        .iter()
        .map(|(pkg_type, _)| pkg_type)
        .cloned()
        .collect();
    assigned_types.sort();

    let mut pkg_types: Vec<_> = pkg.types.iter().map(|(ty, _span)| ty.clone()).collect();
    pkg_types.sort();

    if assigned_types != pkg_types {
        println!("assigned: {assigned_types:?}");
        println!("pkg:      {pkg_types:?}");
        // TODO include the difference in here
        return Err(error::SpanError::new_with_span(
            error::Error::TypeParameterMismatch {
                game_name: ctx.game_name.to_string(),
                pkg_name: pkg_name.to_string(),
                pkg_inst_name: pkg_inst_name.to_string(),
            },
            span,
        ));
    }

    let pkg_inst = PackageInstance::new(
        pkg_inst_name,
        ctx.game_name,
        pkg,
        multi_instance_indices,
        param_list,
        type_list,
    );

    match ResolveTypesPackageInstanceTransform.transform_package_instance(&pkg_inst) {
        Ok((pkg_inst, _)) => {
            ctx.add_pkg_instance(pkg_inst);
            Ok(())
        }
        Err(err) => Err(error::Error::from(err).with_span(span)),
    }
}

pub fn handle_compose_assign_list(ast: Pairs<Rule>) -> Vec<(String, String)> {
    ast.map(|assignment| {
        let mut inner = assignment.into_inner();

        let oracle_name = inner.next().unwrap().as_str();
        let dst_inst_name = inner.next().unwrap().as_str();

        (oracle_name.to_owned(), dst_inst_name.to_owned())
    })
    .collect()
}

pub fn handle_index_id_list<'a>(ast: Pair<'a, Rule>) -> Vec<String> {
    assert_eq!(ast.as_rule(), Rule::index_id_list);
    ast.into_inner()
        .map(|ast| ast.as_str().to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        identifier::game_ident::{GameConstIdentifier, GameIdentifier},
        parser::{package::handle_pkg, SspParser},
    };

    fn unwrap_parse_err<T>(res: Result<T, pest::error::Error<crate::parser::Rule>>) -> T {
        match res {
            Ok(v) => v,
            Err(err) => panic!("parse error: {err}", err = err),
        }
    }

    fn parse_game(code: &str, name: &str, pkg_map: &HashMap<String, Package>) -> Composition {
        let mut game_pairs = unwrap_parse_err(SspParser::parse_composition(code));
        handle_composition(name, code, game_pairs.next().unwrap(), pkg_map)
            .unwrap_or_else(|err| panic!("handle error: {err}", err = err))
    }

    fn parse_pkg(code: &str, name: &str) -> (String, Package) {
        let mut pkg_pairs = unwrap_parse_err(SspParser::parse_package(code));
        handle_pkg(name, code, pkg_pairs.next().unwrap()).unwrap()
    }

    const TINY_PKG_CODE: &str = r#"package TinyPkg {
            params {
              n: Integer,
            }

            oracle N() -> Integer {
              return n;
            }
        }"#;
    const SMALL_FOR_PKG_CODE: &str = r#"package SmallForPkg {
       params {
          n: Integer,
       }

       import oracles {
         for i: 1 <= i <= n {
           N[i]() -> Integer,
         }
       }

       oracle Sum() -> Integer {
         sum <- 0;

         for i: 1 <= i <= n {
           n_i <- invoke N[i]();
           sum <- (sum + n_i);
         }

         return sum;
       }
    }"#;

    const TINY_GAME_CODE: &str = r#"composition TinyGame {
                const n: Integer;
            }
            "#;

    const SMALL_GAME_CODE: &str = r#"composition SmallGame {
        const n: Integer;

        instance tiny_instance  = TinyPkg {
            params {
                n: n,
            }
        }
    }"#;

    const SMALL_MULTI_INST_GAME_CODE: &str = r#"composition SmallMultiInstGame {
        for i: 0 <= i < 200 {
            instance tiny_instance[i] = TinyPkg {
                params {
                    n:  i
                }
            }
        }
    }"#;

    #[test]
    fn tiny_game_without_packages() {
        let game = parse_game(TINY_GAME_CODE, "tiny-game", &HashMap::default());

        assert_eq!(game.name, "TinyGame");
        assert_eq!(game.consts[0].0, "n");
        assert_eq!(game.consts[0].1, Type::Integer);
        assert_eq!(game.consts.len(), 1);
        assert!(game.pkgs.is_empty());
    }

    #[test]
    fn tiny_package() {
        let (name, pkg) = parse_pkg(TINY_PKG_CODE, "tiny-pkg");

        assert_eq!(name, "TinyPkg");
        assert_eq!(pkg.params.len(), 1);
        assert_eq!(pkg.params[0].0, "n");
        assert_eq!(pkg.params[0].1, Type::Integer);
        assert_eq!(pkg.oracles.len(), 1);
        assert_eq!(pkg.oracles[0].sig.name, "N");
        assert_eq!(pkg.oracles[0].sig.tipe, Type::Integer);
        assert!(pkg.oracles[0].sig.args.is_empty());
        assert!(pkg.imports.is_empty());
    }

    #[test]
    fn small_game() {
        let (name, pkg) = parse_pkg(TINY_PKG_CODE, "tiny-pkg");
        let pkg_map = HashMap::from_iter(vec![(name, pkg.clone())]);
        let game = parse_game(SMALL_GAME_CODE, "small-game", &pkg_map);

        assert_eq!(game.name, "SmallGame");
        assert_eq!(game.consts.len(), 1);
        assert_eq!(game.consts[0].0, "n");
        assert_eq!(game.consts[0].1, Type::Integer);
        assert_eq!(game.pkgs.len(), 1);
        assert_eq!(game.pkgs[0].name, "tiny_instance");
        assert_eq!(game.pkgs[0].params.len(), 1);
        assert_eq!(game.pkgs[0].params[0].0.ident_ref(), "n");
        assert_eq!(
            game.pkgs[0].params[0].1,
            Expression::Identifier(Identifier::GameIdentifier(GameIdentifier::Const(
                GameConstIdentifier {
                    name: "n".to_string(),
                    tipe: Type::Integer,
                    game_name: "SmallGame".to_string(),
                    inst_info: None,
                }
            )))
        );
    }

    #[test]
    fn small_for_package() {
        let (name, pkg) = parse_pkg(SMALL_FOR_PKG_CODE, "small-for-pkg");

        assert_eq!(name, "SmallForPkg");
        assert_eq!(pkg.params.len(), 1);
        assert_eq!(pkg.params[0].0, "n");
        assert_eq!(pkg.params[0].1, Type::Integer);
        assert_eq!(pkg.oracles.len(), 1);
        assert_eq!(pkg.oracles[0].sig.name, "Sum");
        assert_eq!(pkg.oracles[0].sig.tipe, Type::Integer);
        assert!(pkg.oracles[0].sig.args.is_empty());
    }

    #[test]
    fn small_multi_inst_game() {
        let (name, pkg) = parse_pkg(TINY_PKG_CODE, "tiny-pkg");
        let pkg_map = HashMap::from_iter(vec![(name, pkg.clone())]);
        let game = parse_game(
            SMALL_MULTI_INST_GAME_CODE,
            "small-multi-inst-game",
            &pkg_map,
        );
        println!("{game:#?}");

        assert_eq!(game.pkgs[0].multi_instance_indices.indices.len(), 1);
    }
}
