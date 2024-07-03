use std::convert::Infallible;

use miette::SourceSpan;

use crate::expressions::Expression;
use crate::identifier::Identifier;
use crate::package::Composition;
use crate::statement::{CodeBlock, FilePosition, Statement};

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
                ac.push((e, varname.clone()));
                (
                    (map_ctr + 1, ac),
                    Identifier::Local(varname).to_expression(),
                )
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
                Identifier::Local(varname.clone()),
                None,
                expr.clone(),
                file_pos.clone(),
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
                    newcode.push(Statement::Return(Some(newexpr), file_pos.clone()));
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
                        file_pos.clone(),
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
                        file_pos.clone(),
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
    use crate::identifier::Identifier;
    use crate::statement::{CodeBlock, FilePosition, Statement};
    use crate::types::Type;

    #[test]
    fn unwrap_assign() {
        let pos: SourceSpan = (0..0).into();
        let code = block! {
            Statement::Assign(Identifier::Local("d".to_string()), None,
                              Expression::Typed(Type::Integer,Box::new(
                                  Expression::Unwrap(Box::new(
                                    Expression::Typed(Type::Integer,Box::new(
                                      Identifier::Local("e".to_string()).to_expression())))))), pos.clone())
        };
        let newcode = block! {
            Statement::Assign(Identifier::Local("unwrap-1".to_string()), None,
                              Expression::Typed(Type::Integer,Box::new(
                                  Expression::Unwrap(Box::new(
                                    Expression::Typed(Type::Integer,Box::new(
                                      Identifier::Local("e".to_string()).to_expression())))))),pos.clone()),
            Statement::Assign(Identifier::Local("d".to_string()), None,
                              Expression::Typed(Type::Integer,Box::new(
                                  Identifier::Local("unwrap-1".to_string()).to_expression())), pos.clone())
        };
        let mut ctr = 1usize;
        assert_eq!(newcode, unwrapify(&code, &mut ctr).unwrap());
    }

    #[test]
    fn unwrap_two_statements() {
        let pos0: SourceSpan = (0..0).into();
        let pos1: SourceSpan = (1..1).into();
        let code = block! {
            Statement::Assign(Identifier::Local("d".to_string()), None,
                              Expression::Typed(Type::Integer,Box::new(
                                Expression::Unwrap(Box::new(
                                  Expression::Typed(Type::Integer,Box::new(
                                    Identifier::Local("e".to_string()).to_expression())))))), pos0.clone()),
            Statement::Assign(Identifier::Local("f".to_string()), None,
                              Expression::Typed(Type::Integer, Box::new(
                                Expression::Unwrap(Box::new(
                                  Expression::Typed(Type::Integer,Box::new(
                                    Identifier::Local("g".to_string()).to_expression())))))), pos1.clone())
        };
        let newcode = block! {
            Statement::Assign(Identifier::Local("unwrap-1".to_string()), None,
                              Expression::Typed(Type::Integer,Box::new(
                                Expression::Unwrap(Box::new(
                                  Expression::Typed(Type::Integer,Box::new(
                                    Identifier::Local("e".to_string()).to_expression())))))), pos0.clone()),
            Statement::Assign(Identifier::Local("d".to_string()), None,
                              Expression::Typed(Type::Integer,Box::new(
                                Identifier::Local("unwrap-1".to_string()).to_expression())), pos0.clone()),
            Statement::Assign(Identifier::Local("unwrap-2".to_string()), None,
                              Expression::Typed(Type::Integer,Box::new(
                                 Expression::Unwrap(Box::new(
                                   Expression::Typed(Type::Integer,Box::new(
                                     Identifier::Local("g".to_string()).to_expression())))))), pos1.clone()),
            Statement::Assign(Identifier::Local("f".to_string()), None,
                              Expression::Typed(Type::Integer,Box::new(
                                Identifier::Local("unwrap-2".to_string()).to_expression())), pos1.clone())
        };
        let mut ctr = 1usize;
        assert_eq!(newcode, unwrapify(&code, &mut ctr).unwrap());
    }

    #[test]
    fn nested_statements() {
        let pos: SourceSpan = (0..0).into();
        let code = block! {
            Statement::Assign(Identifier::Local("d".to_string()), None,
                              Expression::Typed(Type::Integer, Box::new(
                                 Expression::Unwrap(Box::new(
                                   Expression::Typed(Type::Integer,Box::new(
                                     Expression::Unwrap(Box::new(
                                       Expression::Typed(Type::Integer,Box::new(
                                         Identifier::Local("e".to_string()).to_expression())))))))))), pos.clone())
        };
        let newcode = block! {
                    Statement::Assign(Identifier::Local("unwrap-1".to_string()), None,
                                      Expression::Typed(Type::Integer,Box::new(
                                        Expression::Unwrap(Box::new(
                                          Expression::Typed(Type::Integer,Box::new(
                                            Identifier::Local("e".to_string()).to_expression())))))), pos.clone() ),
                    Statement::Assign(Identifier::Local("unwrap-2".to_string()), None,
                                      Expression::Typed(Type::Integer,Box::new(
                                        Expression::Unwrap(Box::new(
                                        Expression::Typed(Type::Integer,Box::new(
                                          Identifier::Local("unwrap-1".to_string()).to_expression())))))), pos.clone() ),
                    Statement::Assign(Identifier::Local("d".to_string()), None,
                                      Expression::Typed(Type::Integer,Box::new(
                                        Identifier::Local("unwrap-2".to_string()).to_expression())), pos.clone())
        };
        let mut ctr = 1usize;
        assert_eq!(newcode, unwrapify(&code, &mut ctr).unwrap());
    }

    #[test]
    fn unwrap_typed_assign() {
        let pos: SourceSpan = (0..0).into();
        let code = block! {
            Statement::Assign(Identifier::Local("d".to_string()), None,
                              Expression::Typed(Type::Integer,
                              Box::new(Expression::Unwrap(Box::new(
                                Expression::Typed(Type::Integer,Box::new(
                                  Identifier::Local("e".to_string()).to_expression())))))), pos.clone())
        };
        let newcode = block! {
            Statement::Assign(Identifier::Local("unwrap-1".to_string()), None,
                              Expression::Typed(Type::Integer, Box::new(
                                  Expression::Unwrap(Box::new(
                                    Expression::Typed(Type::Integer, Box::new(
                                      Identifier::Local("e".to_string()).to_expression())))))), pos.clone()),
            Statement::Assign(Identifier::Local("d".to_string()), None,
                              Expression::Typed(Type::Integer, Box::new(
                                  Identifier::Local("unwrap-1".to_string()).to_expression())), pos.clone())
        };
        let mut ctr = 1usize;
        assert_eq!(newcode, unwrapify(&code, &mut ctr).unwrap());
    }
}
