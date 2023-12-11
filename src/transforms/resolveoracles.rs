use crate::package::{Composition, Edge, OracleDef, Package, PackageInstance};
use crate::statement::{CodeBlock, Statement};

use std::collections::HashMap;

pub struct Transformation<'a>(pub &'a Composition);

#[derive(Debug)]
pub struct ResolutionError(Vec<Statement>);

type Result<T> = std::result::Result<T, ResolutionError>;

fn transform_helper_outer(table: &HashMap<String, String>, block: CodeBlock) -> Result<CodeBlock> {
    fn transform_helper(
        table: &HashMap<String, String>,
        block: CodeBlock,
        err_stmts: &mut Vec<Statement>,
    ) -> CodeBlock {
        let out = block
            .0
            .iter()
            .map(|stmt| match stmt {
                Statement::InvokeOracle {
                    id,
                    opt_idx,
                    name,
                    args,
                    tipe,
                    file_pos,
                    opt_dst_inst_idx,
                    ..
                } => {
                    if let Some(pkgname) = table.get(name) {
                        Statement::InvokeOracle {
                            id: id.clone(),
                            opt_idx: opt_idx.clone(),
                            opt_dst_inst_idx: opt_dst_inst_idx.clone(),
                            name: name.clone(),
                            args: args.clone(),
                            tipe: tipe.clone(),
                            target_inst_name: Some(pkgname.clone()),
                            file_pos: file_pos.clone(),
                        }
                    } else {
                        //return ResolutionError(stmt);
                        panic!();
                    }
                }
                Statement::IfThenElse(cond, ifcode, elsecode, file_pos) => Statement::IfThenElse(
                    cond.clone(),
                    transform_helper(table, ifcode.clone(), err_stmts),
                    transform_helper(table, elsecode.clone(), err_stmts),
                    file_pos.clone(),
                ),
                _ => stmt.clone(),
            })
            .collect();

        CodeBlock(out)
    }

    let mut err_stmts = vec![];

    let out = transform_helper(table, block, &mut err_stmts);

    if !err_stmts.is_empty() {
        Err(ResolutionError(err_stmts))
    } else {
        Ok(out)
    }
}

impl<'a> super::Transformation for Transformation<'a> {
    type Err = ResolutionError;
    type Aux = ();

    fn transform(&self) -> Result<(Composition, ())> {
        let mut pkgs = vec![];

        for (pos, inst) in self.0.pkgs.iter().enumerate() {
            let mut table = HashMap::<String, String>::new();
            let relevant = self.0.edges.iter().filter(|Edge(from, _, _)| *from == pos);

            for Edge(_, to, sig) in relevant {
                let pkgname = self.0.pkgs[*to].name.clone();
                table.insert(sig.name.clone(), pkgname);
            }

            let mut odefs = vec![];
            for oracle in &inst.pkg.oracles {
                odefs.push(OracleDef {
                    code: transform_helper_outer(&table, oracle.code.clone())?,
                    ..oracle.clone()
                });
            }

            pkgs.push(PackageInstance {
                pkg: Package {
                    oracles: odefs,
                    ..inst.pkg.clone()
                },
                ..inst.clone()
            });
        }
        Ok((
            Composition {
                pkgs,
                ..self.0.clone()
            },
            (),
        ))
    }
}
