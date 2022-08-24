use std::collections::HashMap;

use crate::expressions::Expression;
use crate::identifier::Identifier;
use crate::package::{Composition, OracleSig, PackageInstance};
use crate::statement::{CodeBlock, Statement};
use crate::transforms::samplify::SampleInfo;
use crate::types::Type;

use crate::writers::smt::{
    exprs::{smt_to_string, SmtExpr, SmtIte, SmtLet, SspSmtVar},
    state_helpers::{SmtCompositionContext, SmtPackageState},
};

use super::exprs::{SmtAs, SmtEq2};

pub struct CompositionSmtWriter<'a> {
    pub comp: &'a Composition,

    sample_info: &'a SampleInfo,
    state_helpers: HashMap<String, SmtPackageState<'a>>,
    comp_helper: SmtCompositionContext<'a>,
}

impl<'a> CompositionSmtWriter<'a> {
    pub fn new(comp: &'a Composition, samp: &'a SampleInfo) -> CompositionSmtWriter<'a> {
        let mut csw = CompositionSmtWriter {
            comp,
            sample_info: samp,
            state_helpers: Default::default(),
            comp_helper: SmtCompositionContext::new(
                &comp.name,
                comp.pkgs.iter().map(|inst| &inst.name as &str).collect(),
                &comp.consts,
                samp,
            ),
        };

        for inst in &csw.comp.pkgs {
            csw.state_helpers.insert(
                inst.name.clone(),
                SmtPackageState::new(
                    &csw.comp.name,
                    &inst.name,
                    inst.pkg.state.clone(),
                    inst.pkg.params.clone(),
                ),
            );
        }

        csw
    }

    // returns the state helper for the package of a given package instance
    fn get_state_helper(&'a self, instname: &str) -> &'a SmtPackageState<'a> {
        self.state_helpers
            .get(instname)
            .unwrap_or_else(|| panic!("error looking up smt state helper: {}", instname))
    }

    fn get_randomness(&self, sample_id: u32) -> SmtExpr {
        (
            self.comp_helper.smt_accessor_rand(sample_id),
            SspSmtVar::CompositionContext,
        )
            .into()
    }

    // builds a single (declare-datatype ...) expression for package instance `inst`
    fn smt_pkg_state(&self, inst: &PackageInstance) -> SmtExpr {
        self.get_state_helper(&inst.name).smt_declare_datatype()
    }

    // build the (declare-datatype ...) expressions for all package states and the joint composition state
    pub fn smt_composition_state(&self) -> Vec<SmtExpr> {
        // 1. each package in composition
        let mut states: Vec<SmtExpr> = self
            .comp
            .pkgs
            .clone()
            .iter()
            .map(|pkg| self.smt_pkg_state(pkg))
            .collect();

        states.push(self.comp_helper.smt_declare_datatype());

        states
    }

    fn smt_pkg_return(&self, inst: &PackageInstance) -> Vec<SmtExpr> {
        let mut smts = vec![];

        for osig in inst.get_oracle_sigs() {
            let mut constructor = vec![
                SmtExpr::Atom(format!(
                    "mk-return-{}-{}-{}",
                    self.comp.name, inst.name, osig.name
                )),
                SmtExpr::List(vec![
                    SmtExpr::Atom(format!(
                        "return-{}-{}-{}-state",
                        self.comp.name, inst.name, osig.name
                    )),
                    self.comp_helper.smt_sort(),
                ]),
            ];

            if Type::Empty != osig.tipe {
                constructor.push(SmtExpr::List(vec![
                    SmtExpr::Atom(format!(
                        "return-{}-{}-{}-value",
                        self.comp.name, inst.name, osig.name
                    )),
                    osig.tipe.into(),
                ]));
            }

            smts.push(SmtExpr::List(vec![
                SmtExpr::Atom("declare-datatype".to_string()),
                SmtExpr::Atom(format!(
                    "Return_{}_{}_{}",
                    self.comp.name, inst.name, osig.name
                )),
                SmtExpr::List(vec![
                    SmtExpr::List(vec![SmtExpr::Atom(format!(
                        "mk-abort-{}-{}-{}",
                        self.comp.name, inst.name, osig.name
                    ))]),
                    SmtExpr::List(constructor),
                ]),
            ]))
        }
        smts
    }

    pub fn smt_composition_return(&self) -> Vec<SmtExpr> {
        self.comp
            .pkgs
            .clone()
            .iter()
            .flat_map(|inst| self.smt_pkg_return(inst))
            .collect()
    }

    fn code_smt_helper(
        &self,
        block: CodeBlock,
        sig: &OracleSig,
        inst: &PackageInstance,
    ) -> SmtExpr {
        let PackageInstance { name: pkgname, .. } = inst;

        let mut result = None;
        for stmt in block.0.iter().rev() {
            result = Some(match stmt {
                Statement::IfThenElse(cond, ifcode, elsecode) => SmtIte {
                    cond: cond.clone(),
                    then: self.code_smt_helper(ifcode.clone(), sig, inst),
                    els: self.code_smt_helper(elsecode.clone(), sig, inst),
                }
                .into(),
                Statement::Return(None) => {
                    // (mk-return-{name} statevarname)
                    (
                        SspSmtVar::OracleReturnConstructor {
                            compname: self.comp.name.clone(),
                            pkgname: pkgname.clone(),
                            oname: sig.name.clone(),
                        },
                        SspSmtVar::CompositionContext,
                    )
                        .into()
                }
                Statement::Return(Some(expr)) => {
                    // (mk-return-{name} statevarname expr)
                    self.comp_helper.smt_set_pkg_state(
                        &inst.name,
                        &SspSmtVar::SelfState.into(),
                        SmtExpr::List(vec![
                            SspSmtVar::OracleReturnConstructor {
                                compname: self.comp.name.clone(),
                                pkgname: pkgname.clone(),
                                oname: sig.name.clone(),
                            }
                            .into(),
                            SspSmtVar::CompositionContext.into(),
                            expr.clone().into(),
                        ]),
                    )
                }
                Statement::Abort => {
                    // mk-abort-{name}
                    SspSmtVar::OracleAbort {
                        compname: self.comp.name.clone(),
                        pkgname: pkgname.clone(),
                        oname: sig.name.clone(),
                    }
                    .into()
                    //SmtExpr::Atom(format!("mk-abort-{}-{}", pkgname, sig.name))
                }
                // TODO actually use the type that we sample to know how far to advance the randomness tape
                Statement::Sample(ident, opt_idx, sample_id, tipe) => {
                    /*
                     *   1. get counter
                     *   2. assign ident
                     *   3. overwrite state
                     *   4. continue
                     *
                     * let
                     *   ident = sample(ctr)
                     *   __global = mk-compositionState ( mk-randomndess-state (ctr + 1) ... )
                     *
                     *
                     * ,
                     *   let (ident = rand(access(counter)) (
                     *       comp_helper.smt_set(counter, counter+1, body)
                     * ))
                     * )
                     *
                     */
                    let sample_id = sample_id.expect("found a None sample_id");

                    let ctr = self.get_randomness(sample_id);

                    let rand_tipe: SmtExpr = tipe.clone().into();
                    let rand_fn_name = format!(
                        "__sample-rand-{}-{}",
                        self.comp.name,
                        smt_to_string(rand_tipe.clone())
                    );

                    let rand_val: SmtExpr =
                        (rand_fn_name, format!("{sample_id}"), ctr.clone()).into();

                    let new_val = if let Some(idx) = opt_idx {
                        (
                            "store",
                            ident.to_expression(),
                            idx.clone(),
                            rand_val.clone(),
                        )
                            .into()
                    } else {
                        rand_val
                    };

                    let bindings = vec![(ident.ident(), new_val)]
                        .into_iter()
                        .filter(|(x, _)| x != "_")
                        .collect();

                    SmtLet {
                        bindings,
                        body: self.comp_helper.smt_set_rand_ctr(
                            sample_id,
                            &("+", "1", ctr).into(),
                            result.unwrap(),
                        ),
                    }
                    .into()
                }
                Statement::Parse(idents, expr) => {
                    let bindings = idents
                        .iter()
                        .filter(|ident| ident.ident() != "_")
                        .enumerate()
                        .map(|(i, ident)| {
                            let ident = if let Identifier::Local(ident) = ident {
                                ident
                            } else {
                                unreachable!()
                            };

                            (
                                ident.clone(),
                                SmtExpr::List(vec![
                                    SmtExpr::Atom(format!("el{}", i + 1)),
                                    expr.clone().into(),
                                ]),
                            )
                        })
                        .collect();

                    SmtLet {
                        bindings,
                        body: result.unwrap(),
                    }
                    .into()
                }
                Statement::InvokeOracle {
                    target_inst_name: None,
                    ..
                } => {
                    panic!("found an unresolved oracle invocation: {:#?}", stmt);
                }
                Statement::InvokeOracle {
                    id,
                    opt_idx,
                    name,
                    args,
                    target_inst_name: Some(target),
                    tipe: _,
                } => {
                    let smt_expr = SmtLet {
                        bindings: vec![(smt_to_string(SspSmtVar::ReturnValue), {
                            let mut cmdline = vec![
                                SmtExpr::Atom(format!(
                                    "oracle-{}-{}-{}",
                                    self.comp.name, target, name
                                )),
                                SspSmtVar::CompositionContext.into(),
                            ];

                            for arg in args {
                                cmdline.push(arg.clone().into())
                            }

                            SmtExpr::List(cmdline)
                        })],
                        body: SmtIte {
                            cond: SmtEq2 {
                                lhs: SspSmtVar::ReturnValue,
                                rhs: SspSmtVar::OracleAbort {
                                    compname: self.comp.name.clone(),
                                    pkgname: target.clone(),
                                    oname: name.clone(),
                                },
                            },
                            then: SspSmtVar::OracleAbort {
                                compname: self.comp.name.clone(),
                                pkgname: pkgname.into(),
                                oname: sig.name.clone(),
                            },
                            els: SmtLet {
                                bindings: {
                                    let mut bindings = vec![(
                                        smt_to_string(SspSmtVar::CompositionContext),
                                        SmtExpr::List(vec![
                                            SmtExpr::Atom(format!(
                                                "return-{}-{}-{}-state",
                                                self.comp.name, target, name
                                            )),
                                            SspSmtVar::ReturnValue.into(),
                                        ]),
                                    )];

                                    if id.ident() != "_" {
                                        bindings.push((
                                            id.ident(),
                                            SmtExpr::List(vec![
                                                SmtExpr::Atom(format!(
                                                    "return-{}-{}-{}-value",
                                                    self.comp.name, target, name
                                                )),
                                                SspSmtVar::ReturnValue.into(),
                                            ]),
                                        ));
                                    }

                                    bindings
                                },
                                body: result.unwrap(),
                            },
                        },
                    };

                    if opt_idx.is_some() {
                        SmtExpr::List(vec![
                            SmtExpr::Atom("store".into()),
                            id.to_expression().into(),
                            opt_idx.clone().unwrap().into(),
                            smt_expr.into(),
                        ])
                    } else {
                        smt_expr.into()
                    }
                }
                Statement::Assign(ident, opt_idx, expr) => {
                    let (t, inner) = if let Expression::Typed(t, i) = expr {
                        (t.clone(), *i.clone())
                    } else {
                        unreachable!("we expect that this is typed")
                    };

                    // first build the unwrap expression, if we have to
                    let outexpr = if let Expression::Unwrap(inner) = &inner {
                        SmtExpr::List(vec![
                            SmtExpr::Atom("maybe-get".into()),
                            SmtExpr::Atom(smt_to_string(*inner.clone())),
                        ])
                    } else {
                        expr.clone().into()
                    };

                    // then build the table store smt expression, in case we have to
                    let outexpr = if let Some(idx) = opt_idx {
                        let oldvalue = match &ident {
                            &Identifier::State { name, pkgname, .. } => self
                                .get_state_helper(pkgname)
                                .smt_access(name, SspSmtVar::SelfState.into()),
                            Identifier::Local(_) => ident.to_expression().into(),
                            _ => {
                                unreachable!("")
                            }
                        };

                        SmtExpr::List(vec![
                            SmtExpr::Atom("store".into()),
                            oldvalue,
                            idx.clone().into(),
                            outexpr,
                        ])
                    } else {
                        outexpr
                    };

                    // build the actual smt assignment
                    let smtout = match ident {
                        Identifier::State { name, pkgname, .. } => SmtLet {
                            bindings: vec![(
                                smt_to_string(SspSmtVar::SelfState),
                                self.get_state_helper(pkgname).smt_set(name, &outexpr),
                            )],
                            body: result.unwrap(),
                        },

                        Identifier::Local(name) => SmtLet {
                            bindings: vec![(name.clone(), outexpr)]
                                .into_iter()
                                .filter(|(name, _)| name != "_:")
                                .collect(),
                            body: result.unwrap(),
                        },

                        _ => {
                            unreachable!("can't assign to {:#?}", ident)
                        }
                    };

                    // if it's an unwrap, also wrap it with the unwrap check.
                    if let Expression::Unwrap(inner) = expr {
                        SmtIte {
                            cond: SmtEq2 {
                                lhs: *inner.clone(),
                                rhs: SmtAs {
                                    name: "mk-none".into(),
                                    tipe: Type::Maybe(Box::new(t)),
                                },
                            },
                            then: SspSmtVar::OracleAbort {
                                compname: self.comp.name.clone(),
                                pkgname: pkgname.into(),
                                oname: sig.name.clone(),
                            },
                            els: smtout,
                        }
                        .into()
                    } else {
                        smtout.into()
                    }
                }
            });
        }
        result.unwrap()
    }

    /* example
        (define-fun
            stored_key_equals_k
            ((state_all State_composition) (k Key))
            Return_stored-key-equals-k
            (let
                (state_key (state-composition-key state_all))
                (mk-stored-key-equals-k
                    state-all
                    (=
                        (state-key-k state_key)
                        k
                    )
                )
            )
        )
    */

    fn smt_pkg_code(&self, inst: &PackageInstance) -> Vec<SmtExpr> {
        inst.pkg
            .oracles
            .iter()
            .map(|def| {
                let code = &def.code;
                let mut args = vec![SmtExpr::List(vec![
                    SspSmtVar::CompositionContext.into(),
                    self.comp_helper.smt_sort(),
                ])];

                for (name, tipe) in def.sig.args.clone() {
                    args.push(SmtExpr::List(vec![SmtExpr::Atom(name), tipe.into()]))
                }

                SmtExpr::List(vec![
                    SmtExpr::Atom(String::from("define-fun")),
                    SmtExpr::Atom(format!(
                        "oracle-{}-{}-{}",
                        self.comp.name, inst.name, def.sig.name
                    )),
                    SmtExpr::List(args),
                    SmtExpr::Atom(format!(
                        "Return_{}_{}_{}",
                        self.comp.name, inst.name, def.sig.name
                    )),
                    SmtLet {
                        bindings: vec![(
                            smt_to_string(SspSmtVar::SelfState),
                            self.comp_helper
                                .smt_access_pkg(&inst.name, SspSmtVar::CompositionContext.into()),
                        )],
                        body: self.code_smt_helper(code.clone(), &def.sig, inst),
                    }
                    .into(),
                ])
            })
            .collect()
    }

    fn smt_composition_code(&self) -> Vec<SmtExpr> {
        let comment = vec![SmtExpr::Comment(format!(
            "Composition of {}\n",
            self.comp.name
        ))];
        let ordered_pkgs = self.comp.ordered_pkgs();
        let code = ordered_pkgs.iter().flat_map(|inst| self.smt_pkg_code(inst));

        comment.into_iter().chain(code).collect()
    }

    fn smt_composition_randomness(&mut self) -> Vec<SmtExpr> {
        let mut result: Vec<SmtExpr> = self
            .sample_info
            .tipes
            .iter()
            .map(|tipe| {
                let tipeexpr: SmtExpr = tipe.clone().into();

                (
                    "declare-fun",
                    format!(
                        "__sample-rand-{}-{}",
                        self.comp.name,
                        smt_to_string(tipeexpr.clone())
                    ),
                    (SmtExpr::Atom("Int".into()), SmtExpr::Atom("Int".into())),
                    tipeexpr,
                )
                    .into()
            })
            .collect();

        let statehelper = SmtPackageState::new(
            &self.comp.name,
            "__randomness",
            (1..self.sample_info.count)
                .map(|ctr| (format!("ctr{}", ctr), Type::Integer))
                .collect(),
            vec![],
        );
        self.state_helpers
            .insert("__randomness".into(), statehelper.clone());
        result.push(statehelper.smt_declare_datatype());

        result
    }

    pub fn smt_composition_all(&mut self) -> Vec<SmtExpr> {
        //let rand = self.smt_composition_randomness();
        let state = self.smt_composition_state();
        let ret = self.smt_composition_return();
        let code = self.smt_composition_code();

        //rand.into_iter()
        //    .chain(state.into_iter())
        state
            .into_iter()
            .chain(ret.into_iter())
            .chain(code.into_iter())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::string::FromUtf8Error;
    use thiserror::Error;

    use crate::writers::smt::SmtFmt;

    #[derive(Error, Debug)]
    enum TestError {
        #[error("Error parsing the utf8: {0}")]
        Utf8DecodeError(#[from] FromUtf8Error),
        #[error("Error Writing: {0}")]
        WriteError(#[from] std::io::Error),
    }

    type TestResult = std::result::Result<(), TestError>;

    #[test]
    fn test_smtlet() -> TestResult {
        let l = SmtLet {
            bindings: vec![(
                "x".into(),
                Expression::IntegerLiteral(String::from("42")).into(),
            )],
            body: SmtExpr::Atom(String::from("x")),
        };

        let out: SmtExpr = l.into();
        let mut str = Vec::<u8>::new();
        out.write_smt_to(&mut str)?;

        assert_eq!(String::from_utf8(str)?, "(let ((x 42)) x)");

        Ok(())
    }
}
