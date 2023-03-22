use crate::expressions::Expression;
use crate::identifier::Identifier;
use crate::package::{Composition, OracleDef, OracleSig, PackageInstance};
use crate::statement::{CodeBlock, Statement};
use crate::transforms::samplify::SampleInfo;
use crate::types::Type;

use crate::writers::smt::exprs::{smt_to_string, SmtExpr, SmtIte, SmtLet};

use super::contexts::{GameContext, GlobalContext, OracleContext, PackageInstanceContext};
use super::exprs::{SmtAs, SmtEq2};
use super::{names, sorts};

pub struct CompositionSmtWriter<'a> {
    pub comp: &'a Composition,

    sample_info: &'a SampleInfo,
}

impl<'a> CompositionSmtWriter<'a> {
    pub fn new(comp: &'a Composition, samp: &'a SampleInfo) -> CompositionSmtWriter<'a> {
        CompositionSmtWriter {
            comp,
            sample_info: samp,
        }
    }

    fn get_game_context(&self) -> GameContext<'a> {
        GameContext::new(self.comp)
    }

    fn get_package_instance_context(&self, inst_name: &str) -> Option<PackageInstanceContext<'a>> {
        self.get_game_context().pkg_inst_ctx_by_name(inst_name)
    }

    fn get_oracle_context(&self, inst_name: &str, oracle_name: &str) -> Option<OracleContext<'a>> {
        self.get_package_instance_context(inst_name)?
            .oracle_ctx_by_name(oracle_name)
    }

    fn get_randomness(&self, sample_id: usize) -> Option<SmtExpr> {
        let game_context = self.get_game_context();
        let gamestate = (
            "select",
            names::var_globalstate_name(),
            names::var_state_length_name(),
        );

        game_context.smt_access_gamestate_rand(self.sample_info, gamestate, sample_id)
    }

    // builds a single (declare-datatype ...) expression for package instance `inst`
    fn smt_pkg_state(&self, inst: &PackageInstance) -> SmtExpr {
        self.get_package_instance_context(&inst.name)
            .expect(&format!(
                "game {} does not contain package instance with name {}",
                self.comp.name, &inst.name
            ))
            .smt_declare_pkgstate()
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

        states.push(
            self.get_game_context()
                .smt_declare_gamestate(self.sample_info),
        );

        states
    }

    pub fn smt_sort_return(&self, inst_name: &str, oracle_name: &str) -> SmtExpr {
        names::return_sort_name(&self.comp.name, inst_name, oracle_name).into()
    }

    pub fn smt_sort_composition_state(&self) -> SmtExpr {
        names::gamestate_sort_name(&self.comp.name).into()
    }

    fn smt_pkg_intermediate_state(&self, inst: &PackageInstance) -> Vec<SmtExpr> {
        let pkg_inst_ctx = self.get_package_instance_context(&inst.name).unwrap();

        let mut declares = vec![];

        for i in 0..inst.pkg.oracles.len() {
            let octx = pkg_inst_ctx.oracle_ctx_by_oracle_offs(i).unwrap();
            declares.push(octx.smt_declare_intermediate_states());
        }

        return declares;
    }

    fn smt_pkg_return(&self, inst: &PackageInstance) -> Vec<SmtExpr> {
        let pkg_inst_ctx = self.get_package_instance_context(&inst.name).unwrap();

        inst.get_oracle_sigs()
            .iter()
            .map(|osig| {
                pkg_inst_ctx
                    .oracle_ctx_by_name(&osig.name)
                    .unwrap()
                    .smt_declare_return()
            })
            .collect()
    }

    pub fn smt_composition_return(&self) -> Vec<SmtExpr> {
        self.comp
            .pkgs
            .clone()
            .iter()
            .flat_map(|inst| self.smt_pkg_return(inst))
            .collect()
    }

    pub fn smt_composition_intermediate_state(&self) -> Vec<SmtExpr> {
        self.comp
            .pkgs
            .clone()
            .iter()
            .flat_map(|inst| self.smt_pkg_intermediate_state(inst))
            .collect()
    }

    fn code_smt_helper(
        &self,
        block: CodeBlock,
        sig: &OracleSig,
        inst: &PackageInstance,
    ) -> SmtExpr {
        let PackageInstance {
            name: inst_name, ..
        } = inst;
        let OracleSig {
            tipe: oracle_return_tipe,
            name: oracle_name,
            ..
        } = sig;

        let game_context = self.get_game_context();
        let oracle_context = self.get_oracle_context(&inst.name, &sig.name).unwrap();

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
                    // (mk-return-{name} statevarname expr)
                    let var_gamestates = names::var_globalstate_name();
                    let old_gamestate = GlobalContext::smt_latest_gamestate();
                    let var_selfstate = names::var_selfstate_name();
                    let new_gamestate = game_context
                        .smt_update_gamestate_pkgstate(
                            old_gamestate,
                            self.sample_info,
                            &inst.name,
                            var_selfstate,
                        )
                        .unwrap();

                    let some_empty = ("mk-some", "mk-empty");
                    let var_state_len = names::var_state_length_name();
                    let body = oracle_context.smt_construct_return(
                        var_gamestates,
                        var_state_len,
                        some_empty,
                        "false",
                    );

                    game_context.smt_push_global_gamestate(new_gamestate, body)
                }
                Statement::Return(Some(expr)) => {
                    // (mk-return-{name} statevarname expr)

                    let var_gamestates = names::var_globalstate_name();
                    let old_gamestate = GlobalContext::smt_latest_gamestate();
                    let var_selfstate = names::var_selfstate_name();
                    let new_gamestate = game_context
                        .smt_update_gamestate_pkgstate(
                            old_gamestate,
                            self.sample_info,
                            &inst.name,
                            var_selfstate,
                        )
                        .unwrap();

                    let var_state_len = names::var_state_length_name();
                    let body = oracle_context.smt_construct_return(
                        var_gamestates,
                        var_state_len,
                        Expression::Some(Box::new(expr.clone())),
                        "false",
                    );

                    game_context.smt_push_global_gamestate(new_gamestate, body)
                }
                Statement::Abort => {
                    // mk-abort-{name}

                    let var_gamestates = names::var_globalstate_name();
                    let old_gamestate = GlobalContext::smt_latest_gamestate();
                    let var_selfstate = names::var_selfstate_name();
                    let var_state_len = names::var_state_length_name();

                    let new_gamestate = game_context
                        .smt_update_gamestate_pkgstate(
                            old_gamestate,
                            self.sample_info,
                            &inst.name,
                            var_selfstate,
                        )
                        .unwrap();

                    let none = Expression::None(oracle_return_tipe.clone());
                    let body = oracle_context.smt_construct_return(
                        var_gamestates,
                        var_state_len,
                        none,
                        "true",
                    );

                    game_context.smt_push_global_gamestate(new_gamestate, body)
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
                     *       // TODO update this to the new context-based helpers structure
                     *       comp_helper.smt_set(counter, counter+1, body)
                     * ))
                     * )
                     *
                     */
                    let sample_id = sample_id.expect("found a None sample_id");

                    // ctr is the current "i-th sampling for sample id sample_id"
                    let ctr = self.get_randomness(sample_id).unwrap_or_else(|| {
                        let max_known_sample_id = self.sample_info.count;
                        panic!(
                            "found sample id {} that exceeds highest expected {}",
                            sample_id, max_known_sample_id
                        )
                    });

                    let rand_fn_name = names::fn_sample_rand_name(&self.comp.name, tipe);

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

                    let cur_gamestate = GlobalContext::smt_latest_gamestate();
                    let new_gamestate = game_context
                        .smt_increment_gamestate_rand(cur_gamestate, self.sample_info, sample_id)
                        .unwrap();

                    SmtLet {
                        bindings,
                        body: game_context
                            .smt_overwrite_latest_global_gamestate(new_gamestate, result.unwrap()),
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

                            (ident.clone(), (format!("el{}", i + 1), expr.clone()).into())
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
                Statement::InvokeOracle { tipe: None, .. } => {
                    panic!("found an unresolved oracle invocation: {:#?}", stmt);
                }
                Statement::InvokeOracle {
                    id,
                    opt_idx,
                    name,
                    args,
                    target_inst_name: Some(target),
                    tipe: Some(_),
                } => {
                    let called_oracle_context = self.get_oracle_context(target, name).unwrap();
                    let this_oracle_context =
                        self.get_oracle_context(&inst.name, oracle_name).unwrap();

                    let game_context = self.get_game_context();
                    let then_body = game_context.smt_push_global_gamestate(
                        game_context
                            .smt_update_gamestate_pkgstate(
                                GlobalContext::smt_latest_gamestate(),
                                self.sample_info,
                                &inst.name,
                                names::var_selfstate_name(),
                            )
                            .unwrap(),
                        this_oracle_context.smt_construct_abort(
                            names::var_globalstate_name(),
                            names::var_state_length_name(),
                        ),
                    );

                    let let_bindings = vec![
                        (
                            names::var_globalstate_name(),
                            called_oracle_context.smt_access_return_state(names::var_ret_name()),
                        )
                            .into(),
                        (
                            names::var_state_length_name(),
                            called_oracle_context
                                .smt_access_return_state_length(names::var_ret_name()),
                        )
                            .into(),
                    ];

                    let smt_expr = SmtLet {
                        bindings: vec![(names::var_ret_name(), {
                            let mut cmdline = vec![
                                names::oracle_function_name(&self.comp.name, target, name).into(),
                                names::var_globalstate_name().into(),
                                names::var_state_length_name().into(),
                            ];

                            for arg in args {
                                cmdline.push(arg.clone().into())
                            }

                            SmtExpr::List(cmdline)
                        })],
                        body: SmtIte {
                            cond: called_oracle_context
                                .smt_access_return_is_abort(names::var_ret_name()),
                            then: SmtLet {
                                bindings: let_bindings.clone(),
                                body: then_body,
                            },
                            els: SmtLet {
                                bindings: {
                                    let mut bindings = let_bindings.clone();

                                    if id.ident() != "_" {
                                        bindings.push((
                                            id.ident(),
                                            SmtExpr::List(vec![
                                                SmtExpr::Atom("maybe-get".into()),
                                                called_oracle_context
                                                    .smt_access_return_value(names::var_ret_name()),
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
                        (
                            "store",
                            id.to_expression(),
                            opt_idx.clone().unwrap(),
                            smt_expr,
                        )
                            .into()
                    } else {
                        smt_expr.into()
                    }
                }
                Statement::Assign(ident, opt_idx, expr) => {
                    let inst_context = self.get_package_instance_context(&inst.name).unwrap();
                    let oracle_context = self.get_oracle_context(&inst.name, oracle_name).unwrap();

                    let (t, inner) = if let Expression::Typed(t, i) = expr {
                        (t.clone(), *i.clone())
                    } else {
                        unreachable!("we expect that this is typed")
                    };

                    //eprintln!(r#"DEBUG code_smt_helper Assign {expr:?} to identifier {ident:?}")"#);

                    // first build the unwrap expression, if we have to
                    let outexpr = if let Expression::Unwrap(inner) = &inner {
                        ("maybe-get", *inner.clone()).into()
                    } else {
                        expr.clone().into() // TODO maybe this should be inner??
                    };

                    // then build the table store smt expression, in case we have to
                    let outexpr = if let Some(idx) = opt_idx {
                        let oldvalue = match &ident {
                            &Identifier::State {
                                name,
                                pkg_inst_name: pkgname,
                                ..
                            } => {
                                //assert_eq!(pkgname, inst_name, "failed assertion: in an oracle in instance {inst_name} I found a state identifier with {pkgname}. I assumed these would always be equal.");
                                inst_context
                                    .smt_access_pkgstate(names::var_selfstate_name(), name)
                                    .unwrap()
                            }
                            Identifier::Local(_) => ident.to_expression().into(),
                            _ => {
                                unreachable!("")
                            }
                        };

                        ("store", oldvalue, idx.clone(), outexpr).into()
                    } else {
                        outexpr
                    };

                    // build the actual smt assignment
                    let smtout = match ident {
                        Identifier::State {
                            name,
                            pkg_inst_name: pkgname,
                            ..
                        } => {
                            //assert_eq!(pkgname, inst_name, "failed assertion: in an oracle in instance {inst_name} I found a state identifier with {pkgname}. I assumed these would always be equal.");
                            SmtLet {
                                bindings: vec![(
                                    names::var_selfstate_name(),
                                    inst_context
                                        .smt_update_pkgstate(
                                            names::var_selfstate_name(),
                                            name,
                                            outexpr,
                                        )
                                        .unwrap()
                                        .into(),
                                )],
                                body: result.unwrap(),
                            }
                        }

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
                    // TODO: are we sure we don't want to deconstruct `inner` here?
                    // it seems impossible to me that expr ever matches here,
                    // because above we make sure it's an Expression::Typed.
                    if let Expression::Unwrap(inner) = expr {
                        SmtIte {
                            cond: SmtEq2 {
                                lhs: *inner.clone(),
                                rhs: SmtAs {
                                    name: "mk-none".into(),
                                    tipe: Type::Maybe(Box::new(t)),
                                },
                            },
                            then: oracle_context.smt_construct_abort(
                                names::var_globalstate_name(),
                                names::var_state_length_name(),
                            ),
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

    fn smt_define_oracle_fn(&self, inst: &PackageInstance, def: &OracleDef) -> SmtExpr {
        let code = &def.code;
        let mut args = vec![
            (
                names::var_globalstate_name(),
                sorts::Array {
                    key: crate::types::Type::Integer,
                    value: names::gamestate_sort_name(&self.comp.name),
                },
            )
                .into(),
            (names::var_state_length_name(), crate::types::Type::Integer).into(),
        ];

        let rest_args = def.sig.args.iter().cloned().map(|arg| arg.into());
        args.extend(rest_args);

        let game_name = &self.comp.name;
        let inst_name = &inst.name;
        let oracle_name = &def.sig.name;

        let game_context = GameContext::new(&self.comp);

        (
            "define-fun",
            names::oracle_function_name(game_name, inst_name, oracle_name),
            SmtExpr::List(args.clone()),
            names::return_sort_name(game_name, inst_name, oracle_name),
            SmtLet {
                bindings: vec![(
                    names::var_selfstate_name(),
                    game_context
                        .smt_access_gamestate_pkgstate(
                            (
                                "select",
                                names::var_globalstate_name(),
                                names::var_state_length_name(),
                            ),
                            inst_name,
                        )
                        .unwrap(),
                )],
                body: self.code_smt_helper(code.clone(), &def.sig, inst),
            },
        )
            .into()
    }

    fn smt_pkg_code(&self, inst: &PackageInstance) -> Vec<SmtExpr> {
        inst.pkg
            .oracles
            .iter()
            .map(|def| self.smt_define_oracle_fn(inst, def))
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

    fn smt_composition_paramfuncs(&self) -> Vec<SmtExpr> {
        let fns = self
            .comp
            .consts
            .iter()
            .filter(|(_, tipe)| matches!(tipe, Type::Fn(_, _)));
        let comp_name = &self.comp.name;

        let mut funcs = vec![];

        for (name, tipe) in fns {
            let (arg_types, ret_type) = if let Type::Fn(arg_types, ret_type) = tipe {
                (arg_types, ret_type)
            } else {
                unreachable!()
            };

            let arg_types: SmtExpr = arg_types
                .iter()
                .map(|tipe| tipe.clone().into())
                .collect::<Vec<SmtExpr>>()
                .into();

            funcs.push(
                (
                    "declare-fun",
                    format!("__func-{comp_name}-{name}"),
                    arg_types,
                    (**ret_type).clone(),
                )
                    .into(),
            )
        }
        funcs.sort();
        funcs
    }

    fn smt_composition_randomness(&mut self) -> Vec<SmtExpr> {
        let mut result: Vec<_> = self
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
        result.sort();
        result
    }

    pub fn smt_composition_all(&mut self) -> Vec<SmtExpr> {
        let rand = self.smt_composition_randomness();
        let paramfuncs = self.smt_composition_paramfuncs();
        let state = self.smt_composition_state();
        let ret = self.smt_composition_return();
        let interm = self.smt_composition_intermediate_state();
        let code = self.smt_composition_code();

        rand.into_iter()
            .chain(paramfuncs.into_iter())
            .chain(interm.into_iter())
            .chain(state.into_iter())
            .chain(ret.into_iter())
            .chain(code.into_iter())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fmt::Write;
    use std::string::FromUtf8Error;
    use thiserror::Error;

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
        let mut str = String::new();

        write!(&mut str, "{out}").unwrap();

        assert_eq!(str, "(let ((x 42)\n)\n x)\n");

        Ok(())
    }
}
