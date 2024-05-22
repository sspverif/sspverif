use super::exprs::SmtExpr;
use super::patterns::{GlobalStatePattern, PackageStatePattern, SelfStatePattern};
use crate::expressions::Expression;
use crate::identifier::pkg_ident::PackageIdentifier;
use crate::identifier::pkg_inst_ident::PackageInstanceIdentifier;
use crate::identifier::{GameInstanceConst, Identifier, PackageConst, PackageState};
use crate::types::Type;

impl From<Expression> for SmtExpr {
    fn from(expr: Expression) -> SmtExpr {
        //eprintln!("DEBUG expr->smt: {expr:?}");
        match expr {
            Expression::EmptyTable(t) => {
                if let Type::Table(idxtipe, valtipe) = t {
                    (
                        (
                            "as",
                            "const",
                            ("Array", *idxtipe, Type::Maybe(valtipe.clone())),
                        ),
                        ("as", "mk-none", Type::Maybe(valtipe)),
                    )
                        .into()
                } else {
                    panic!("Empty table of type {:?}", t)
                }
            }
            Expression::Typed(_t, inner) => SmtExpr::from(*inner),
            Expression::Unwrap(_inner) => {
                panic!("unwrap expressions need to be on the right hand side of an assign!");
                // TODO find a better way to present that error to the user.
            }
            Expression::Some(inner) => {
                SmtExpr::List(vec![SmtExpr::Atom("mk-some".into()), SmtExpr::from(*inner)])
            }
            Expression::None(inner) => SmtExpr::List(vec![
                SmtExpr::Atom("as".into()),
                SmtExpr::Atom("mk-none".into()),
                Type::Maybe(Box::new(inner)).into(),
            ]),
            Expression::StringLiteral(litname) => SmtExpr::Atom(format!("\"{}\"", litname)),
            Expression::BooleanLiteral(litname) => SmtExpr::Atom(litname),
            Expression::IntegerLiteral(litname) => SmtExpr::Atom(format!("{litname}")),
            Expression::Equals(exprs) => {
                let mut acc = vec![SmtExpr::Atom("=".to_string())];
                for expr in exprs {
                    acc.push(expr.clone().into());
                }

                SmtExpr::List(acc)
            }
            Expression::Add(lhs, rhs) => SmtExpr::List(vec![
                SmtExpr::Atom("+".to_string()),
                SmtExpr::from(*lhs),
                SmtExpr::from(*rhs),
            ]),
            Expression::Sub(lhs, rhs) => SmtExpr::List(vec![
                SmtExpr::Atom("-".to_string()),
                SmtExpr::from(*lhs),
                SmtExpr::from(*rhs),
            ]),
            Expression::Mul(lhs, rhs) => SmtExpr::List(vec![
                SmtExpr::Atom("*".to_string()),
                SmtExpr::from(*lhs),
                SmtExpr::from(*rhs),
            ]),
            Expression::Div(lhs, rhs) => SmtExpr::List(vec![
                SmtExpr::Atom("/".to_string()),
                SmtExpr::from(*lhs),
                SmtExpr::from(*rhs),
            ]),
            Expression::Not(expr) => {
                SmtExpr::List(vec![SmtExpr::Atom("not".to_string()), (*expr).into()])
            }
            Expression::And(vals) => SmtExpr::List({
                let mut list = vec![SmtExpr::Atom("and".to_owned())];
                for val in vals {
                    list.push(SmtExpr::from(val))
                }
                list
            }),
            Expression::Or(vals) => SmtExpr::List({
                let mut list = vec![SmtExpr::Atom("or".to_owned())];
                for val in vals {
                    list.push(SmtExpr::from(val))
                }
                list
            }),
            Expression::Xor(vals) => SmtExpr::List({
                let mut list = vec![SmtExpr::Atom("xor".to_owned())];
                for val in vals {
                    list.push(SmtExpr::from(val))
                }
                list
            }),
            Expression::Identifier(Identifier::PackageIdentifier(pkg_ident)) => {
                pkg_ident.ident_ref().into()
            }
            Expression::Identifier(Identifier::Scalar(identname)) => {
                panic! {"Found a scalar {:} which should have been removed by varspecify at this point", identname}
            }
            Expression::Identifier(Identifier::Local(identname)) => SmtExpr::Atom(identname),

            // TODO
            // I would love to use PackageStatePattern here, but in order to use the access
            // method, we need the Package, which we don't have here. We also need the type of
            // the variable. All this means we'd need a lot more context. The only way I see
            // how to introduce the context here withing the constraints of the Into trait
            // would be to have all the information inside the Identifier, ideally as
            // references.
            //
            // Having them as references would mean that Identifier gets a lifetime, and by
            // extension also Expression and probably Statement. This sounds like it would be
            // pretty cumbersome, but maybe necessary for a clean structure.
            //
            // For now I'll leave it be.
            Expression::Identifier(Identifier::State(PackageState {
                name: ident_name,
                ref game_inst_name,
                ref pkg_inst_name,
                ..
            })) => (
                format!("state-{game_inst_name}-{pkg_inst_name}-{ident_name}"),
                &SelfStatePattern,
            )
                .into(),
            Expression::Identifier(Identifier::GameInstanceConst(GameInstanceConst {
                game_inst_name,
                name_in_comp,
                ..
            }))
            | Expression::Identifier(Identifier::Parameter(PackageConst {
                name_in_comp,
                game_inst_name,
                ..
            })) => (
                format!("composition-param-{game_inst_name}-{name_in_comp}"),
                &GlobalStatePattern,
            )
                .into(),
            Expression::Bot => SmtExpr::Atom("bot".to_string()),
            Expression::TableAccess(table, index) => SmtExpr::List(vec![
                SmtExpr::Atom("select".into()),
                table.to_expression().into(),
                (*index).into(),
            ]),
            Expression::Tuple(exprs) => {
                let mut l = vec![SmtExpr::Atom(format!("mk-tuple{}", exprs.len()))];

                for expr in exprs {
                    l.push(expr.into())
                }

                SmtExpr::List(l)
            }
            Expression::FnCall(
                Identifier::Parameter(PackageConst {
                    name_in_comp: name,
                    game_inst_name,
                    ..
                }),
                args,
            ) => {
                let fn_name = format!("__func-{game_inst_name}-{name}");
                let mut call = vec![SmtExpr::Atom(fn_name)];

                for expr in args {
                    call.push(expr.into());
                }

                SmtExpr::List(call)
            }
            Expression::FnCall(id, exprs) => {
                let mut call = vec![SmtExpr::Atom(id.ident())];

                for expr in exprs {
                    call.push(expr.into());
                }

                SmtExpr::List(call)
            }
            Expression::List(inner) => {
                let t = if let Expression::Typed(t, _) = inner[0].clone() {
                    Some(t)
                } else {
                    None
                };

                let t = t.unwrap();

                let nil = SmtExpr::List(vec![
                    SmtExpr::Atom("as".to_owned()),
                    SmtExpr::Atom("nil".to_owned()),
                    SmtExpr::List(vec![SmtExpr::Atom("List".to_owned()), t.into()]),
                ]);

                let mut lst = nil;

                for el in inner.iter().rev() {
                    lst =
                        SmtExpr::List(vec![SmtExpr::Atom("insert".into()), el.clone().into(), lst])
                }

                lst
            }
            Expression::Set(inner) => {
                let t = if let Expression::Typed(t, _) = inner[0].clone() {
                    Some(t)
                } else {
                    None
                };

                let t = t.unwrap();

                let empty_set = SmtExpr::List(vec![
                    SmtExpr::List(vec![
                        SmtExpr::Atom("as".to_owned()),
                        SmtExpr::Atom("const".to_owned()),
                        SmtExpr::List(vec![SmtExpr::Atom("Set".to_owned()), t.into()]),
                    ]),
                    SmtExpr::Atom("false".to_string()),
                ]);

                let mut set = empty_set;

                for el in inner.iter().rev() {
                    set = SmtExpr::List(vec![
                        SmtExpr::Atom("store".into()),
                        set,
                        el.clone().into(),
                        SmtExpr::Atom("true".to_string()),
                    ])
                }

                set
            }
            _ => {
                panic!("not implemented: {:?}", expr);
            }
        }
    }
}
