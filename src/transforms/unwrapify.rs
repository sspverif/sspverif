use std::convert::Infallible;

use miette::SourceSpan;

use crate::expressions::Expression;
use crate::identifier::Identifier;
use crate::package::Composition;
use crate::statement::{CodeBlock, Statement};

pub type Error = Infallible;

pub struct Transformation<'a>(pub &'a Composition);

impl<'a> super::Transformation for Transformation<'a> {
    type Err = Error;
    type Aux = ();

    fn transform(&self) -> Result<(Composition, ()), Error> {
        let insts: Result<Vec<_>, Infallible> = self
            .0
            .pkgs
            .iter()
            .map(|inst| {
                let mut newinst = inst.clone();
                for (i, oracle) in newinst.pkg.oracles.clone().iter().enumerate() {
                    let mut ctr = 1usize;
                    newinst.pkg.oracles[i].code = unwrapify(&oracle.code, &mut ctr)?;
                }
                Ok(newinst)
            })
            .collect();
        Ok((
            Composition {
                pkgs: insts?,
                ..self.0.clone()
            },
            (),
        ))
    }
}

fn replace_unwrap(expr: &Expression, ctr: &mut usize) -> (Expression, Vec<(Expression, String)>) {
    let ((local_ctr, result), newexpr) =
        expr.mapfold((*ctr, Vec::new()), |(map_ctr, mut ac), e| {
            if let Expression::Unwrap(_) = e {
                let varname: String = format!("unwrap-{}", map_ctr);
                let ty = e.get_type();
                ac.push((e, varname.clone()));
                ((map_ctr + 1, ac), Identifier::Generated(varname, ty).into())
            } else {
                ((map_ctr, ac), e)
            }
        });
    *ctr = local_ctr;
    (newexpr, result)
}

fn create_unwrap_stmts(
    unwraps: Vec<(Expression, String)>,
    file_pos: &SourceSpan,
) -> Vec<Statement> {
    unwraps
        .iter()
        .map(|(expr, varname)| {
            Statement::Assign(
                Identifier::Generated(varname.clone(), expr.get_type()),
                None,
                expr.clone(),
                *file_pos,
            )
        })
        .collect()
}

/*
[0] foo <- bar(unwrap(x))

replace_unwrap([0])
 -> x, unwrap-x-12

[0] wurde zu foo <- bar(unwrap-12-x)
*/

pub fn unwrapify(cb: &CodeBlock, ctr: &mut usize) -> Result<CodeBlock, Error> {
    let mut newcode = Vec::new();
    for stmt in cb.0.clone() {
        match stmt {
            Statement::Abort(_)
            | Statement::Sample(_, None, _, _, _)
            | Statement::Return(None, _) => {
                newcode.push(stmt);
            }
            Statement::Return(Some(ref expr), ref file_pos) => {
                let (newexpr, unwraps) = replace_unwrap(expr, ctr);
                if unwraps.is_empty() {
                    newcode.push(stmt);
                } else {
                    newcode.extend(create_unwrap_stmts(unwraps, file_pos));
                    newcode.push(Statement::Return(Some(newexpr), *file_pos));
                }
            }
            Statement::Assign(ref id, ref opt_idx, ref expr, ref file_pos) => {
                // TODO: special case for direct unwraps

                let opt_idx = opt_idx.clone().map(|idx| {
                    let (newexpr, unwraps) = replace_unwrap(&idx, ctr);
                    newcode.extend(create_unwrap_stmts(unwraps, file_pos));
                    newexpr
                });

                let (newexpr, unwraps) = replace_unwrap(expr, ctr);
                if unwraps.is_empty() {
                    newcode.push(stmt);
                } else {
                    newcode.extend(create_unwrap_stmts(unwraps, file_pos));
                    newcode.push(Statement::Assign(
                        id.clone(),
                        opt_idx.clone(),
                        newexpr,
                        *file_pos,
                    ));
                }
            }
            Statement::IfThenElse(expr, ifcode, elsecode, file_pos) => {
                let (newexpr, unwraps) = replace_unwrap(&expr, ctr);
                newcode.extend(create_unwrap_stmts(unwraps, &file_pos));
                newcode.push(Statement::IfThenElse(
                    newexpr,
                    unwrapify(&ifcode, ctr)?,
                    unwrapify(&elsecode, ctr)?,
                    file_pos,
                ));
            }
            Statement::For(ident, lower_bound, upper_bound, body, file_pos) => {
                let (new_lower_bound, unwraps) = replace_unwrap(&lower_bound, ctr);
                newcode.extend(create_unwrap_stmts(unwraps, &file_pos));
                let (new_upper_bound, unwraps) = replace_unwrap(&upper_bound, ctr);
                newcode.extend(create_unwrap_stmts(unwraps, &file_pos));
                newcode.push(Statement::For(
                    ident,
                    new_lower_bound,
                    new_upper_bound,
                    unwrapify(&body, ctr)?,
                    file_pos,
                ))
            }
            Statement::Sample(ref id, Some(ref expr), ref sample_id, ref tipe, ref file_pos) => {
                let (newexpr, unwraps) = replace_unwrap(expr, ctr);
                if unwraps.is_empty() {
                    newcode.push(stmt);
                } else {
                    newcode.extend(create_unwrap_stmts(unwraps, file_pos));
                    newcode.push(Statement::Sample(
                        id.clone(),
                        Some(newexpr),
                        *sample_id,
                        tipe.clone(),
                        *file_pos,
                    ));
                }
            }
            Statement::Parse(idents, expr, file_pos) => {
                let (newexpr, unwraps) = replace_unwrap(&expr, ctr);

                newcode.extend(create_unwrap_stmts(unwraps, &file_pos));
                newcode.push(Statement::Parse(idents, newexpr, file_pos));
            }
            Statement::InvokeOracle {
                id,
                opt_idx,
                opt_dst_inst_idx,
                name,
                args,
                target_inst_name,
                tipe,
                file_pos,
            } => {
                let opt_idx = opt_idx.map(|expr| {
                    let (newexpr, unwraps) = replace_unwrap(&expr, ctr);
                    newcode.extend(create_unwrap_stmts(unwraps, &file_pos));
                    newexpr
                });
                let args = args
                    .iter()
                    .map(|expr| {
                        let (newexpr, unwraps) = replace_unwrap(expr, ctr);
                        newcode.extend(create_unwrap_stmts(unwraps, &file_pos));
                        newexpr
                    })
                    .collect();
                newcode.push(Statement::InvokeOracle {
                    id,
                    opt_idx,
                    opt_dst_inst_idx,
                    name,
                    args,
                    target_inst_name,
                    tipe,
                    file_pos,
                });
            }
        }
    }
    Ok(CodeBlock(newcode))
}

#[cfg(test)]
mod test {
    use miette::SourceSpan;

    use super::unwrapify;
    use crate::block;
    use crate::expressions::Expression;
    use crate::identifier::pkg_ident::{PackageIdentifier, PackageLocalIdentifier};
    use crate::identifier::Identifier;
    use crate::statement::{CodeBlock, Statement};
    use crate::types::Type;

    fn local_ident(name: &str, ty: Type) -> Identifier {
        Identifier::PackageIdentifier(PackageIdentifier::Local(PackageLocalIdentifier {
            pkg_name: "TestPackage".to_string(),
            oracle_name: "TestOracle".to_string(),
            name: name.to_string(),
            tipe: ty,
            pkg_inst_name: None,
            game_name: None,
            game_inst_name: None,
            proof_name: None,
        }))
    }

    fn gend_ident(name: &str, ty: Type) -> Identifier {
        Identifier::Generated(name.to_string(), ty)
    }

    fn maybe(ty: Type) -> Type {
        Type::Maybe(Box::new(ty))
    }

    fn unwrap<E: Clone + Into<Expression>>(expr: &E) -> Expression {
        Expression::Unwrap(Box::new(expr.clone().into()))
    }

    #[test]
    fn unwrap_assign() {
        let pos: SourceSpan = (0..0).into();
        let d = local_ident("d", Type::Integer);
        let e = local_ident("e", maybe(Type::Integer));
        let u1 = gend_ident("unwrap-1", Type::Integer);

        let code = block! {
            Statement::Assign(d.clone(), None, unwrap(&e), pos)
        };
        let newcode = block! {
            Statement::Assign(u1.clone(), None, unwrap(&e),pos),
            Statement::Assign(d.clone(), None, u1.into(), pos)
        };
        let mut ctr = 1usize;
        assert_eq!(newcode, unwrapify(&code, &mut ctr).unwrap());
    }

    #[test]
    fn unwrap_two_statements() {
        let pos0: SourceSpan = (0..0).into();
        let pos1: SourceSpan = (1..1).into();

        let d = local_ident("d", Type::Integer);
        let e = local_ident("e", maybe(Type::Integer));
        let u1 = gend_ident("unwrap-1", Type::Integer);

        let f = local_ident("f", Type::Integer);
        let g = local_ident("g", maybe(Type::Integer));
        let u2 = gend_ident("unwrap-2", Type::Integer);

        let code = block! {
            Statement::Assign(d.clone(), None, unwrap(&e), pos0),
            Statement::Assign(f.clone(), None, unwrap(&g), pos1)
        };
        let newcode = block! {
            Statement::Assign(u1.clone(), None, unwrap(&e), pos0),
            Statement::Assign(d, None, u1.into(), pos0),
            Statement::Assign(u2.clone(), None, unwrap(&g), pos1),
            Statement::Assign(f, None, u2.into(), pos1)
        };
        let mut ctr = 1usize;
        assert_eq!(newcode, unwrapify(&code, &mut ctr).unwrap());
    }

    #[test]
    fn nested_statements() {
        let pos: SourceSpan = (0..0).into();

        let d = local_ident("d", Type::Integer);
        let e = local_ident("e", maybe(maybe(Type::Integer)));
        let u1 = gend_ident("unwrap-1", maybe(Type::Integer));
        let u2 = gend_ident("unwrap-2", Type::Integer);
        let code = block! {
            Statement::Assign(d.clone(), None, unwrap(&unwrap(&e)), pos)
        };
        let newcode = block! {
                    Statement::Assign(u1.clone(), None, unwrap(&e), pos ),
                    Statement::Assign(u2.clone(), None, unwrap(&u1), pos ),
                    Statement::Assign(d, None, u2.into(), pos)
        };
        let mut ctr = 1usize;
        assert_eq!(newcode, unwrapify(&code, &mut ctr).unwrap());
    }
}
