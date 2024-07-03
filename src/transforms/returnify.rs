use crate::package::Composition;
use crate::statement::{CodeBlock, FilePosition, Statement};
use crate::types::Type;

#[derive(Clone, Debug)]
pub enum Error {
    MissingReturn {
        file_pos: FilePosition,
        pkg_inst_name: String,
        oracle_name: String,
    },
}

pub struct TransformNg;

impl<'a> super::GameTransform for TransformNg {
    type Err = Error;
    type Aux = ();

    fn transform_game(&self, game: &Composition) -> Result<(Composition, Self::Aux), Self::Err> {
        let insts: Result<Vec<_>, _> = game
            .pkgs
            .iter()
            .map(|inst| {
                let mut newinst = inst.clone();
                for (i, oracle) in newinst.pkg.oracles.clone().iter().enumerate() {
                    newinst.pkg.oracles[i].code = returnify(
                        &oracle.code,
                        oracle.sig.tipe == Type::Empty,
                        &inst.name,
                        &oracle.sig.name,
                    )?;
                }
                for (i, oracle) in newinst.pkg.split_oracles.clone().iter().enumerate() {
                    newinst.pkg.split_oracles[i].code = returnify(
                        &oracle.code,
                        oracle.sig.tipe == Type::Empty,
                        &inst.name,
                        &oracle.sig.name,
                    )?;
                }
                Ok(newinst)
            })
            .collect();
        Ok((
            Composition {
                pkgs: insts?,
                ..game.clone()
            },
            (),
        ))
    }
}

use super::GameTransform;

impl super::GameInstanceTransform for TransformNg {
    type Err = Error;
    type Aux = ();

    fn transform_game_instance(
        &self,
        instance: &crate::proof::GameInstance,
    ) -> Result<(crate::proof::GameInstance, Self::Aux), Self::Err> {
        let (game, _) = <Self as GameTransform>::transform_game(&self, instance.game())?;
        Ok((instance.with_other_game(game), ()))
    }
}

mod old {
    use super::super::Transformation as TransformationTrait;
    use super::{returnify, Error};
    use crate::package::Composition;
    use crate::types::Type;

    pub struct Transformation<'a>(pub &'a Composition);

    impl<'a> TransformationTrait for Transformation<'a> {
        type Err = Error;
        type Aux = ();

        fn transform(&self) -> Result<(Composition, ()), Error> {
            let insts: Result<Vec<_>, _> = self
                .0
                .pkgs
                .iter()
                .map(|inst| {
                    let mut newinst = inst.clone();
                    for (i, oracle) in newinst.pkg.oracles.clone().iter().enumerate() {
                        newinst.pkg.oracles[i].code = returnify(
                            &oracle.code,
                            oracle.sig.tipe == Type::Empty,
                            &inst.name,
                            &oracle.sig.name,
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
                (),
            ))
        }
    }
}

pub fn returnify(
    cb: &CodeBlock,
    none_ok: bool,
    pkg_inst_name: &str,
    oracle_name: &str,
) -> Result<CodeBlock, Error> {
    match cb.0.last() {
        Some(Statement::IfThenElse(expr, ifcode, elsecode, file_pos)) => {
            let mut retval = cb.0.clone();
            retval.pop();
            retval.push(Statement::IfThenElse(
                expr.clone(),
                returnify(ifcode, none_ok, pkg_inst_name, oracle_name)?,
                returnify(elsecode, none_ok, pkg_inst_name, oracle_name)?,
                file_pos.clone(),
            ));
            Ok(CodeBlock(retval))
        }
        Some(Statement::Return(_, _)) | Some(Statement::Abort(_)) => Ok(cb.clone()),
        Some(other) => {
            if !none_ok {
                panic!(
                    r#"
                Err(Error::MissingReturn {{
                    file_pos: other.file_pos(), # {:?}
                    oracle_name,                # {oracle_name}
                    pkg_inst_name,              # {pkg_inst_name}
                }})
                "#,
                    other.file_pos()
                )
            } else {
                let mut retval = cb.0.clone();
                retval.push(Statement::Return(None, other.file_pos().clone()));
                Ok(CodeBlock(retval))
            }
        }
        None => {
            if !none_ok {
                panic!(
                    r#"
                Err(Error::MissingReturn {{
                    file_pos: other.file_pos(), # 
                    oracle_name,                # {oracle_name}
                    pkg_inst_name,              # {pkg_inst_name}
                }})
                "#
                )
            } else {
                let mut retval = cb.0.clone();
                retval.push(Statement::Return(None, (0..1).into())); // TODO proper source span!
                Ok(CodeBlock(retval))
            }
        }
    }
}

/// Unit tests for returnify.
/// - First, all cases where blocks already end properly should be preserved:
///     preserves_return_none, preserves_return_some, preserves_abort
/// - Second, if a block ends in a non-return-type statement, a Return(None) is added
///     adds_return
/// - Finally, returnify should also consider ell branches of if-then-else
///     adds_if_return_with_branches, adds_else_return_with_branches
#[cfg(test)]
mod test {
    use miette::SourceSpan;

    use super::returnify;
    use crate::block;
    use crate::expressions::Expression;
    use crate::identifier::Identifier;
    use crate::statement::{CodeBlock, FilePosition, Statement};
    use crate::types::Type;

    #[test]
    fn preserves_return_none() {
        let file_pos: SourceSpan = (0..1).into();
        let code = block! {
            Statement::Sample(Identifier::new_scalar("d"), None, None, Type::Integer, file_pos.clone()),
            Statement::Return(None, file_pos)
        };
        assert_eq!(
            code,
            returnify(&code, true, "some_pkg_inst", "some_oracle").unwrap()
        );
    }

    #[test]
    fn preserves_return_some() {
        let file_pos: SourceSpan = (0..1).into();
        let code = block! {
            Statement::Sample(Identifier::new_scalar("d"), None, None, Type::Integer, file_pos.clone()),
            Statement::Return(Some(Expression::IntegerLiteral(5)), file_pos.clone())
        };
        assert_eq!(
            code,
            returnify(&code, true, "some_pkg_inst", "some_oracle").unwrap()
        );
    }

    #[test]
    fn preserves_abort() {
        let file_pos: SourceSpan = (0..1).into();
        let code = block! {
            Statement::Sample(Identifier::new_scalar("d"), None, None, Type::Integer, file_pos.clone()),
            Statement::Abort(file_pos)
        };
        assert_eq!(
            code,
            returnify(&code, true, "some_pkg_inst", "some_oracle").unwrap()
        );
    }

    #[test]
    fn adds_return() {
        let file_pos: SourceSpan = (0..1).into();
        let before = block! {
            Statement::Sample(Identifier::new_scalar("d"), None, None, Type::Integer, file_pos.clone())
        };
        let after = block! {
            Statement::Sample(Identifier::new_scalar("d"), None, None,  Type::Integer, file_pos.clone()),
            Statement::Return(None, file_pos.clone())
        };
        assert_eq!(
            after,
            returnify(&before, true, "some_pkg_inst", "some_oracle").unwrap()
        );
        assert_eq!(
            after,
            returnify(&after, true, "some_pkg_inst", "some_oracle").unwrap()
        );
    }

    #[test]
    fn adds_if_return_with_branches() {
        let file_pos: SourceSpan = (0..1).into();
        let before = block! {
            Statement::IfThenElse(
                Expression::new_equals(vec![&(Identifier::new_scalar("a").to_expression()),
                                            &(Identifier::new_scalar("a").to_expression())]),
                block!{
                    Statement::Sample(Identifier::new_scalar("d"), None, None, Type::Integer, file_pos.clone())
                },
                block!{
                    Statement::Sample(Identifier::new_scalar("e"), None, None, Type::Integer, file_pos.clone()),
                    Statement::Return(None, file_pos.clone())
                }, file_pos.clone())
        };
        let after = block! {
            Statement::IfThenElse(
                Expression::new_equals(vec![&(Identifier::new_scalar("a").to_expression()),
                                            &(Identifier::new_scalar("a").to_expression())]),
                block!{
                    Statement::Sample(Identifier::new_scalar("d"), None, None, Type::Integer, file_pos.clone()),
                    Statement::Return(None, file_pos.clone())
                },
                block!{
                    Statement::Sample(Identifier::new_scalar("e"), None, None, Type::Integer, file_pos.clone()),
                    Statement::Return(None, file_pos.clone())
                }, file_pos.clone())
        };
        assert_eq!(
            after,
            returnify(&before, true, "some_pkg_inst", "some_oracle").unwrap()
        );
        assert_eq!(
            after,
            returnify(&after, true, "some_pkg_inst", "some_oracle").unwrap()
        );
    }

    #[test]
    fn adds_else_return_with_branches() {
        let file_pos: SourceSpan = (0..1).into();
        let before = block! {
            Statement::IfThenElse(
                Expression::new_equals(vec![&(Identifier::new_scalar("a").to_expression()),
                                            &(Identifier::new_scalar("a").to_expression())]),
                block!{
                    Statement::Sample(Identifier::new_scalar("d"), None, None, Type::Integer, file_pos.clone())
                },
                block!{
                    Statement::Sample(Identifier::new_scalar("e"), None, None, Type::Integer, file_pos.clone())
                }, file_pos.clone())
        };
        let after = block! {
            Statement::IfThenElse(
                Expression::new_equals(vec![&(Identifier::new_scalar("a").to_expression()),
                                            &(Identifier::new_scalar("a").to_expression())]),
                block!{
                    Statement::Sample(Identifier::new_scalar("d"), None, None, Type::Integer, file_pos.clone()),
                    Statement::Return(None, file_pos.clone())
                },
                block!{
                    Statement::Sample(Identifier::new_scalar("e"), None, None, Type::Integer, file_pos.clone()),
                    Statement::Return(None, file_pos.clone())
                }, file_pos.clone())
        };
        assert_eq!(
            after,
            returnify(&before, true, "some_pkg_inst", "some_oracle").unwrap()
        );
        assert_eq!(
            after,
            returnify(&after, true, "some_pkg_inst", "some_oracle").unwrap()
        );
    }
}
