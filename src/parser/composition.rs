use super::package::{ForComp, ForSpec};
use super::{common::*, error, Rule};

use pest::iterators::{Pair, Pairs};
use std::collections::HashMap;
use std::iter::FromIterator;

use crate::expressions::Expression;
use crate::identifier::Identifier;
use crate::package::{
    Composition, Edge, Export, MultiInstanceEdge, MultiInstanceExport, Package, PackageInstance,
};
use crate::statement::FilePosition;
use crate::transforms::resolvetypes::ResolveTypesPackageInstanceTransform;
use crate::transforms::PackageInstanceTransform;

use crate::types::Type;
use crate::util::resolver::Named;

pub fn handle_compose_assign_list(ast: Pairs<Rule>) -> Vec<(String, String)> {
    ast.map(|assignment| {
        let mut inner = assignment.into_inner();

        let oracle_name = inner.next().unwrap().as_str();
        let dst_inst_name = inner.next().unwrap().as_str();

        (oracle_name.to_owned(), dst_inst_name.to_owned())
    })
    .collect()
}

pub fn handle_compose_assign_list_multi_inst(
    ast: Pairs<Rule>,
    instances: &HashMap<String, (usize, PackageInstance)>,
    file_name: &str,
) -> Result<Vec<(String, String, Vec<Expression>)>, ParseGameError> {
    ast.map(|assignment| {
        let mut line_builder = (None, None, vec![]);
        for piece in assignment.into_inner() {
            match piece.as_rule() {
                Rule::identifier if line_builder.0.is_none() => {
                    line_builder.0 = Some(piece.as_str().to_string())
                }
                Rule::identifier if line_builder.1.is_none() => {
                    if !instances.contains_key(piece.as_str()) {
                        return Err(ParseGameError::UndeclaredInstance(
                            piece.as_str().to_string(),
                            FilePosition::from_span(file_name, piece.as_span()),
                        ));
                    }

                    line_builder.1 = Some(piece.as_str().to_string())
                }
                Rule::compose_assign_modifier_with_index => line_builder.2.extend(
                    piece
                        .into_inner()
                        .next()
                        .unwrap()
                        .into_inner()
                        .map(handle_expression),
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
    .collect()
}

pub fn handle_types_def(
    ast: Pair<Rule>,
    inst_name: &str,
    file_name: &str,
) -> error::Result<Vec<(Type, Type)>> {
    match ast.into_inner().next() {
        None => Ok(vec![]),
        Some(ast) => handle_types_def_list(ast, inst_name, file_name),
    }
}

#[derive(Debug, Clone)]
pub enum ParseGameError {
    UndeclaredInstance(String, FilePosition),
    NoSuchOracleInInstance(String, String, FilePosition),
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
        }
    }
}

impl std::error::Error for ParseGameError {}
impl crate::error::LocationError for ParseGameError {
    fn file_pos<'a>(&'a self) -> &'a FilePosition {
        match self {
            ParseGameError::UndeclaredInstance(_, file_pos)
            | ParseGameError::NoSuchOracleInInstance(_, _, file_pos) => file_pos,
        }
    }
}

/*
This functions parses the body of a compose block. It returns internal edges and exports.
 */
pub fn handle_compose_assign_body_list(
    ast: Pair<Rule>,
    instances: &HashMap<String, (usize, PackageInstance)>,
    file_name: &str,
) -> Result<(Vec<Edge>, Vec<Export>), ParseGameError> {
    let mut edges = vec![];
    let mut exports = vec![];
    for body in ast.into_inner() {
        let mut inner = body.into_inner();
        let inst_name = inner.next().unwrap().as_str();
        if inst_name == "adversary" {
            for (oracle_name, dst_inst_name, _) in
                handle_compose_assign_list_multi_inst(inner, instances, file_name)?
            {
                let (dst_offset, dst_inst) = match instances.get(&dst_inst_name) {
                    None => {
                        panic!(
                            "instance {} not declared but used in compose block for {}",
                            dst_inst_name, inst_name
                        );
                    }
                    Some(inst) => inst,
                };
                let oracle_sig = match dst_inst
                    .pkg
                    .oracles
                    .iter()
                    .find(|oracle_def| oracle_def.sig.name == oracle_name)
                {
                    None => {
                        panic!(
                            "oracle {} not found in package instance {}",
                            oracle_name, dst_inst_name
                        );
                    }
                    Some(def) => def.sig.clone(),
                };
                exports.push(Export(*dst_offset, oracle_sig));
            }

            continue;
        }

        let (offset, _inst) = match instances.get(inst_name) {
            None => {
                panic!("instance {} not found in compose block", inst_name);
            }
            Some(x) => x,
        };

        for (oracle_name, dst_inst_name, _) in
            handle_compose_assign_list_multi_inst(inner, instances, file_name)?
        {
            println!("oname: {oracle_name} dst_inst_name: {dst_inst_name}");
            let (dst_offset, dst_inst) = match instances.get(&dst_inst_name) {
                None => {
                    panic!(
                        "instance {} not declared but used in compose block for {}",
                        dst_inst_name, inst_name
                    );
                }
                Some(inst) => inst,
            };

            let oracle_sig = match dst_inst
                .pkg
                .oracles
                .iter()
                .find(|oracle_def| oracle_def.sig.name == oracle_name)
            {
                None => {
                    panic!(
                        "oracle {} not found in package instance {}",
                        oracle_name, dst_inst_name
                    );
                }
                Some(def) => def.sig.clone(),
            };

            edges.push(Edge(*offset, *dst_offset, oracle_sig));
        }
    }

    Ok((edges, exports))
}

pub fn handle_for_loop_body(
    ast: Pair<Rule>,
    game_name: &str,
    loopvars: &mut Vec<ForSpec>,
    pkgs: &HashMap<String, Package>,
    pkg_insts: &HashMap<String, (usize, PackageInstance)>,
    consts: &HashMap<String, Type>,
    file_name: &str,
) -> error::Result<(
    Vec<PackageInstance>,
    Vec<MultiInstanceEdge>,
    Vec<MultiInstanceExport>,
)> {
    let mut instances = vec![];
    let mut multi_edges = vec![];
    let mut multi_exports = vec![];

    for comp_spec in ast.into_inner() {
        match comp_spec.as_rule() {
            Rule::game_for => {
                let (mut mult_pkg_insts, mut new_multi_edges, mut new_multi_exports) =
                    handle_for_loop(
                        comp_spec, game_name, loopvars, pkgs, pkg_insts, &consts, file_name,
                    )?;

                instances.append(&mut mult_pkg_insts);
                multi_edges.append(&mut new_multi_edges);
                multi_exports.append(&mut new_multi_exports);
            }
            Rule::instance_decl => {
                let inst = handle_instance_decl_multi_inst(
                    comp_spec, game_name, pkgs, &consts, loopvars, file_name,
                )?;
                instances.push(inst);
            }
            Rule::compose_decl_multi_inst => {
                let comp_spec_span = comp_spec.as_span();
                let (mut edges_, mut exports_) = handle_compose_assign_body_list_multi_inst(
                    comp_spec, &pkg_insts, loopvars, file_name,
                )
                .map_err(|e| error::Error::ParseGameError(e).with_span(comp_spec_span))?;
                multi_edges.append(&mut edges_);
                multi_exports.append(&mut exports_);
            }
            _ => unreachable!(),
        }
    }

    Ok((instances, multi_edges, multi_exports))
}

pub fn handle_compose_assign_body_list_multi_inst(
    ast: Pair<Rule>,
    instances: &HashMap<String, (usize, PackageInstance)>,
    loopvars: &Vec<ForSpec>,
    file_name: &str,
) -> Result<(Vec<MultiInstanceEdge>, Vec<MultiInstanceExport>), ParseGameError> {
    let mut edges = vec![];
    let mut exports = vec![];
    for body in ast.into_inner() {
        println!(
            "handle compose assign body list multi inst {:?}",
            body.as_rule()
        );
        assert!(matches!(
            body.as_rule(),
            Rule::compose_assign_body_multi_inst
        ));

        let mut inner = body.into_inner();
        let inst_name = inner.next().unwrap().as_str();

        let maybe_src_indices = inner.peek().unwrap();
        let source_instance_idx = if matches!(maybe_src_indices.as_rule(), Rule::indices_ident) {
            inner.next();
            maybe_src_indices
                .into_inner()
                .map(|idx_ident| Identifier::Scalar(idx_ident.as_str().to_string()))
                .collect()
        } else {
            vec![]
        };

        match inst_name {
            "adversary" => {
                for (oracle_name, dst_inst_name, dest_instance_idx) in
                    handle_compose_assign_list_multi_inst(inner, instances, file_name)?
                {
                    let (dst_offset, dst_inst) = match instances.get(&dst_inst_name) {
                        None => {
                            panic!(
                                "instance {} not declared but used in compose block for {}",
                                dst_inst_name, inst_name
                            );
                        }
                        Some(inst) => inst,
                    };
                    let oracle_sig = match dst_inst
                        .pkg
                        .oracles
                        .iter()
                        .find(|oracle_def| oracle_def.sig.name == oracle_name)
                    {
                        None => {
                            panic!(
                                "oracle {} not found in package instance {}",
                                oracle_name, dst_inst_name
                            );
                        }
                        Some(def) => def.sig.clone(),
                    };
                    exports.push(MultiInstanceExport {
                        loopvars: loopvars.clone(),
                        dest_pkgidx: *dst_offset,
                        oracle_sig,
                        dest_instance_idx,
                    });
                }
            }
            _ => {
                let (src_offset, _src_inst) = match instances.get(inst_name) {
                    None => {
                        panic!("instance {} not found in compose block", inst_name);
                    }
                    Some(x) => x,
                };

                for (oracle_name, dst_inst_name, dest_instance_idx) in
                    handle_compose_assign_list_multi_inst(inner, instances, file_name)?
                {
                    let (dst_offset, dst_inst) = match instances.get(&dst_inst_name) {
                        None => {
                            panic!(
                                "instance {} not declared but used in compose block for {}",
                                dst_inst_name, inst_name
                            );
                        }
                        Some(inst) => inst,
                    };

                    let oracle_sig = match dst_inst
                        .pkg
                        .oracles
                        .iter()
                        .find(|oracle_def| oracle_def.sig.name == oracle_name)
                    {
                        None => {
                            panic!(
                                "oracle {} not found in package instance {}",
                                oracle_name, dst_inst_name
                            );
                        }
                        Some(def) => def.sig.clone(),
                    };

                    edges.push(MultiInstanceEdge {
                        loopvars: loopvars.clone(),
                        source_pkgidx: *src_offset,
                        dest_pkgidx: *dst_offset,
                        oracle_sig,
                        dest_instance_idx,
                        source_instance_idx: source_instance_idx.clone(),
                    });
                }
            }
        }
    }

    Ok((edges, exports))
}

pub fn handle_for_loop(
    ast: Pair<Rule>,
    comp_name: &str,
    loopvars: &mut Vec<ForSpec>,
    pkgs: &HashMap<String, Package>,
    pkg_insts: &HashMap<String, (usize, PackageInstance)>,
    consts: &HashMap<String, Type>,
    file_name: &str,
) -> error::Result<(
    Vec<PackageInstance>,
    Vec<MultiInstanceEdge>,
    Vec<MultiInstanceExport>,
)> {
    let mut parsed: Vec<Pair<Rule>> = ast.into_inner().collect();
    let decl_var_name = parsed[0].as_str();
    let lower_bound = handle_expression(parsed.remove(1));
    let lower_bound_type = parsed[1].as_str();
    let bound_var_name = parsed[2].as_str();
    let upper_bound_type = parsed[3].as_str();
    let upper_bound = handle_expression(parsed.remove(4));
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

    let for_spec = ForSpec::new(
        Identifier::Scalar(decl_var_name.into()),
        lower_bound,
        upper_bound,
        start_comp,
        end_comp,
    );

    loopvars.push(for_spec);

    let result = handle_for_loop_body(
        body_ast, comp_name, loopvars, pkgs, pkg_insts, consts, file_name,
    );

    loopvars.pop();

    result
}

pub fn handle_comp_spec_list(
    ast: Pair<Rule>,
    game_name: &str,
    file_name: &str,
    pkg_map: &HashMap<String, Package>,
) -> error::Result<Composition> {
    let span = ast.as_span();
    let mut consts = HashMap::new();
    let mut consts_as_list = vec![];
    let mut instances = vec![];
    let mut instance_table = HashMap::new();

    let mut edges = None;
    let mut exports = None;

    let mut multi_edges = vec![];
    let mut multi_exports = vec![];

    /*
    Note: the grammar enforces that we first have the const declarations,
    then the instance declarations and only then the composition block.
    This means that we can assume that the last step is done processing.
    */
    for comp_spec in ast.into_inner() {
        match comp_spec.as_rule() {
            Rule::const_decl => {
                let (name, tipe) = handle_const_decl(comp_spec);
                consts_as_list.push((name.clone(), tipe.clone()));
                consts.insert(name, tipe);
            }
            Rule::game_for => {
                let (mult_pkg_insts, mut new_multi_edges, mut new_multi_exports) = handle_for_loop(
                    comp_spec,
                    game_name,
                    &mut vec![],
                    pkg_map,
                    &instance_table,
                    &consts,
                    file_name,
                )?;

                println!("after handle for:");
                println!("  game name: {game_name}");
                for pkg_inst in &mult_pkg_insts {
                    println!(
                        "    pkg_inst: {:} {:?}",
                        pkg_inst.as_name(),
                        pkg_inst.multi_instance_indices
                    );
                }

                for inst in mult_pkg_insts {
                    instances.push(inst.clone());
                    let offset = instances.len() - 1;
                    instance_table.insert(inst.name.clone(), (offset, inst));
                }

                multi_edges.append(&mut new_multi_edges);
                multi_exports.append(&mut new_multi_exports);
            }
            Rule::instance_decl => {
                let inst = handle_instance_decl(comp_spec, game_name, pkg_map, &consts, file_name)?;
                instances.push(inst.clone());
                let offset = instances.len() - 1;
                instance_table.insert(inst.name.clone(), (offset, inst));
            }
            Rule::compose_decl => {
                let comp_spec_span = comp_spec.as_span();
                let (edges_, exports_) =
                    handle_compose_assign_body_list(comp_spec, &instance_table, file_name)
                        .map_err(|e| error::Error::ParseGameError(e).with_span(comp_spec_span))?;
                edges = Some(edges_);
                exports = Some(exports_);
            }
            _ => {
                unreachable!();
            }
        }
    }

    let (edges, exports) = match (edges, exports) {
        (None, None) => Err(error::Error::MissingComposeBlock {
            game_name: game_name.to_string(),
        }
        .with_span(span)),
        (Some(edges), Some(exports)) => Ok((edges, exports)),
        _ => {
            unreachable!();
        }
    }?;

    let mut consts = Vec::from_iter(consts);
    consts.sort();

    Ok(Composition {
        edges,
        exports,
        split_exports: vec![],
        name: game_name.to_owned(),
        pkgs: instances,
        consts,
        multi_inst_edges: multi_edges,
        multi_inst_exports: multi_exports,
    })
}

pub fn handle_instance_assign_list(
    ast: Pair<Rule>,
    inst_name: &str,
    file_name: &str,
    defined_consts: &[(String, Type)],
) -> error::Result<(Vec<(String, Expression)>, Vec<(Type, Type)>)> {
    let mut params = vec![];
    let mut types = vec![];

    for elem in ast.into_inner() {
        match elem.as_rule() {
            Rule::params_def => {
                let mut defs =
                    handle_params_def_list(elem.into_inner().next().unwrap(), defined_consts)?;
                params.append(&mut defs);
            }
            Rule::types_def => {
                let mut defs =
                    handle_types_def_list(elem.into_inner().next().unwrap(), inst_name, file_name)?;
                types.append(&mut defs);
            }
            _ => unreachable!("{:#?}", elem),
        }
    }

    Ok((params, types))
}

pub fn handle_instance_decl_multi_inst(
    ast: Pair<Rule>,
    game_name: &str,
    pkg_map: &HashMap<String, Package>,
    consts: &HashMap<String, Type>,
    loopvars: &Vec<ForSpec>,
    file_name: &str,
) -> error::Result<PackageInstance> {
    let span = ast.as_span();

    let mut inner = ast.into_inner();
    let inst_name = inner.next().unwrap().as_str();
    let index_id_list: Vec<_> = inner
        .next()
        .unwrap()
        .into_inner()
        .map(|ast| ast.as_str())
        .collect();

    let pkg_name = inner.next().unwrap().as_str();
    let data = inner.next().unwrap();

    for index in &index_id_list {
        if loopvars
            .iter()
            .find(|for_spec| for_spec.ident().ident_ref() == *index)
            .is_none()
        {
            return Err(error::Error::UndefinedIdentifer(index.to_string()).with_span(span));
        }
    }

    let pkg = match pkg_map.get(pkg_name) {
        None => {
            panic!("package {} is unknown", pkg_name);
        }
        Some(pkg) => Ok(pkg),
    }?;

    // a list of (name, type) pairs of game constants/params and loop variables
    let defined_consts: Vec<_> = consts
        .iter()
        .map(|(name, tipe)| (name.clone(), tipe.clone()))
        .chain(
            loopvars
                .iter()
                .map(|for_spec| (for_spec.ident().ident(), Type::Integer)),
        )
        .collect();

    let data_span = data.as_span();
    let (mut param_list, type_list) =
        handle_instance_assign_list(data, inst_name, file_name, &defined_consts)?;

    param_list.sort();

    // check that const param lists match
    let typed_params: Result<Vec<_>, _> = param_list
        .iter()
        .map(|(pkg_param, comp_param)| {
            let const_type = match comp_param {
                Expression::Identifier(id) => defined_consts
                    .iter()
                    .find(|(name, _)| name == &id.ident())
                    .map(|(_, tipe)| tipe.to_owned())
                    .unwrap(),
                Expression::BooleanLiteral(_) => Type::Boolean,
                Expression::IntegerLiteral(_) => Type::Integer,
                otherwise => {
                    panic!("unhandled expression: {:?}", otherwise)
                }
            };

            Ok((pkg_param.to_string(), const_type))
        })
        .collect();
    let mut typed_params = typed_params?;
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

    let mut pkg_types = pkg.types.clone();
    pkg_types.sort();

    if assigned_types != pkg_types {
        // TODO include the difference in here
        return Err(error::SpanError::new_with_span(
            error::Error::TypeParameterMismatch {
                game_name: game_name.to_string(),
                pkg_name: pkg_name.to_string(),
                pkg_inst_name: inst_name.to_string(),
            },
            span,
        ));
    }

    let multi_instance_indices = index_id_list.into_iter().map(str::to_string).collect();

    let inst = PackageInstance {
        name: inst_name.to_owned(),
        params: param_list,
        types: type_list,
        pkg: pkg.clone(),
        multi_instance_indices,
        forspecs: loopvars.clone(),
    };

    match ResolveTypesPackageInstanceTransform.transform_package_instance(&inst) {
        Ok((inst, _)) => Ok(inst),
        Err(err) => Err(error::Error::from(err).with_span(span)),
    }
}

pub fn handle_index_id_list(ast: Pair<Rule>) -> Vec<String> {
    assert_eq!(ast.as_rule(), Rule::index_id_list);
    ast.into_inner()
        .map(|ast| ast.as_str().to_string())
        .collect()
}

pub fn handle_instance_decl(
    ast: Pair<Rule>,
    game_name: &str,
    pkg_map: &HashMap<String, Package>,
    consts: &HashMap<String, Type>,
    file_name: &str,
) -> error::Result<PackageInstance> {
    let span = ast.as_span();

    let mut inner = ast.into_inner();
    let inst_name = inner.next().unwrap().as_str();

    let index_or_pkgname = inner.next().unwrap();
    let (multi_instance_indices, pkg_name) = if index_or_pkgname.as_rule() == Rule::index_id_list {
        (
            handle_index_id_list(index_or_pkgname),
            inner.next().unwrap().as_str(),
        )
    } else {
        (vec![], index_or_pkgname.as_str())
    };

    let data = inner.next().unwrap();

    let pkg = match pkg_map.get(pkg_name) {
        None => {
            panic!("package {} is unknown", pkg_name);
        }
        Some(pkg) => Ok(pkg),
    }?;

    let defined_consts: Vec<_> = consts
        .iter()
        .map(|(name, tipe)| (name.clone(), tipe.clone()))
        .collect();

    let (mut param_list, type_list) =
        handle_instance_assign_list(data, inst_name, file_name, &defined_consts)?;

    param_list.sort();

    // check that const param lists match
    let mut typed_params: Vec<_> = param_list
        .iter()
        .map(|(pkg_param, comp_param)| match comp_param {
            Expression::Identifier(id) => {
                let maybe_type = consts.get(&id.ident());

                assert!(
                    maybe_type.is_some(),
                    "constant not specified: {} at {:?}",
                    id.ident(),
                    span
                );
                (pkg_param.to_owned(), maybe_type.unwrap().clone())
            }
            Expression::BooleanLiteral(_) => (pkg_param.to_string(), Type::Boolean),
            Expression::IntegerLiteral(_) => (pkg_param.to_string(), Type::Integer),
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

    let mut pkg_types = pkg.types.clone();
    pkg_types.sort();

    if assigned_types != pkg_types {
        // TODO include the difference in here
        return Err(error::SpanError::new_with_span(
            error::Error::TypeParameterMismatch {
                game_name: game_name.to_string(),
                pkg_name: pkg_name.to_string(),
                pkg_inst_name: inst_name.to_string(),
            },
            span,
        ));
    }

    let inst = PackageInstance {
        name: inst_name.to_owned(),
        params: param_list,
        types: type_list,
        pkg: pkg.clone(),
        multi_instance_indices,
        forspecs: vec![],
    };

    match ResolveTypesPackageInstanceTransform.transform_package_instance(&inst) {
        Ok((inst, _)) => Ok(inst),
        Err(err) => Err(error::Error::from(err).with_span(span)),
    }
}

pub fn handle_composition(
    ast: Pair<Rule>,
    pkg_map: &HashMap<String, Package>,
    file_name: &str,
) -> error::Result<Composition> {
    let mut inner = ast.into_inner();
    let game_name = inner.next().unwrap().as_str();
    let spec = inner.next().unwrap();
    handle_comp_spec_list(spec, game_name, file_name, pkg_map)
}
