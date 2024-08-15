use crate::expressions::Expression;
use crate::identifier::Identifier;
use crate::package::{OracleDef, PackageInstance};
use crate::proof::GameInstance;
use crate::split::{SplitPath, SplitType};
use crate::statement::{CodeBlock, Statement};
use crate::transforms::samplify::SampleInfo;
use crate::transforms::split_partial::SplitInfo;
use crate::types::Type;

use crate::writers::smt::exprs::{smt_to_string, SmtExpr, SmtIte, SmtLet};
use crate::writers::smt::partials::into_partial_dtypes;
use crate::writers::smt::patterns::{PartialReturnConstructor, PartialReturnPattern};

use super::contexts::{
    GameInstanceContext, GenericOracleContext, OracleContext, PackageInstanceContext,
    SplitOracleContext,
};
use super::exprs::{SmtAs, SmtEq2};
use super::names;
use super::partials::PartialsDatatype;
use super::patterns::{
    declare_datatype, FunctionPattern, GlobalStatePattern, IntermediateStateConstructor,
    IntermediateStatePattern, IntermediateStateSelector, OraclePattern, PartialOraclePattern,
    ReturnValue, ReturnValueSelector, SelfStatePattern, VariablePattern,
};

use super::patterns;
use patterns::DatastructurePattern;

pub struct CompositionSmtWriter<'a> {
    game_inst: &'a GameInstance,

    sample_info: &'a SampleInfo,
    split_info: &'a SplitInfo,
}

impl<'a> CompositionSmtWriter<'a> {
    pub fn new(
        game_inst: &'a GameInstance,
        sample_info: &'a SampleInfo,
        split_info: &'a SplitInfo,
    ) -> CompositionSmtWriter<'a> {
        CompositionSmtWriter {
            game_inst,
            sample_info,
            split_info,
        }
    }

    fn context(&self) -> GameInstanceContext<'a> {
        GameInstanceContext::new(self.game_inst)
    }

    fn get_package_instance_context(&self, inst_name: &str) -> Option<PackageInstanceContext<'a>> {
        self.context().pkg_inst_ctx_by_name(inst_name)
    }

    fn get_oracle_context(&self, inst_name: &str, oracle_name: &str) -> Option<OracleContext<'a>> {
        self.get_package_instance_context(inst_name)?
            .oracle_ctx_by_name(oracle_name)
    }

    fn get_randomness<GS: Into<SmtExpr>>(
        &self,
        gamestate: GS,
        sample_id: usize,
    ) -> Option<SmtExpr> {
        let game_context = self.context();

        game_context.smt_access_gamestate_rand(self.sample_info, gamestate, sample_id)
    }

    // builds a single (declare-datatype ...) expression for package instance `inst`
    fn smt_pkg_state(&self, inst: &PackageInstance) -> SmtExpr {
        self.get_package_instance_context(&inst.name)
            .unwrap_or_else(|| {
                panic!(
                    "game {} does not contain package instance with name {}",
                    self.game_inst.game().name,
                    &inst.name
                )
            })
            .smt_declare_pkgstate()
    }

    // build the (declare-datatype ...) expressions for all package states and the joint composition state
    pub fn smt_composition_state(&self) -> Vec<SmtExpr> {
        let mut states: Vec<SmtExpr> = self
            .context()
            .game()
            .pkgs
            .iter()
            .map(|pkg| self.smt_pkg_state(pkg))
            .collect();

        states.push(self.context().smt_declare_gamestate(self.sample_info));

        states
    }

    pub fn smt_sort_return(&self, inst_name: &str, oracle_name: &str) -> SmtExpr {
        names::return_sort_name(&self.context().game().name, inst_name, oracle_name).into()
    }

    pub fn smt_sort_composition_state(&self) -> SmtExpr {
        names::gamestate_sort_name(&self.context().game().name).into()
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
        self.context()
            .game()
            .pkgs
            .clone()
            .iter()
            .flat_map(|inst| self.smt_pkg_return(inst))
            .collect()
    }

    fn smt_codeblock_nonsplit(&self, oracle_ctx: &OracleContext, block: CodeBlock) -> SmtExpr {
        let game_inst_ctx = self.context();
        let game_inst = game_inst_ctx.game_inst();
        let game = game_inst_ctx.game();

        let mut stmts = block.0.iter().rev();

        let innermost = stmts.next();
        let mut result = self.smt_build_innermost_nonsplit(oracle_ctx, innermost.unwrap());

        for stmt in stmts {
            result = match stmt {
                Statement::IfThenElse(cond, ifcode, elsecode, _) => SmtIte {
                    cond: cond.clone(),
                    then: self.smt_codeblock_nonsplit(oracle_ctx, ifcode.clone()),
                    els: self.smt_codeblock_nonsplit(oracle_ctx, elsecode.clone()),
                }
                .into(),
                Statement::Return(None, _) => {
                    // (mk-return-{name} statevarname expr)
                    self.smt_build_return_none_nonsplit(oracle_ctx)
                }
                Statement::Return(Some(expr), _) => {
                    // (mk-return-{name} statevarname expr)
                    self.smt_build_return_some_nonsplit(oracle_ctx, expr)
                }
                Statement::Abort(_) => {
                    // mk-abort-{name}
                    self.smt_build_abort(oracle_ctx)
                }
                Statement::For(_, _, _, _, _) => {
                    let game_inst_name = game_inst.name();
                    let _game_name = game.name();
                    let pkg_inst_name = &oracle_ctx.pkg_inst_ctx().pkg_inst().name;
                    let pkg_name = &oracle_ctx.pkg_inst_ctx().pkg_inst().pkg.name;
                    let oracle_name = oracle_ctx.oracle_name();
                    unreachable!("found a for statement in the smt writer stage. \
                                 this should have been eliminated by now and can't be handled here. \
                                 game:{game_inst_name}(game_name) pkg:{pkg_inst_name}({pkg_name}) oracle:{oracle_name}", game_inst_name = game_inst_name, pkg_inst_name = pkg_inst_name, pkg_name = pkg_name, oracle_name = oracle_name)
                }
                // TODO actually use the type that we sample to know how far to advance the randomness tape
                Statement::Sample(ident, opt_idx, sample_id, tipe, _) => {
                    self.smt_build_sample(oracle_ctx, result, ident, opt_idx, sample_id, tipe)
                }
                Statement::Parse(idents, expr, _) => {
                    self.smt_build_parse(oracle_ctx, result, idents, expr)
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
                    ..
                } => self.smt_build_invoke(oracle_ctx, result, id, opt_idx, name, args, target),
                Statement::Assign(ident, opt_idx, expr, _) => {
                    self.smt_build_assign(oracle_ctx, result, ident, opt_idx, expr)
                }
            };
        }
        result
    }

    fn smt_codeblock_split(&self, oracle_ctx: &SplitOracleContext, block: CodeBlock) -> SmtExpr {
        let game_inst_ctx = self.context();
        let game_inst = game_inst_ctx.game_inst();
        let game = game_inst_ctx.game();

        let mut stmts = block.0.iter().rev();

        let innermost = stmts.next();
        let mut result = self.smt_build_innermost_split(oracle_ctx, innermost.unwrap());

        for stmt in stmts {
            result = match stmt {
                Statement::IfThenElse(cond, ifcode, elsecode, _) => SmtIte {
                    cond: cond.clone(),
                    then: self.smt_codeblock_split(oracle_ctx, ifcode.clone()),
                    els: self.smt_codeblock_split(oracle_ctx, elsecode.clone()),
                }
                .into(),
                Statement::Return(None, _) => {
                    // (mk-return-{name} statevarname expr)
                    todo!();
                    //self.smt_build_return_none_nonsplit(oracle_ctx)
                }
                Statement::Return(Some(_expr), _) => {
                    // (mk-return-{name} statevarname expr)
                    todo!();
                    //self.smt_build_return_some_nonsplit(oracle_ctx, expr)
                }
                Statement::Abort(_) => {
                    // mk-abort-{name}
                    self.smt_build_abort(oracle_ctx)
                }
                Statement::For(_, _, _, _, _) => {
                    let game_inst_name = game_inst.name();
                    let _game_name = game.name();
                    let pkg_inst_name = &oracle_ctx.pkg_inst_ctx().pkg_inst().name;
                    let pkg_name = &oracle_ctx.pkg_inst_ctx().pkg_inst().pkg.name;
                    let oracle_name = oracle_ctx.oracle_name();
                    unreachable!("found a for statement in the smt writer stage. \
                                 this should have been eliminated by now and can't be handled here. \
                                 game:{game_inst_name}(game_name) pkg:{pkg_inst_name}({pkg_name}) oracle:{oracle_name}", game_inst_name = game_inst_name, pkg_inst_name = pkg_inst_name, pkg_name =pkg_name, oracle_name = oracle_name)
                }
                // TODO actually use the type that we sample to know how far to advance the randomness tap
                Statement::Sample(ident, opt_idx, sample_id, tipe, _) => {
                    self.smt_build_sample(oracle_ctx, result, ident, opt_idx, sample_id, tipe)
                }
                Statement::Parse(idents, expr, _) => {
                    self.smt_build_parse(oracle_ctx, result, idents, expr)
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
                    ..
                } => self.smt_build_invoke(oracle_ctx, result, id, opt_idx, name, args, target),
                Statement::Assign(ident, opt_idx, expr, _) => {
                    self.smt_build_assign(oracle_ctx, result, ident, opt_idx, expr)
                }
            };
        }
        result
    }

    fn smt_build_innermost_nonsplit(
        &self,
        oracle_ctx: &OracleContext,
        stmt: &Statement,
    ) -> SmtExpr {
        match stmt {
            Statement::IfThenElse(cond, ifcode, elsecode, _) => SmtIte {
                cond: cond.clone(),
                then: self.smt_codeblock_nonsplit(oracle_ctx, ifcode.clone()),
                els: self.smt_codeblock_nonsplit(oracle_ctx, elsecode.clone()),
            }
            .into(),
            Statement::Return(None, _) => {
                // (mk-return-{name} statevarname expr)
                self.smt_build_return_none_nonsplit(oracle_ctx)
            }
            Statement::Return(Some(expr), _) => {
                // (mk-return-{name} statevarname expr)
                self.smt_build_return_some_nonsplit(oracle_ctx, expr)
            }
            Statement::Abort(_) => {
                // mk-abort-{name}
                self.smt_build_abort(oracle_ctx)
            }
            _ => unreachable!("found invalid statement at end of oracle: {:#?}", stmt),
        }
    }

    /*
     *
     * what is it that we have to do here?
     * - build either an abort or a return
     * - a return needs to contain the correct partial state for the next partial oracle invocation
     *
     * q: when we have progressed to this place, the code has already been rewritten
     *    by the partial split transform and if-then-elses that need to have
     *    been split have been removed. what is it then that is at the end of the oracles?
     * a: code now panics because the oracle codeblock ends in an assign, so maybe it can be
     *    anything? that's weird though, because the split partial transform is before the
     *    returnify transform. whatever code was returned by splitpartial should have been
     *    returnified...
     *    oh wait! the returnify transform does not operate on split oracles, only regulare ones!
     *    this seems to be a problem with separating the two that we hadn't factored in yet :(
     *    that could be fixed by just also looping over the code of the split_oracles. glad it was
     *    that easy!
     *
     * log: we still have the problem that, when sampling, we try to access the gamestate by
    // *      doing (select __global_state __state_length), which won't work here: we only have
     *      one game state here.
     *      maybe we need to add a method to the GenericOracleContext trait that returns the
     *      SmtExpr that evaluates to the gamestate in the context of the oracle.
     * log: okay, looks like i have removed a lot of instances where this is a problem. now
     *      this happens when _storing_ something in the gamestate variable. but i'll call it a
     *      night for now :)
     * log: new day, now together with christoph!
     *      we have started migrating the nonsplit return to also just have a single game state.
     *      that should make things simpler, hopefully!
     *      now it fails at the return statement below again
     *
     * what to do on return?
     * - figure out next state. how?
     *   - current path type
     *     - ifcond => if cond, then next, else elsenext
     *     - forstep => if still in range next, else elsenext
     *     - _ => next
     *
     * - how do we get the path type?
     *    - ofthen there is a trailing /plain
     *    - just start with last and go back until we find the right one
     *
     *
     *
     * */

    fn smt_build_next_intermediate_state(
        &self,
        oracle_ctx: &SplitOracleContext,
    ) -> Option<SmtExpr> {
        // println!("building next state");
        // println!("split oracle name: {}", oracle_ctx.split_path().smt_name());

        let path = oracle_ctx.split_path();
        let entry = self
            .split_info
            .iter()
            .find(|entry| entry.path() == path)
            .unwrap();
        let branches = entry.branches();
        // println!("branches: {branches:?}");

        let first_true = branches
            .iter()
            .position(|(cond, _)| cond == &Expression::BooleanLiteral("true".to_string()))
            .unwrap_or_else(|| panic!("branches: {:?}", branches));

        let (_, elsepath) = &branches[first_true];
        let mut block: SmtExpr = self.smt_build_intermediate_state_from_path(oracle_ctx, elsepath);

        for i in (0..=first_true).rev().skip(1) {
            let (cond, ifpath) = &branches[i];
            let ifblock = self.smt_build_intermediate_state_from_path(oracle_ctx, ifpath);

            block = SmtIte {
                cond: cond.clone(),
                then: ifblock,
                els: block,
            }
            .into();
        }

        // println!("block: {block}");

        Some(block)
    }

    fn smt_build_intermediate_state_from_path(
        &self,
        oracle_ctx: &SplitOracleContext,
        path: &SplitPath,
    ) -> SmtExpr {
        let game_inst_ctx = oracle_ctx.game_inst_ctx();
        let pkg_inst = oracle_ctx.pkg_inst_ctx().pkg_inst();

        let game_inst_name = &game_inst_ctx.game_inst().name();
        let pkg_inst_name = &pkg_inst.name;
        let oracle_name = oracle_ctx.oracle_name();

        let pattern = IntermediateStatePattern {
            game_inst_name,
            pkg_inst_name,
            oracle_name,
        };

        let spec = pattern.datastructure_spec(oracle_ctx.partials_dtype());

        let common_prefix = oracle_ctx.split_path().common_prefix(path);
        let common_loopvars =
            common_prefix
                .path()
                .iter()
                .filter_map(|elem| match elem.split_type() {
                    SplitType::ForStep(name, start, _) => Some((name, start)),
                    _ => None,
                });

        let their_loopvars = path
            .path()
            .iter()
            .filter_map(|elem| match elem.split_type() {
                SplitType::ForStep(name, start, _) => Some((name, start)),
                _ => None,
            });

        let is_next_iteration = path < oracle_ctx.split_path();

        let mut next_loopvar_values = vec![];

        for (name, start) in their_loopvars {
            let is_common = common_loopvars
                .clone()
                .any(|(their_name, _)| name == their_name);

            next_loopvar_values.push(match (is_common, is_next_iteration) {
                (true, true) => (
                    name.ident(),
                    (
                        "+",
                        1,
                        pattern
                            .access(
                                &spec,
                                &IntermediateStateSelector::LoopVar(path, name.ident_ref()),
                                "__intermediate_state",
                            )
                            .unwrap(),
                    )
                        .into(),
                ),
                (true, false) => (
                    name.ident(),
                    pattern
                        .access(
                            &spec,
                            &IntermediateStateSelector::LoopVar(path, name.ident_ref()),
                            "__intermediate_state",
                        )
                        .unwrap(),
                ),
                (false, _) => (name.ident(), start.clone().into()),
            })
        }

        pattern
            .call_constructor(
                &spec,
                &IntermediateStateConstructor::OracleState(path),
                |sel| {
                    Some(match sel {
                        IntermediateStateSelector::Arg(_, name, _)
                        | IntermediateStateSelector::Local(_, name, _) => (*name).into(),
                        IntermediateStateSelector::LoopVar(_, name) => {
                            let (_, next_value) = next_loopvar_values
                                .iter()
                                .find(|(lv_name, _next_value)| lv_name == name)
                                .unwrap();
                            next_value.clone()
                        }
                        IntermediateStateSelector::Child(_) => "mk-child-this-is-wrong".into(),
                        IntermediateStateSelector::Return(_) => unreachable!(),
                    })
                },
            )
            .unwrap()

        // let constructor = IntermediateStateConstructor::OracleState(path);
        // let constructor_name = pattern.constructor_name(&constructor);
        //
        // let entry = self
        //     .split_info
        //     .iter()
        //     .find(|entry| entry.path() == path)
        //     .unwrap();
        //
        // let mut locals: Vec<SmtExpr> = entry
        //     .locals()
        //     .iter()
        //     .map(|(name, _)| name.clone().into())
        //     .collect();
        //
        // let mut fn_call = vec![constructor_name.into()];
        // fn_call.append(&mut locals);
        //
        // SmtExpr::List(fn_call)
    }

    fn smt_build_innermost_split(
        &self,
        oracle_ctx: &SplitOracleContext,
        stmt: &Statement,
    ) -> SmtExpr {
        //let odef = oracle_ctx.oracle_def();
        let game_inst_ctx = oracle_ctx.game_inst_ctx();
        let pkg_inst = oracle_ctx.pkg_inst_ctx().pkg_inst();

        let game_inst_name = game_inst_ctx.game_inst().name();
        let pkg_inst_name = &pkg_inst.name;
        let oracle_name = oracle_ctx.oracle_name();

        //("todo_build_innermost_split",).into();
        match stmt {
            Statement::IfThenElse(cond, ifcode, elsecode, _) => SmtIte {
                cond: cond.clone(),
                then: self.smt_codeblock_split(oracle_ctx, ifcode.clone()),
                els: self.smt_codeblock_split(oracle_ctx, elsecode.clone()),
            }
            .into(),
            // this is probably wrong because we don't have any selectors, but maybe it's ok idk:
            Statement::Abort(_) => {
                let partial_return_pattern = PartialReturnPattern {
                    game_inst_name,
                    pkg_inst_name,
                    oracle_name,
                };
                let spec = partial_return_pattern.datastructure_spec(&());
                partial_return_pattern
                    .call_constructor(&spec, &PartialReturnConstructor::Abort, |_| unreachable!())
                    .unwrap()
            }
            Statement::Return(None, _) => {
                let partial_return_pattern = PartialReturnPattern {
                    game_inst_name,
                    pkg_inst_name,
                    oracle_name,
                };

                let spec = partial_return_pattern.datastructure_spec(&());
                partial_return_pattern
                    .call_constructor(&spec, &PartialReturnConstructor::Return, |sel| {
                        Some(match sel {
                            patterns::PartialReturnSelector::GameState => "__global_state".into(),
                            patterns::PartialReturnSelector::IntermediateState => {
                                let ipattern = oracle_ctx.intermediate_state_pattern();
                                let ispec =
                                    ipattern.datastructure_spec(oracle_ctx.partials_dtype());
                                ipattern
                                    .call_constructor(
                                        &ispec,
                                        &IntermediateStateConstructor::End,
                                        |sel| {
                                            Some(match sel {
                                                IntermediateStateSelector::Arg(_, _, _)
                                                | IntermediateStateSelector::LoopVar(_, _)
                                                | IntermediateStateSelector::Local(_, _, _)
                                                | IntermediateStateSelector::Child(_) => {
                                                    unreachable!()
                                                }
                                                IntermediateStateSelector::Return(_) => {
                                                    "mk-empty".into()
                                                }
                                            })
                                        },
                                    )
                                    .unwrap()
                            }
                        })
                    })
                    .unwrap()
            }
            Statement::Return(Some(_expr), _) => {
                // (mk-return-{name} statevarname expr)
                // self.smt_build_return_some(oracle_ctx, expr)

                // this is also a real return
                ("todo_build_innermost_split_some_return",).into()
            }
            _ => {
                // build the generic partialreturn and let the caller know that this statment still
                // needs to be processed

                let next_state =
                    if let Some(next_state) = self.smt_build_next_intermediate_state(oracle_ctx) {
                        next_state
                    } else {
                        let constructor_name = IntermediateStatePattern {
                            game_inst_name,
                            pkg_inst_name,
                            oracle_name,
                        }
                        .constructor_name(&IntermediateStateConstructor::End);

                        (constructor_name, "mk-empty").into()
                    };
                // let split_path = oracle_ctx.split_path();
                // let this_entry = self
                //     .split_info
                //     .iter()
                //     .find(|entry| entry.path() == split_path)
                //     .unwrap();
                // println!("{} -- {:#?}", split_path.smt_name(), this_entry);

                let partial_return_pattern = PartialReturnPattern {
                    game_inst_name,
                    pkg_inst_name,
                    oracle_name,
                };

                let constructor_name =
                    partial_return_pattern.constructor_name(&PartialReturnConstructor::Return);

                (constructor_name, "__global_state", next_state).into()
            }
        }
    }

    fn smt_build_return_none_nonsplit(&self, oracle_ctx: &OracleContext) -> SmtExpr {
        let game_inst_ctx = oracle_ctx.game_inst_ctx();
        let pkg_inst = oracle_ctx.pkg_inst_ctx().pkg_inst();

        let var_gamestate = &GlobalStatePattern;
        oracle_ctx.smt_game_state();
        let var_selfstate = names::var_selfstate_name();

        let new_gamestate = game_inst_ctx
            .smt_update_gamestate_pkgstate(
                var_gamestate.name(),
                self.sample_info,
                &pkg_inst.name,
                var_selfstate,
            )
            .unwrap();

        SmtLet {
            bindings: vec![(var_gamestate.name(), new_gamestate)],
            body: oracle_ctx.smt_construct_return(var_gamestate, "mk-empty"),
        }
        .into()
    }

    fn smt_build_return_some_nonsplit(
        &self,
        oracle_ctx: &OracleContext,
        expr: &Expression,
    ) -> SmtExpr {
        let game_inst_ctx = oracle_ctx.game_inst_ctx();
        let pkg_inst = oracle_ctx.pkg_inst_ctx().pkg_inst();

        let var_gamestates = names::var_globalstate_name();
        let old_gamestate = oracle_ctx.smt_game_state();
        let var_selfstate = names::var_selfstate_name();
        let new_gamestate = game_inst_ctx
            .smt_update_gamestate_pkgstate(
                old_gamestate,
                self.sample_info,
                &pkg_inst.name,
                var_selfstate,
            )
            .unwrap();

        SmtLet {
            bindings: vec![(names::var_globalstate_name(), new_gamestate)],
            body: oracle_ctx.smt_construct_return(var_gamestates, expr.clone()),
        }
        .into()
    }

    fn smt_build_abort<OCTX: GenericOracleContext>(&self, oracle_ctx: &OCTX) -> SmtExpr {
        let game_inst_ctx = oracle_ctx.game_inst_ctx();
        let inst = oracle_ctx.pkg_inst_ctx().pkg_inst();

        let var_gamestate = &GlobalStatePattern;
        let var_selfstate = names::var_selfstate_name();

        let new_gamestate = game_inst_ctx
            .smt_update_gamestate_pkgstate(
                var_gamestate,
                self.sample_info,
                &inst.name,
                var_selfstate,
            )
            .unwrap();

        let body = oracle_ctx.smt_construct_abort();

        SmtLet {
            bindings: vec![(var_gamestate.name(), new_gamestate)],
            body,
        }
        .into()
    }

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

    fn smt_build_sample<OCTX: GenericOracleContext>(
        &self,
        oracle_ctx: &OCTX,
        result: SmtExpr,
        ident: &Identifier,
        opt_idx: &Option<Expression>,
        sample_id: &Option<usize>,
        tipe: &Type,
    ) -> SmtExpr {
        let sample_id = sample_id.expect("found a None sample_id");
        let game_inst_ctx = self.context();

        let game_name = game_inst_ctx.game().name();

        let gamestate = oracle_ctx.smt_game_state();
        // ctr is the current "i-th sampling for sample id sample_id"
        let ctr = self
            .get_randomness(gamestate, sample_id)
            .unwrap_or_else(|| {
                let max_known_sample_id = self.sample_info.count;
                panic!(
                    "found sample id {} that exceeds highest expected {}",
                    sample_id, max_known_sample_id
                )
            });

        let rand_fn_name = names::fn_sample_rand_name(game_name, tipe);

        let rand_val: SmtExpr = (rand_fn_name, format!("{sample_id}"), ctr.clone()).into();

        let new_val = if let Some(idx) = opt_idx {
            ("store", ident.clone(), idx.clone(), rand_val.clone()).into()
        } else {
            rand_val
        };

        let bindings = vec![(ident.ident(), new_val)]
            .into_iter()
            .filter(|(x, _)| x != "_")
            .collect();

        let var_gamestate = &GlobalStatePattern;
        let new_gamestate = game_inst_ctx
            .smt_increment_gamestate_rand(var_gamestate, self.sample_info, sample_id)
            .unwrap();

        SmtLet {
            bindings,
            body: SmtLet {
                bindings: vec![(var_gamestate.name(), new_gamestate)],
                body: result,
            },
        }
        .into()
    }

    fn smt_build_parse<OCTX: GenericOracleContext>(
        &self,
        _oracle_ctx: &OCTX,
        result: SmtExpr,
        idents: &[Identifier],
        expr: &Expression,
    ) -> SmtExpr {
        let bindings = idents
            .iter()
            .filter(|ident| ident.ident() != "_")
            .enumerate()
            .map(|(i, ident)| {
                let ident = if let Identifier::Generated(ident, _) = ident {
                    ident
                } else {
                    unreachable!()
                };

                (ident.clone(), (format!("el{}", i + 1), expr.clone()).into())
            })
            .collect();

        SmtLet {
            bindings,
            body: result,
        }
        .into()
    }

    fn smt_build_invoke<OCTX: GenericOracleContext>(
        &self,
        oracle_ctx: &OCTX,
        result: SmtExpr,
        id: &Identifier,
        opt_idx: &Option<Expression>,
        name: &str,
        args: &[Expression],
        target: &str,
    ) -> SmtExpr {
        let game_inst_ctx = self.context();
        let pkg_inst = oracle_ctx.pkg_inst_ctx().pkg_inst();
        let oracle_name = oracle_ctx.oracle_name();

        let _game_inst_name = game_inst_ctx.game_inst().name();
        let _pkg_inst_name = &pkg_inst.name;

        let called_oracle_context = self.get_oracle_context(target, name).unwrap();
        let this_oracle_context = self
            .get_oracle_context(&pkg_inst.name, oracle_name)
            .unwrap();

        let var_gamestate = &GlobalStatePattern;
        let var_selfstate = &SelfStatePattern;

        let then_new_gamestate = game_inst_ctx
            .smt_update_gamestate_pkgstate(
                var_gamestate,
                self.sample_info,
                &pkg_inst.name,
                var_selfstate,
            )
            .unwrap();

        let is_abort_body = SmtLet {
            bindings: vec![(var_gamestate.name(), then_new_gamestate)],
            body: this_oracle_context.smt_construct_abort(),
        };

        let let_bindings = vec![(
            var_gamestate.name(),
            called_oracle_context.smt_access_return_state(names::var_ret_name()),
        )];

        let return_value_pattern = ReturnValue::new(oracle_ctx.oracle_return_type());
        let return_value_spec = return_value_pattern.datastructure_spec(&());

        let call_args: Vec<SmtExpr> = [SmtExpr::from(var_gamestate)]
            .iter()
            .cloned()
            .chain(args.iter().map(|arg| arg.clone().into()))
            .collect();
        let call_result = called_oracle_context.oracle_pattern().call(&call_args);

        let smt_expr = SmtLet {
            bindings: vec![(names::var_ret_name(), call_result)],
            body: SmtIte {
                cond: called_oracle_context.smt_access_return_is_abort(names::var_ret_name()),
                then: SmtLet {
                    bindings: let_bindings.clone(),
                    body: is_abort_body,
                },
                els: SmtLet {
                    bindings: {
                        let mut bindings = let_bindings.clone();

                        if id.ident() != "_" {
                            bindings.push((
                                id.ident(),
                                return_value_pattern
                                    .access(
                                        &return_value_spec,
                                        &ReturnValueSelector,
                                        called_oracle_context
                                            .smt_access_return_value(names::var_ret_name()),
                                    )
                                    .unwrap(),
                            ));
                        }

                        bindings
                    },
                    body: result,
                },
            },
        };

        if opt_idx.is_some() {
            ("store", id.clone(), opt_idx.clone().unwrap(), smt_expr).into()
        } else {
            smt_expr.into()
        }
    }

    fn smt_build_assign<OCTX: GenericOracleContext>(
        &self,
        oracle_ctx: &OCTX,
        result: SmtExpr,
        ident: &Identifier,
        opt_idx: &Option<Expression>,
        expr: &Expression,
    ) -> SmtExpr {
        let t = expr.get_type();

        //eprintln!(r#"DEBUG code_smt_helper Assign {expr:?} to identifier {ident:?}")"#);

        // first build the unwrap expression, if we have to
        let outexpr = if let Expression::Unwrap(inner) = &expr {
            ("maybe-get", *inner.clone()).into()
        } else {
            expr.clone().into() // TODO maybe this should be inner??
        };

        // then build the table store smt expression, in case we have to
        let outexpr = if let Some(idx) = opt_idx {
            let oldvalue: SmtExpr = match &ident {
                // TODO: we got rid of this Identifier variant. Need to update to the new ones
                //
                // &Identifier::State(PackageState { name, .. }) => {
                //     //assert_eq!(pkgname, inst_name, "failed assertion: in an oracle in instance {inst_name} I found a state identifier with {pkgname}. I assumed these would always be equal.");
                //     oracle_ctx
                //         .pkg_inst_ctx()
                //         .smt_access_pkgstate(names::var_selfstate_name(), name)
                //         .unwrap()
                // }
                //
                Identifier::Generated(_, _) => ident.clone().into(),
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
            // TODO: we got rid of this Identifier variant. Need to update to the new ones
            //
            // Identifier::State(PackageState { name, .. }) => {
            //     //assert_eq!(pkgname, inst_name, "failed assertion: in an oracle in instance {inst_name} I found a state identifier with {pkgname}. I assumed these would always be equal.");
            //     SmtLet {
            //         bindings: vec![(
            //             names::var_selfstate_name(),
            //             oracle_ctx
            //                 .pkg_inst_ctx()
            //                 .smt_update_pkgstate(names::var_selfstate_name(), name, outexpr)
            //                 .unwrap()
            //                 .into(),
            //         )],
            //         body: result,
            //     }
            // }
            Identifier::Generated(name, _) => SmtLet {
                bindings: vec![(name.clone(), outexpr)]
                    .into_iter()
                    .filter(|(name, _)| name != "_:")
                    .collect(),
                body: result,
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
                        term: "mk-none",
                        sort: Type::Maybe(Box::new(t)),
                    },
                },
                then: oracle_ctx.smt_construct_abort(),
                els: smtout,
            }
            .into()
        } else {
            smtout.into()
        }
    }

    // fn smt_next_gamestate<S: Into<SmtExpr>>(
    //     oracle_ctx: &OracleContext,
    //     entry: &SplitInfoEntry,
    //     ret_stmt: &Statement,
    //     old_gamestate: S,
    // ) -> SmtExpr {
    //     let oracle_name = &oracle_ctx.oracle_def().sig.name;
    //     let pkg_inst_name = oracle_ctx.pkg_inst_ctx().pkg_inst_name();
    //
    //     let mut cur_path = entry.path();
    //
    //     // we need to figure out the gamestates for both next and elsenext
    //
    //     let shared_prefix_next = entry
    //         .next()
    //         .map(|next| cur_path.longest_shared_prefix(next));
    //
    //     let shared_prefix_elsenext = entry
    //         .elsenext()
    //         .map(|next| cur_path.longest_shared_prefix(next));
    //
    //     let shared_prefix_next = match shared_prefix_next {
    //         None => return oracle_ctx.smt_access_intermediate_parent(old_gamestate),
    //         Some(x) => x,
    //     };
    //
    //     todo!()
    //
    //     // x <- Foo() ---
    //     // ----------   | next
    //     // return x   <--
    //
    //     // .../Invoc():
    //     //     invoke Oracle
    //     //
    //     // .../Plain
    //     // .../Invoc/...
    //     // .../Invoc/...
    //     // .../Plain
    //
    //     // We are in a partial function!
    //     //
    //     // what now?
    //     // -> construct next partial state!
    //     //    what is the next parent?
    //     //    enter function: push!
    //     //    leave function: pop!
    //     //    else:           copy!
    //     //
    //     // complications:
    //     // - next or elsenext?
    //     // - what if we leave multiple functions at once?
    //     //   - this can't happen: after an invoc there definitely is a plain block
    //     //     with a return statement
    //     //     - TODO: partial_split transform: make sure there the is a return if
    //     //             invoc is last statement
    //     // - we also can't leave and enter at the same time
    //     //
    //     // so: same as determine_next, but return actual smt expression for the
    //     // next partial state?
    //     // - find common prefix path
    //     // - pop path until common prefix
    //     //   - match popped path element
    //     //     - plain: skip
    //     //     - ifcond:
    //     //     - ifbranch
    //     //     - elsebranch
    //     //     - invoc
    //     //     - forstep
    //     // - go down to next
    //     //
    // }

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

    fn smt_define_nonsplit_oracle_fn(&self, inst: &PackageInstance, def: &OracleDef) -> SmtExpr {
        let game_inst_ctx = self.context();
        let _game_state_pattern = game_inst_ctx.game_state_pattern();
        let var_globalstate = &GlobalStatePattern;
        let var_selfstate = &SelfStatePattern;

        let game_inst_name = game_inst_ctx.game_inst().name();
        let pkg_inst_name = &inst.name;
        let oracle_sig = &def.sig;
        let oracle_name = &oracle_sig.name;

        let oracle_ctx = game_inst_ctx
            .pkg_inst_ctx_by_name(pkg_inst_name)
            .expect("couldn't find package instance with name {inst_name}")
            .oracle_ctx_by_name(oracle_name)
            .expect("couldn't find oracle with name {oracle_name}");

        let oracle_pattern = OraclePattern {
            game_inst_name,
            pkg_inst_name,
            oracle_name,
            oracle_args: &oracle_sig.args,
        };

        oracle_pattern.define_fun(SmtLet {
            bindings: vec![(
                var_selfstate.name(),
                game_inst_ctx
                    .smt_access_gamestate_pkgstate(var_globalstate.name(), pkg_inst_name)
                    .unwrap(),
            )],
            body: self.smt_codeblock_nonsplit(&oracle_ctx, def.code.clone()),
        })
    }

    fn smt_define_split_oracle_fn(&self, split_oracle_ctx: &SplitOracleContext) -> SmtExpr {
        let game_inst_ctx = self.context();
        let game_inst_name = game_inst_ctx.game_inst().name();
        let game_name = game_inst_ctx.game().name();
        let def = split_oracle_ctx.oracle_def();
        let pkg_inst = split_oracle_ctx.pkg_inst_ctx().pkg_inst();

        let code = &def.code;
        let mut args = vec![
            (
                names::var_globalstate_name(),
                names::gamestate_sort_name(game_name),
            )
                .into(),
            (
                "__intermediate_state",
                split_oracle_ctx.intermediate_state_pattern().sort(),
            )
                .into(),
        ];

        //args.extend(split_oracle_ctx.oracle_def().sig.path.additional_args());

        let rest_args = def.sig.args.iter().cloned().map(|arg| arg.into());
        args.extend(rest_args);

        let pkg_inst_name = &pkg_inst.name;
        let oracle_name = &split_oracle_ctx.oracle_def().sig.name;
        //let oracle_name = split_oracle_ctx.oracle_def().sig.name_with_path();

        let split_path = &split_oracle_ctx.oracle_def().sig.path;

        let partial_oracle_function_pattern = PartialOraclePattern {
            game_inst_name,
            pkg_inst_name,
            oracle_name,
            split_path,
        };

        let pattern = split_oracle_ctx.intermediate_state_pattern();
        let dtype = split_oracle_ctx.partials_dtype();
        let spec = pattern.datastructure_spec(dtype);
        let intermediate_state_constructor = IntermediateStateConstructor::OracleState(split_path);
        let bindings = vec![(
            names::var_selfstate_name(),
            game_inst_ctx
                .smt_access_gamestate_pkgstate(names::var_globalstate_name(), pkg_inst_name)
                .unwrap(),
        )];

        (
            "define-fun",
            partial_oracle_function_pattern.function_name(),
            SmtExpr::List(args.clone()),
            partial_oracle_function_pattern.function_return_sort(),
            SmtLet {
                bindings,
                body: pattern
                    .recover_variables(
                        &spec,
                        &intermediate_state_constructor,
                        self.smt_codeblock_split(split_oracle_ctx, code.clone()),
                    )
                    .unwrap(),
            },
        )
            .into()
    }

    fn smt_pkg_code(&self, inst: &PackageInstance) -> Vec<SmtExpr> {
        inst.pkg
            .oracles
            .iter()
            .map(|def| self.smt_define_nonsplit_oracle_fn(inst, def))
            .collect()
    }

    fn smt_composition_code(&self) -> Vec<SmtExpr> {
        let game_inst_ctx = self.context();
        let game_inst_name = game_inst_ctx.game_inst().name();
        let game_name = game_inst_ctx.game().name();
        let comment = vec![SmtExpr::Comment(format!(
            "Game instance {game_inst_name} of game {game_name}\n",
        ))];
        let ordered_pkgs = game_inst_ctx.game().ordered_pkgs();
        let code = ordered_pkgs.iter().flat_map(|inst| self.smt_pkg_code(inst));

        comment.into_iter().chain(code).collect()
    }

    fn smt_composition_paramfuncs(&self) -> Vec<SmtExpr> {
        let game_inst_ctx = self.context();
        let game_inst = game_inst_ctx.game_inst();
        let game_inst_name = game_inst.name();
        let game = game_inst_ctx.game();
        let fns = game
            .consts
            .iter()
            .filter(|(_, tipe)| matches!(tipe, Type::Fn(_, _)));
        let _comp_name = game.name();

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
                    format!("__func-{game_inst_name}-{name}"),
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
        let game_inst_ctx = self.context();
        let _game_inst = game_inst_ctx.game_inst();
        let game = game_inst_ctx.game();
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
                        game.name,
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

    fn smt_composition_partial_stuff(&self) -> Vec<SmtExpr> {
        let game_inst_ctx = self.context();
        let game_inst = game_inst_ctx.game_inst();
        let game = game_inst_ctx.game();

        let partial_dtpes = into_partial_dtypes(self.split_info);
        let game_inst_name = game_inst.name();

        let mut return_typedefs = vec![];
        let mut intermediate_state_typedefs = vec![];
        let mut dispatch_fndefs = vec![];
        let mut split_oracle_fndefs = vec![];

        for dtype in &partial_dtpes {
            let pkg_inst_name = &dtype.pkg_inst_name;
            let oracle_name = &dtype.real_oracle_sig.name;
            let _game_name = &game.name;
            let pkg_inst_ctx = game_inst_ctx.pkg_inst_ctx_by_name(pkg_inst_name).unwrap();

            let intermediate_state_pattern = IntermediateStatePattern {
                game_inst_name,
                pkg_inst_name,
                oracle_name,
            };
            let intermediate_state_spec = intermediate_state_pattern.datastructure_spec(dtype);

            // this can't work, because it looks for an existing oracle. but it doesn't exist
            // anymore
            //let oracle_ctx = pkg_inst_ctx.oracle_ctx_by_name(oracle_name).unwrap();

            for step in &dtype.partial_steps {
                let orig_oracle_sig = &dtype.real_oracle_sig;
                let oname = &orig_oracle_sig.name;
                let opath = step.path().smt_name();
                let split_oracle_ctx = pkg_inst_ctx
                    .split_oracle_ctx_by_name_and_path(&orig_oracle_sig.name, step.path(), dtype)
                    .unwrap_or_else(|| {
                        panic!(
                            "couldn't find split oracle context for ({oname}, {opath}) in {:?}",
                            pkg_inst_ctx.pkg_inst().pkg.split_oracles
                        )
                    });
                split_oracle_fndefs.push(self.smt_define_split_oracle_fn(&split_oracle_ctx));
            }

            return_typedefs.push(self.smt_declare_partial_return_datatype(dtype));
            intermediate_state_typedefs.push(declare_datatype(
                &intermediate_state_pattern,
                &intermediate_state_spec,
            ));
            dispatch_fndefs.push(pkg_inst_ctx.smt_declare_oracle_dispatch_function(dtype));
        }

        let mut out = intermediate_state_typedefs;

        out.append(&mut return_typedefs);
        out.append(&mut split_oracle_fndefs);
        out.append(&mut dispatch_fndefs);
        out
    }

    pub(crate) fn smt_declare_partial_return_datatype(&self, dtype: &PartialsDatatype) -> SmtExpr {
        let game_inst_ctx = self.context();
        let game_inst = game_inst_ctx.game_inst();
        let _game = game_inst_ctx.game();

        let game_inst_name = game_inst.name();
        let pkg_inst_name = &dtype.pkg_inst_name;
        let oracle_name = &dtype.real_oracle_sig.name;

        let prp = patterns::PartialReturnPattern {
            game_inst_name,
            pkg_inst_name,
            oracle_name,
        };

        let spec = prp.datastructure_spec(&());
        declare_datatype(&prp, &spec)
    }

    pub fn smt_composition_all(&mut self) -> Vec<SmtExpr> {
        let rand = self.smt_composition_randomness();
        let paramfuncs = self.smt_composition_paramfuncs();
        let state = self.smt_composition_state();
        let ret = self.smt_composition_return();
        let code = self.smt_composition_code();
        let partial_stuff = self.smt_composition_partial_stuff();

        rand.into_iter()
            .chain(paramfuncs)
            .chain(state)
            .chain(ret)
            .chain(code)
            .chain(partial_stuff)
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
            bindings: vec![("x".into(), Expression::IntegerLiteral(42).into())],
            body: SmtExpr::Atom(String::from("x")),
        };

        let out: SmtExpr = l.into();
        let mut str = String::new();

        write!(&mut str, "{out}").unwrap();

        assert_eq!(str, "(let ((x 42)\n)\n x)\n");

        Ok(())
    }
}
