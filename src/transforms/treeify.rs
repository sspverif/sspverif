use std::convert::Infallible;

use crate::package::Composition;
use crate::statement::{CodeBlock, FilePosition, Statement};

pub struct Transformation<'a>(pub &'a Composition);

impl<'a> super::Transformation for Transformation<'a> {
    type Err = Infallible;
    type Aux = ();

    fn transform(&self) -> Result<(Composition, ()), Infallible> {
        let insts = self.0.pkgs.iter().map(|inst| {
            let mut newinst = inst.clone();
            newinst.pkg.oracles.iter_mut().for_each(|oracle| {
                oracle.code = treeify(&oracle.code);
            });
            newinst
        });
        Ok((
            Composition {
                pkgs: insts.collect(),
                ..self.0.clone()
            },
            (),
        ))
    }
}

fn treeify(cb: &CodeBlock) -> CodeBlock {
    let mut before: Vec<Statement> = vec![];
    let mut after: Vec<Statement> = vec![];

    let mut ite_stmt = None;

    for elem in &cb.0 {
        match &*elem {
            Statement::IfThenElse(cond_, CodeBlock(ifcode_), CodeBlock(elsecode_), file_pos) => {
                if ite_stmt.is_none() {
                    ite_stmt = Some(elem.clone());
                } else {
                    after.push(elem.clone());
                }
            }
            Statement::For(ident, from, to, code, file_pos) => {
                let new_elem = Statement::For(
                    ident.clone(),
                    from.clone(),
                    to.clone(),
                    treeify(code),
                    file_pos.clone(),
                );

                if ite_stmt.is_none() {
                    before.push(new_elem);
                } else {
                    after.push(new_elem);
                }
            }
            _ => {
                if ite_stmt.is_none() {
                    before.push(elem.clone());
                } else {
                    after.push(elem.clone());
                }
            }
        }
    }

    if let Some(Statement::IfThenElse(
        ref cond,
        ref mut ifcode,
        ref mut elsecode,
        ref ite_file_pos,
    )) = &mut ite_stmt
    {
        let last_file_pos = after
            .last()
            .map(|stmt| stmt.file_pos())
            .or(Some(ite_file_pos))
            .unwrap();

        let block_file_pos = FilePosition::new(
            ite_file_pos.file_name().to_string(),
            ite_file_pos.line_start(),
            last_file_pos.line_end(),
        );

        ifcode.0.append(&mut after.clone());
        elsecode.0.append(&mut after.clone());

        before.push(Statement::IfThenElse(
            cond.clone(),
            treeify(ifcode),
            treeify(elsecode),
            block_file_pos,
        ));

        CodeBlock(before)
    } else {
        cb.clone()
    }
}

#[cfg(test)]
mod treeify_fn_test {
    use crate::expressions::Expression;
    use crate::identifier::Identifier;
    use crate::statement::{CodeBlock, FilePosition, Statement};

    use super::treeify;

    #[test]
    fn nothing_happens_without_if() {
        let file_pos = FilePosition::new("test_file".to_string(), 0, 1);
        let cb = CodeBlock(vec![Statement::Return(None, file_pos)]);

        assert_eq!(cb.clone(), treeify(&cb));
    }
    #[test]
    fn treeify_one_sided_if_depth_1() {
        let file_pos_0 = FilePosition::new("test_file".to_string(), 0, 1);
        let file_pos_1 = FilePosition::new("test_file".to_string(), 1, 1);
        let file_pos_2 = FilePosition::new("test_file".to_string(), 2, 2);
        let x = Identifier::new_scalar("x");
        let y = Identifier::new_scalar("y");
        let before = CodeBlock(vec![
            Statement::IfThenElse(
                y.to_expression(),
                CodeBlock(vec![Statement::Assign(
                    x.clone(),
                    None,
                    Expression::IntegerLiteral(4),
                    file_pos_1.clone(),
                )]),
                CodeBlock(vec![]),
                file_pos_0.clone(),
            ),
            Statement::Return(Some(x.to_expression()), file_pos_2.clone()),
        ]);

        let file_pos_0_new = FilePosition::new("test_file".to_string(), 0, 2);

        let after = CodeBlock(vec![Statement::IfThenElse(
            y.to_expression(),
            CodeBlock(vec![
                Statement::Assign(x.clone(), None, Expression::IntegerLiteral(4), file_pos_1),
                Statement::Return(Some(x.to_expression()), file_pos_2.clone()),
            ]),
            CodeBlock(vec![Statement::Return(Some(x.to_expression()), file_pos_2)]),
            file_pos_0_new,
        )]);

        assert_eq!(after.clone(), treeify(&before));

        // make sure it's idempotent
        assert_eq!(after.clone(), treeify(&after));
    }

    #[test]
    fn treeify_one_sided_if_depth_2() {
        let file_pos_outerif = FilePosition::new("test_file".to_string(), 0, 2);
        let file_pos_innerif = FilePosition::new("test_file".to_string(), 1, 2);
        let file_pos_assign = FilePosition::new("test_file".to_string(), 2, 2);
        let file_pos_return = FilePosition::new("test_file".to_string(), 3, 3);

        /*
         *
         * 0: if y:
         * 1:   if z:
         * 2:     x = 42
         * 3: return x
         *
         * becomes
         *
         * 0: if y: (0..3)
         * 1:   if z: (1..3)
         * 2:     x = 42 (2)
         * 3:     return x (3)
         * 4:   else return x(3)
         * 5: else return x(3)
         *
         *
         * */

        let x = Identifier::new_scalar("x");
        let y = Identifier::new_scalar("y");
        let z = Identifier::new_scalar("z");
        let before = CodeBlock(vec![
            Statement::IfThenElse(
                y.to_expression(),
                CodeBlock(vec![Statement::IfThenElse(
                    z.to_expression(),
                    CodeBlock(vec![Statement::Assign(
                        x.clone(),
                        None,
                        Expression::IntegerLiteral(42),
                        file_pos_assign.clone(),
                    )]),
                    CodeBlock(vec![]),
                    file_pos_innerif,
                )]),
                CodeBlock(vec![]),
                file_pos_outerif,
            ),
            Statement::Return(Some(x.to_expression()), file_pos_return.clone()),
        ]);

        let file_pos_outerif_new = FilePosition::new("test_file".to_string(), 0, 3);
        let file_pos_innerif_new = FilePosition::new("test_file".to_string(), 1, 3);

        let after = CodeBlock(vec![Statement::IfThenElse(
            y.to_expression(),
            CodeBlock(vec![Statement::IfThenElse(
                z.to_expression(),
                CodeBlock(vec![
                    Statement::Assign(
                        x.clone(),
                        None,
                        Expression::IntegerLiteral(42),
                        file_pos_assign,
                    ),
                    Statement::Return(Some(x.to_expression()), file_pos_return.clone()),
                ]),
                CodeBlock(vec![Statement::Return(
                    Some(x.to_expression()),
                    file_pos_return.clone(),
                )]),
                file_pos_innerif_new,
            )]),
            CodeBlock(vec![Statement::Return(
                Some(x.to_expression()),
                file_pos_return.clone(),
            )]),
            file_pos_outerif_new,
        )]);

        assert_eq!(after.clone(), treeify(&before));

        // make sure it's idempotent
        assert_eq!(after.clone(), treeify(&after));
    }

    #[test]
    fn treeify_subsequent_ifs() {
        let file_pos_firstif = FilePosition::new("test_file".to_string(), 0, 3);
        let file_pos_secondif = FilePosition::new("test_file".to_string(), 4, 7);
        let file_pos_firstifassign = FilePosition::new("test_file".to_string(), 1, 1);
        let file_pos_firstselseassign = FilePosition::new("test_file".to_string(), 3, 3);
        let file_pos_secondifassign = FilePosition::new("test_file".to_string(), 5, 5);
        let file_pos_secondselseassign = FilePosition::new("test_file".to_string(), 7, 7);
        let file_pos_return = FilePosition::new("test_file".to_string(), 8, 8);

        let file_pos_firstif_new = FilePosition::new("test_file".to_string(), 0, 8);
        let file_pos_second_new = FilePosition::new("test_file".to_string(), 4, 8);

        /*
         * if y: (0..3)
         *   x = 1 (1)
         * else:
         *   x = 2 (3)
         * if z: (4..7)
         *   x = 3 (5)
         * else:
         *   x = 4 (7)
         * return x (8)
         *
         * becomes:
         *
         * if y: (0..8)
         *   x = 1 (1)
         *   if z: (4..8)
         *     x = 3 (5)
         *     return x (8)
         *   else:
         *     x = 4 (7)
         *     return x (8)
         * else:
         *
         *
         *
         *
         *
         *
         *
         *
         * */

        let x = Identifier::new_scalar("x");
        let y = Identifier::new_scalar("y");
        let z = Identifier::new_scalar("z");
        let before = CodeBlock(vec![
            Statement::IfThenElse(
                y.to_expression(),
                CodeBlock(vec![Statement::Assign(
                    x.clone(),
                    None,
                    Expression::IntegerLiteral(1),
                    file_pos_firstifassign.clone(),
                )]),
                CodeBlock(vec![Statement::Assign(
                    x.clone(),
                    None,
                    Expression::IntegerLiteral(2),
                    file_pos_firstselseassign.clone(),
                )]),
                file_pos_firstif.clone(),
            ),
            Statement::IfThenElse(
                z.to_expression(),
                CodeBlock(vec![Statement::Assign(
                    x.clone(),
                    None,
                    Expression::IntegerLiteral(3),
                    file_pos_secondifassign.clone(),
                )]),
                CodeBlock(vec![Statement::Assign(
                    x.clone(),
                    None,
                    Expression::IntegerLiteral(4),
                    file_pos_secondselseassign.clone(),
                )]),
                file_pos_secondif.clone(),
            ),
            Statement::Return(Some(x.to_expression()), file_pos_return.clone()),
        ]);

        let after = CodeBlock(vec![Statement::IfThenElse(
            y.to_expression(),
            CodeBlock(vec![
                Statement::Assign(
                    x.clone(),
                    None,
                    Expression::IntegerLiteral(1),
                    file_pos_firstifassign.clone(),
                ),
                Statement::IfThenElse(
                    z.to_expression(),
                    CodeBlock(vec![
                        Statement::Assign(
                            x.clone(),
                            None,
                            Expression::IntegerLiteral(3),
                            file_pos_secondifassign.clone(),
                        ),
                        Statement::Return(Some(x.to_expression()), file_pos_return.clone()),
                    ]),
                    CodeBlock(vec![
                        Statement::Assign(
                            x.clone(),
                            None,
                            Expression::IntegerLiteral(4),
                            file_pos_secondselseassign.clone(),
                        ),
                        Statement::Return(Some(x.to_expression()), file_pos_return.clone()),
                    ]),
                    file_pos_second_new.clone(),
                ),
            ]),
            CodeBlock(vec![
                Statement::Assign(
                    x.clone(),
                    None,
                    Expression::IntegerLiteral(2),
                    file_pos_firstselseassign.clone(),
                ),
                Statement::IfThenElse(
                    z.to_expression(),
                    CodeBlock(vec![
                        Statement::Assign(
                            x.clone(),
                            None,
                            Expression::IntegerLiteral(3),
                            file_pos_secondifassign.clone(),
                        ),
                        Statement::Return(Some(x.to_expression()), file_pos_return.clone()),
                    ]),
                    CodeBlock(vec![
                        Statement::Assign(
                            x.clone(),
                            None,
                            Expression::IntegerLiteral(4),
                            file_pos_secondselseassign.clone(),
                        ),
                        Statement::Return(Some(x.to_expression()), file_pos_return.clone()),
                    ]),
                    file_pos_second_new.clone(),
                ),
            ]),
            file_pos_firstif_new,
        )]);

        assert_eq!(after.clone(), treeify(&before));

        // make sure it's idempotent
        assert_eq!(after.clone(), treeify(&after));
    }
}

// #[cfg(test)]
// mod treeify_transform_test {
//     use super::super::Transformation as TTransformation;
//     use super::Transformation;
//
//     use crate::expressions::Expression;
//     use crate::package::{Composition, Edge, Export};
//     use crate::testdata::{keypkg, modprf};
//     use std::collections::HashMap;
//
//     #[test]
//     fn runs_for_all_packages_and_oracles() {
//         let mut params = HashMap::new();
//         params.insert(
//             "n".to_string(),
//             Expression::IntegerLiteral("256".to_string()),
//         );
//
//         let key_real_pkg = keypkg::key_pkg(&params);
//         let mod_prf_real_pkg = modprf::mod_prf(&params);
//
//         let mod_prf_game = Composition {
//             pkgs: vec![key_real_pkg.clone(), mod_prf_real_pkg.clone()],
//             edges: vec![Edge(1, 0, key_real_pkg.pkg.clone().oracles[1].sig.clone())],
//             exports: vec![
//                 Export(0, key_real_pkg.pkg.clone().oracles[0].sig.clone()),
//                 Export(1, mod_prf_real_pkg.pkg.clone().oracles[0].sig.clone()),
//             ],
//             name: "real".to_string(),
//             consts: vec![],
//             split_exports: vec![],
//         };
//
//         let transform = Transformation(&mod_prf_game);
//         let (tranformed, _) = transform.transform().expect("error when transforming");
//
//         assert_eq!(
//             mod_prf_game.pkgs[0].pkg.oracles[0],
//             tranformed.pkgs[0].pkg.oracles[0]
//         );
//         assert_ne!(
//             mod_prf_game.pkgs[0].pkg.oracles[1],
//             tranformed.pkgs[0].pkg.oracles[1]
//         );
//         assert_eq!(
//             mod_prf_game.pkgs[1].pkg.oracles[0],
//             tranformed.pkgs[1].pkg.oracles[0]
//         );
//     }
// }
