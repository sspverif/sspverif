use crate::project;
use std::io::Write;
use std::io::{self, BufRead};

use crate::parser::{Rule, SspParser};
use crate::types::Type;
use miette::{Diagnostic, NamedSource, SourceSpan};
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct FormatContext<'a> {
    pub file_name: &'a str,
    pub file_content: &'a str,

    pub indent_level: usize,
    buffer: String,
}

impl<'a> FormatContext<'a> {
    pub fn new(file_name: &'a str, file_content: &'a str) -> Self {
        FormatContext {
            file_name,
            file_content,
            indent_level: 0,
            buffer: String::new(),
        }
    }

    pub fn to_str(&self) -> &str {
        &self.buffer
    }

    pub fn push_line(&mut self, line: &str) {
        if line == "" {
            self.buffer.push_str("\n");
        } else {
            let indent = std::iter::repeat("    ")
                .take(self.indent_level)
                .collect::<String>();
            self.buffer.push_str(&format!("{indent}{line}\n"));
        }
    }

    pub fn add_indent(&mut self) {
        self.indent_level += 1
    }

    pub fn remove_indent(&mut self) {
        self.indent_level = if self.indent_level > 0 {
            self.indent_level - 1
        } else {
            0
        }
    }
}

fn format_oracle_sig(
    ctx: &mut FormatContext,
    oracle_sig_ast: Pair<Rule>,
) -> Result<(), project::error::Error> {
    let mut inner = oracle_sig_ast.into_inner();
    let name = inner.next().unwrap().as_str();
    let maybe_arglist = inner.next().unwrap();
    let args = if maybe_arglist.as_str() == "" {
        vec![]
    } else {
        let arglist = maybe_arglist.into_inner().next().unwrap();
        arglist
            .into_inner()
            .map(|arg| {
                let mut inner = arg.into_inner();
                let id = inner.next().unwrap().as_str();
                let tipe = format_type(inner.next().unwrap())?;
                Ok(format!("{id}: {tipe}"))
            })
            .collect::<Result<Vec<String>,project::error::Error>>()?
    };

    let maybe_tipe = inner.next();
    let tipe = match maybe_tipe {
        None => "()",
        Some(t) => &format_type(t)?,
    };

    ctx.push_line(&format!(
        "oracle {}({}) -> {} {{",
        name,
        args.join(", "),
        tipe
    ));
    Ok(())
}

fn format_type(tipe_ast: Pair<Rule>) -> Result<String, project::error::Error> {
	Ok(match tipe_ast.as_rule() {
		Rule::type_tuple => {
            let inner = tipe_ast.into_inner()
                .map(|t| format_type(t))
                .collect::<Result<Vec<_>, _>>()?
				.join(", ");
			format!("({inner})")
        }
		Rule::type_table => {
            let mut inner = tipe_ast.into_inner();
            let indextype = format_type(inner.next().unwrap())?;
            let valuetype = format_type(inner.next().unwrap())?;
            format!("Table({indextype}, {valuetype})")
        }
		_ => tipe_ast.as_str().to_owned()
	})
}

fn format_expr(expr_ast: Pair<Rule>) -> Result<String, project::error::Error> {
    let span = expr_ast.as_span();
    return Ok(match expr_ast.as_rule() {
        Rule::expr_sub => {
            let mut inner = expr_ast.into_inner();
            let lhs = format_expr(inner.next().unwrap())?;
            let rhs = format_expr(inner.next().unwrap())?;
            format!("({lhs} - {rhs})")
        }
        Rule::expr_equals => {
            let concat_expr = expr_ast
                .into_inner()
                .map(|expr| format_expr(expr))
                .collect::<Result<Vec<_>, _>>()?
                .join(" == ");
            format!("({concat_expr})")
        }
        Rule::expr_not_equals => {
            let concat_expr = expr_ast
                .into_inner()
                .map(|expr| format_expr(expr))
                .collect::<Result<Vec<_>, _>>()?
                .join(" != ");
            format!("({concat_expr})")
        }
        Rule::expr_or => {
            let concat_expr = expr_ast
                .into_inner()
                .map(|expr| format_expr(expr))
                .collect::<Result<Vec<_>, _>>()?
                .join(" or ");
            format!("({concat_expr})")
        }
        Rule::expr_tuple => {
            let concat_expr = expr_ast
                .into_inner()
                .map(|expr| format_expr(expr))
                .collect::<Result<Vec<_>, _>>()?
                .join(", ");
            format!("({concat_expr})")
        }
        Rule::expr_not => {
            let inner = format_expr(expr_ast.into_inner().next().unwrap())?;
            format!("not {inner}")
        }
        Rule::expr_none => {
            let inner = format_type(expr_ast.into_inner().next().unwrap())?;
            format!("None as {inner}")
        }
        Rule::expr_some => {
            let inner = format_expr(expr_ast.into_inner().next().unwrap())?;
            format!("Some({inner})")
        }
        Rule::expr_unwrap => {
            let inner = format_expr(expr_ast.into_inner().next().unwrap())?;
            format!("Unwrap({inner})")
        }
        Rule::expr_newtable => {
            let mut inner = expr_ast.into_inner();
            let idxtipe = format_type(inner.next().unwrap())?;
            let valtipe = format_type(inner.next().unwrap())?;
            format!("new Table({idxtipe}, {valtipe})")
        }
        Rule::fn_call => {
            let mut inner = expr_ast.into_inner();
            let ident = inner.next().unwrap().as_str();
            let args = match inner.next() {
                None => vec![],
                Some(inner_args) => inner_args
                    .into_inner()
                    .map(|expr| format_expr(expr))
                    .collect::<Result<_, _>>()?,
            }.join(", ");
			format!("{ident}({args})")
        }
        Rule::literal_boolean | Rule::literal_integer => expr_ast.as_str().to_string(),
        Rule::identifier => {
            let name = expr_ast.as_str();
            name.to_owned()
        }
        Rule::table_access => {
            let mut inner = expr_ast.into_inner();
            let ident_ast = inner.next().unwrap();
            let ident = ident_ast.as_str();
            let idx_expr = format_expr(inner.next().unwrap())?;
            format!("{ident}[{idx_expr}]")
        }
        _ => unreachable!("Unhandled expression {:#?}", expr_ast),
    });
}

fn format_code(ctx: &mut FormatContext, code_ast: Pair<Rule>) -> Result<(), project::error::Error> {
    for stmt in code_ast.into_inner() {
        let span = stmt.as_span();
        let source_span = SourceSpan::from(span.start()..span.end());
        //let file_pos = FilePosition::from_span(ctx.file_name, span);

        match stmt.as_rule() {
            Rule::ite => {
                let mut inner = stmt.into_inner();
                let cond_expr = format_expr(inner.next().unwrap())?;

                ctx.push_line(&format!("if {cond_expr} {{"));
                ctx.add_indent();
                format_code(ctx, inner.next().unwrap())?;
                ctx.remove_indent();

                match inner.next() {
                    None => {}
                    Some(c) => {
                        ctx.push_line("} else {");
                        ctx.add_indent();
                        format_code(ctx, c)?;
                        ctx.remove_indent();
                    }
                }

                ctx.push_line("}");
            }
            Rule::return_stmt => {
                let mut inner = stmt.into_inner();

                match inner.next() {
                    None => {
                        ctx.push_line(&format!("return;"));
                    }
                    Some(e) => {
                        ctx.push_line(&format!("return {};", format_expr(e)?));
                    }
                }
            }
            Rule::assert => {
                let mut inner = stmt.into_inner();

                ctx.push_line(&format!("assert {};", format_expr(inner.next().unwrap())?));
            }
            Rule::abort => {
                ctx.push_line(&format!("abort;"));
            }
            Rule::sample => {
                let mut inner = stmt.into_inner();
                let name_ast = inner.next().unwrap();
                let tipe = inner.next().unwrap().as_str();
                let ident = name_ast.as_str();

                ctx.push_line(&format!("{ident} <-$ {tipe};"));
            }

            Rule::assign => {
                let mut inner = stmt.into_inner();
                let name_ast = inner.next().unwrap();
                let name = name_ast.as_str();
                let expr = format_expr(inner.next().unwrap())?;

                ctx.push_line(&format!("{name} <- {expr};"));
            }

            Rule::table_sample => {
                let mut inner = stmt.into_inner();
                let name_ast = inner.next().unwrap();
                let ident = name_ast.as_str();
                let index = format_expr(inner.next().unwrap())?;
                let tipe = inner.next().unwrap().as_str();

                ctx.push_line(&format!("{ident}[{index}] <-$ {tipe};"));
            }

            Rule::table_assign => {
                let mut inner = stmt.into_inner();
                let name_ast = inner.next().unwrap();
                let name = name_ast.as_str();
                let index = format_expr(inner.next().unwrap())?;
                let expr = format_expr(inner.next().unwrap())?;

                ctx.push_line(&format!("{name}[{index}] <- {expr};"));
            }

            Rule::invocation => {
                let mut inner = stmt.into_inner();
                let target_ident_name_ast = inner.next().unwrap();
                let ident = target_ident_name_ast.as_str();
                let maybe_index = inner.next().unwrap();
                let (index, oracle_inv) = if maybe_index.as_rule() == Rule::table_index {
                    let mut inner_index = maybe_index.into_inner();
                    (
                        format!("[{}]", format_expr(inner_index.next().unwrap())?),
                        inner.next().unwrap(),
                    )
                } else {
                    ("".to_owned(), maybe_index)
                };
                let mut inner = oracle_inv.into_inner();
                let oracle_name_ast = inner.next().unwrap();
                let oracle_name_span = oracle_name_ast.as_span();
                let oracle_name = oracle_name_ast.as_str();
                let multi_instance = String::new();
                let mut argstring = String::new();

                for ast in inner {
                    match ast.as_rule() {
                        Rule::oracle_call_index => {
                            let index_expr_ast = ast.into_inner().next().unwrap();
                            let multi_instance = format!("[{}]", format_expr(index_expr_ast)?);
                        }
                        Rule::fn_call_arglist => {
                            let arglist: Vec<_> = ast
                                .into_inner()
                                .map(|expr| format_expr(expr))
                                .collect::<Result<Vec<_>, _>>()?;
                            argstring.push_str(&arglist.join(", "));
                        }
                        _ => unreachable!(),
                    }
                }

                ctx.push_line(&format!(
                    "{ident}[index] <- invoke {oracle_name}{multi_instance}({argstring});"
                ));
            }
            Rule::parse => {
                let mut inner = stmt.into_inner();
                let list = inner.next().unwrap();
                let expr = format_expr(inner.next().unwrap())?;
                let idents = list
                    .into_inner()
                    .map(|(ident_name)| ident_name.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");

                ctx.push_line(&format!("(idents) <- parse {expr};"));
            }
            Rule::for_ => {
                let mut parsed: Vec<Pair<Rule>> = stmt.into_inner().collect();
                let decl_var_name = parsed[0].as_str();
                let lower_bound = format_expr(parsed.remove(1))?;
                let lower_bound_type = parsed[1].as_str();
                let bound_var_name = parsed[2].as_str();
                let upper_bound_type = parsed[3].as_str();
                let upper_bound = format_expr(parsed.remove(4))?;
                let loopvar = decl_var_name.to_string();

                ctx.push_line(&format!("for {loopvar}: {lower_bound} {lower_bound_type} {loopvar} {upper_bound_type} {upper_bound}"));
                ctx.add_indent();

                format_code(ctx, parsed.remove(4))?;

                ctx.remove_indent();
                ctx.push_line("}");
            }
            _ => {
                unreachable!("{:#?}", stmt)
            }
        }
    }

    Ok(())
}

fn format_oracle_def(
    ctx: &mut FormatContext,
    oracle_def_ast: Pair<Rule>,
) -> Result<(), project::error::Error> {
    let span = oracle_def_ast.as_span();
    let source_span = SourceSpan::from(span.start()..span.end());
    let mut inner = oracle_def_ast.into_inner();

    ctx.push_line("");
    format_oracle_sig(ctx, inner.next().unwrap())?;
    ctx.add_indent();

    format_code(ctx, inner.next().unwrap())?;

    ctx.remove_indent();
    ctx.push_line("}");

    Ok(())
}


fn format_types_block(
    ctx: &mut FormatContext,
    types_ast: Pair<Rule>,
) -> Result<(), project::error::Error> {
    let mut inner = types_ast.into_inner();
    let typename = inner.next().unwrap().as_str();
    let typedef = format_type(inner.next().unwrap())?;
    ctx.push_line(&format!("({typename}: {typedef})"));
	Ok(())
}

fn format_decl_list(
    ctx: &mut FormatContext,
    decl_list_ast: Pair<Rule>,
) -> Result<(), project::error::Error> {
	for entry in decl_list_ast.into_inner() {
		let mut inner = entry.into_inner();
		let declname = inner.next().unwrap().as_str();
		let decldef = format_type(inner.next().unwrap())?;
		ctx.push_line(&format!("{declname}: {decldef},"));
	}
	Ok(())
}

fn format_import_oracles(
    ctx: &mut FormatContext,
    decl_list_ast: Pair<Rule>,
) -> Result<(), project::error::Error> {
	for entry in decl_list_ast.into_inner() {

		match entry.as_rule() {
			Rule::import_oracles_oracle_sig => {
				let mut inner = entry.into_inner();
				let ident = inner.next().unwrap().as_str();
				let args = inner.next().unwrap();
				if ! matches!(args.as_rule(), Rule::fn_maybe_arglist) {
					unimplemented!("cant do for imports for now");
				}

				let args = args.into_inner().map(|arg| arg.as_str().to_owned()).collect::<Vec<String>>().join(", ");
				let return_type = inner.next();
				match return_type {
					None => ctx.push_line(&format!("{ident}({args}) -> (),")),
					Some(t) => {
						let rettype = format_type(t)?;
						ctx.push_line(&format!("{ident}({args}) -> {rettype},"));
					}
				}
			}
			Rule::import_oracles_for => {
				unimplemented!("cant do for imports for now")
			}
			_ => {unreachable!("")}
		}
		// let mut inner = entry.into_inner();
		// let declname = inner.next().unwrap().as_str();
		// let decldef = format_type(inner.next().unwrap())?;
		// ctx.push_line(&format!("{declname}: {decldef},"));
	}
	Ok(())
}

fn format_pkg_spec(
    ctx: &mut FormatContext,
    pkg_spec_ast: Pair<Rule>,
) -> Result<(), project::error::Error> {
    // sort different types consistently when generating output
	let specs : Vec<_> = pkg_spec_ast.into_inner().collect();

	let types_rules : Vec<_> = specs.iter().filter(|x| matches!(x.as_rule(), Rule::types)).collect();
	let params_rules : Vec<_> = specs.iter().filter(|x| matches!(x.as_rule(), Rule::params)).collect();
	let state_rules : Vec<_> = specs.iter().filter(|x| matches!(x.as_rule(), Rule::state)).collect();
	let import_rules : Vec<_> = specs.iter().filter(|x| matches!(x.as_rule(), Rule::import_oracles)).collect();
	let oracle_def_rules : Vec<_> = specs.iter().filter(|x| matches!(x.as_rule(), Rule::oracle_def)).collect();

	if types_rules.len() > 0 {
		ctx.push_line("types {");
		ctx.add_indent();
		for type_block in types_rules {
			format_types_block(ctx,type_block.clone().into_inner().next().unwrap())?;
		}
		ctx.remove_indent();
		ctx.push_line("}");
		ctx.push_line("");
	}

	if params_rules.len() > 0 {
		ctx.push_line("params {");
		ctx.add_indent();
		for param_block in params_rules {
			format_decl_list(ctx,param_block.clone().into_inner().next().unwrap())?;
		}
		ctx.remove_indent();
		ctx.push_line("}");
		ctx.push_line("");
	}
	
	if state_rules.len() > 0 {
		ctx.push_line("state {");
		ctx.add_indent();
		for state_block in state_rules {
			format_decl_list(ctx,state_block.clone().into_inner().next().unwrap())?;
		}
		ctx.remove_indent();
		ctx.push_line("}");
		ctx.push_line("");
	}

	if import_rules.len() > 0 {
		ctx.push_line("import oracles {");
		ctx.add_indent();
		for import_block in import_rules {
			format_import_oracles(ctx,import_block.clone().into_inner().next().unwrap())?;
		}
		ctx.remove_indent();
		ctx.push_line("}");
		ctx.push_line("");
	}

	for oracle_def in oracle_def_rules {
        format_oracle_def(ctx, oracle_def.clone())?;
	}
		
	
    // for spec in pkg_spec_ast.into_inner() {
    //     match spec.as_rule() {
    //         Rule::types => {}
    //         Rule::params => {}
    //         Rule::state => {}
    //         Rule::import_oracles => {}
    //         Rule::oracle_def => {
    //             format_oracle_def(ctx, spec)?;
    //         }
    //         _ => unreachable!("unhandled ast node in package: {}", spec),
    //     }
    // }

    Ok(())
}

fn format_pkg(ctx: &mut FormatContext, pkg_ast: Pair<Rule>) -> Result<(), project::error::Error> {
    let mut inner = pkg_ast.into_inner();
    let pkg_name = inner.next().unwrap().as_str();
    let spec = inner.next().unwrap();

    ctx.push_line(&format!("package {pkg_name} {{"));
    ctx.add_indent();

    format_pkg_spec(ctx, spec)?;

    ctx.remove_indent();
    ctx.push_line("}");
    Ok(())
}

pub fn format_file(file: &std::path::PathBuf) -> Result<(), project::error::Error> {
    let mut indent = 0;
    let file_content = std::fs::read_to_string(file)?;

    let absname = std::path::absolute(file)?;
    let dirname = absname.parent().unwrap();
    let mut target = tempfile::NamedTempFile::new_in(dirname)?;

    let mut ast =
        SspParser::parse_package(&file_content).map_err(|e| (file.to_str().unwrap(), e))?;
    let mut ctx = FormatContext::new(file.to_str().unwrap(), &file_content);
    let formated = format_pkg(&mut ctx, ast.next().unwrap())?;

    write!(target, "{}", ctx.to_str());

	match target.persist(file) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.error),
    }?;
    Ok(())
}

