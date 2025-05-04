use crate::expressions::Expression;
use crate::package::Composition;
use crate::statement::{CodeBlock, IfThenElse, Statement};
use crate::types::Type;
use std::collections::HashSet;
use std::convert::Infallible;
use std::iter::FromIterator;

#[derive(Debug, Clone)]

pub struct Transformation<'a>(pub &'a Composition);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Position {
    pub game_name: String,
    pub inst_name: String,
    pub pkg_name: String,
    pub oracle_name: String,

    pub dst_name: String,
    pub dst_index: Option<Expression>,

    pub sample_id: usize,
    pub ty: Type,
    pub sample_name: Option<String>,
}

#[derive(Clone, Debug)]
pub struct SampleInfo {
    pub tys: Vec<Type>,
    pub count: usize,
    pub positions: Vec<Position>,
}

impl super::Transformation for Transformation<'_> {
    type Err = Infallible;
    type Aux = SampleInfo;

    fn transform(&self) -> Result<(Composition, SampleInfo), Infallible> {
        let mut ctr = 0usize;
        let mut samplings = HashSet::new();
        let mut positions = vec![];

        let game_name = self.0.name.as_str();

        let insts: Result<Vec<_>, Infallible> = self
            .0
            .pkgs
            .iter()
            .map(|inst| {
                let inst_name = inst.name.as_str();
                let pkg_name = inst.pkg.name.as_str();

                let mut newinst = inst.clone();
                for (i, oracle) in newinst.pkg.oracles.clone().iter().enumerate() {
                    newinst.pkg.oracles[i].code = samplify(
                        &oracle.code,
                        game_name,
                        pkg_name,
                        inst_name,
                        &oracle.sig.name,
                        &mut ctr,
                        &mut samplings,
                        &mut positions,
                    )?;
                }
                Ok(newinst)
            })
            .collect();
        Ok((
            Composition {
                pkgs: insts?,
                ..self.0.clone()
            },
            SampleInfo {
                tys: Vec::from_iter(samplings),
                count: ctr,
                positions,
            },
        ))
    }
}

pub fn samplify(
    cb: &CodeBlock,
    game_name: &str,
    pkg_name: &str,
    inst_name: &str,
    oracle_name: &str,
    ctr: &mut usize,
    sampletypes: &mut HashSet<Type>,
    positions: &mut Vec<Position>,
) -> Result<CodeBlock, Infallible> {
    let mut newcode = Vec::new();
    for stmt in cb.0.clone() {
        match stmt {
            Statement::IfThenElse(ite) => {
                newcode.push(Statement::IfThenElse(IfThenElse {
                    then_block: samplify(
                        &ite.then_block,
                        game_name,
                        pkg_name,
                        inst_name,
                        oracle_name,
                        ctr,
                        sampletypes,
                        positions,
                    )?,
                    else_block: samplify(
                        &ite.else_block,
                        game_name,
                        pkg_name,
                        inst_name,
                        oracle_name,
                        ctr,
                        sampletypes,
                        positions,
                    )?,
                    ..ite
                }));
            }
            Statement::For(iter, start, end, code, file_pos) => newcode.push(Statement::For(
                iter,
                start,
                end,
                samplify(
                    &code,
                    game_name,
                    pkg_name,
                    inst_name,
                    oracle_name,
                    ctr,
                    sampletypes,
                    positions,
                )?,
                file_pos,
            )),

            Statement::Sample(id, expr, None, ty, sample_name, file_pos) => {
                let pos = Position {
                    game_name: game_name.to_string(),
                    inst_name: inst_name.to_string(),
                    pkg_name: pkg_name.to_string(),
                    oracle_name: oracle_name.to_string(),
                    dst_name: id.ident(),
                    dst_index: expr.clone(),
                    sample_id: *ctr,
                    ty: ty.clone(),
                    sample_name: sample_name.clone(),
                };
                sampletypes.insert(ty.clone());
                positions.push(pos);
                newcode.push(Statement::Sample(
                    id.clone(),
                    expr,
                    Some(*ctr),
                    ty.clone(),
                    sample_name,
                    file_pos,
                ));
                *ctr += 1;
            }
            _ => newcode.push(stmt),
        }
    }
    Ok(CodeBlock(newcode))
}
