use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::iter::FromIterator;

use crate::util::resolver::Named;
use crate::writers::smt::patterns::FunctionPattern;
use crate::{
    hacks,
    package::{Composition, Export, OracleSig, SplitExport},
    proof::{Claim, ClaimType, Equivalence, GameInstance, Proof},
    split::{SplitOracleSig, SplitPath},
    transforms::{
        proof_transforms::EquivalenceTransform,
        samplify::{Position, SampleInfo},
        split_partial::SplitInfoEntry,
        ProofTransform,
    },
    types::Type,
    util::{
        prover_process::{Communicator, ProverResponse},
        resolver::{Resolver, SliceResolver},
    },
    writers::smt::{
        contexts::{self, GenericOracleContext},
        declare,
        exprs::{SmtAnd, SmtAssert, SmtEq2, SmtExpr, SmtForall, SmtImplies, SmtIte, SmtNot},
        partials::{into_partial_dtypes, PartialsDatatype},
        patterns::{self, ConstantPattern, DatastructurePattern},
        writer::CompositionSmtWriter,
    },
};

pub mod error;
mod verify_fn;

use error::{Error, Result};
pub use verify_fn::verify;

struct EquivalenceContext<'a> {
    eq: &'a Equivalence,
    proof: &'a Proof,
    auxs: &'a <EquivalenceTransform as ProofTransform>::Aux,
}

impl<'a> EquivalenceContext<'a> {
    fn emit_base_declarations(&self, comm: &mut Communicator) -> Result<()> {
        let mut base_declarations: Vec<SmtExpr> = vec![("set-logic", "ALL").into()];

        println!("types {:?}", self.types());

        for tipe in self.types() {
            if let Type::Bits(id) = &tipe {
                base_declarations.extend(hacks::BitsDeclaration(id.to_string()).into_iter());
            }
        }

        base_declarations.extend(hacks::MaybeDeclaration.into_iter());
        base_declarations.push(hacks::ReturnValueDeclaration.into());
        base_declarations.extend(hacks::TuplesDeclaration(1..32).into_iter());
        base_declarations.extend(hacks::EmptyDeclaration.into_iter());

        for decl in base_declarations {
            comm.write_smt(decl)?
        }

        Ok(())
    }

    fn emit_game_definitions(&self, comm: &mut Communicator) -> Result<()> {
        let instance_resolver = SliceResolver(self.proof.instances());
        let left = instance_resolver.resolve(&self.eq.left_name()).unwrap();
        let right = instance_resolver.resolve(&self.eq.right_name()).unwrap();

        let mut left_writer =
            CompositionSmtWriter::new(left, self.sample_info_left(), self.split_info_left());
        let mut right_writer =
            CompositionSmtWriter::new(right, self.sample_info_right(), self.split_info_right());

        let mut out = vec![];

        out.append(&mut left_writer.smt_composition_all());
        out.append(&mut right_writer.smt_composition_all());

        for decl in out {
            comm.write_smt(decl)?
        }

        Ok(())
    }

    fn emit_constant_declarations(&self, comm: &mut Communicator) -> Result<()> {
        /*
         *
         * things being declared here:
         * - nonsplit oracle args
         * - for $game_inst in left, right
         *   - old game state $game_inst
         *   - new game state $game_inst
         *   - randomness counters $game_inst
         *   - randomness values $game_inst
         *   - for oracle in game.non-split-exports
         *     - return $game_inst $oracle
         *   - for oracle in game.split-exports
         *     - partial return $game_inst $oracle
         *     - split oracle args
         *
         * things being constrained here:
         * - for $game_inst in left, right
         *   - rand_ctr_$i = get_rand(game_state, $i)
         *   - rand_val_$i = rand_$game_inst($i, rand_ctr_$i)
         *   - for $oracle in $game_inst.non-split-exports
         *     - return = $oracle(state, args...)
         *     - new_game_state_$game_inst = get-state(return)
         *       - wait, maybe this should only be in the procondition of the claim statements
         *   - for $oracle in $game_inst.non-split-exports
         *     - partial return = $oracle(state, args...)
         *
         * Jan's thoughts on the design of the next iteration of this:
         *
         * What can go wrong here?
         *
         *   Underconstraining
         *
         *     The solver would give us a sat where we expect an unsat and we can
         *     use the model to see which constraint is missing. Until that is done, we can't prove
         *     anything but that is not that big of a deal. So I guess this is an easily debuggable
         *     completeness problem.
         *
         *   Overconstraining
         *
         *     We might add too many constraints, which would lead to the solver
         *     reporting unsat where it should return sat. This would break soundness, in ways that
         *     are not easily debuggable.
         *
         *   I feel like soundness is more important than completeness!
         *
         * What can we do to prevent that? (TODO)
         *
         *   Testing
         *
         *     I suppose the best way to guard against this is to have test cases with proofs
         *     that are expected to not go through and make sure that this is actually the case.
         *
         *   Clear Documentation/Spec
         *
         *     Making explicit the model we have of the system helps both
         *     with catching logic bugs (because in order to vet the logic you can read the docs)
         *     and implementation bugs (because you can compare the implementation against the spec).
         *
         * When do we apply the constraints?
         *
         *   Option A: Immediately after declaring
         *
         *     This doesn't work for e.g. the "new state", as that would be constrained in
         *     contradictory ways. My current heuristic is that if the value is the output of a
         *     function and there are several potential functions that it could be the output of,
         *     then it won't work.
         *
         *       Can we maybe avoid that issue by not "overloading" constants? Use constants as
         *       the output of one particular thing? What are other instances of constants that are
         *       constrained differently depending on the call?
         *
         *         Other instances: I was going to say PartialReturn, but not only by "real" oracle
         *         but also by split oracle, but I don' think that is true since because of the
         *         dispatch function. So maybe it's just Return and PartialReturn, by "real" oracle?
         *
         *         We could avoid that by not having a single "new state" constant, but one per
         *         oracle. That might be a tad inconvenient though? Or we just bind the convenient
         *         names using let, either in the lemma/relation/invariant or in the glue code
         *         calling it. This would mean we don't even need the constants and don't need to
         *         constrain them. Sounds like there is less chance of confusion, too!
         *
         *   Option B: First declare all constants, then constrain
         *
         *     Seems difficult to keep track of the constraints we still need to do.
         *
         * So to me it seems the best way is to
         *
         * 1.  declare foundational constants ("old state", "function arguments")
         * 2.  declare constants that conceptually are outputs of a known function taking
         *     foundational constants ("return per oracle") and immediately constrain them
         * 3.  only bind convenience values in (let ..) blocks close to the code using them.
         *     This can be done manually in the user code, or in the glue code calling the user
         *     code.
         *
         *       I think there is a discussion to be had here, though. If we go with the let-bind
         *       approache, we can't make the randomness mapping a bunch of asserts. It needs to be
         *       an expression that evaluates to a bool. Is the user fine with that?
         *
         *       I think this can affect model readability (for a human) in one of two ways:
         *
         *         Possible Impact A: There a fewer global constants, and all the values are in the
         *         specific part of the gamestate. It is more tidy and it is easy to find what you
         *         are looking for.
         *
         *         Possibe Impact B: Instead of having a global constant rand-Real-1-4 as a constant
         *         in the model, you have to sift through the game state structs to find the
         *         correct one to see the value, which makes it more difficult.
         *
         *         I wonder which of these would be stronger, and believe it depends on the habits and
         *         preferences of the user.
         *
         * Which leaves us to specify (and give reasons for) our list of constants and constraints.
         * Afterwards, we also make a list of constants constraints we chose not to include here.
         *
         *   Foundational Constants: Old Gamestate, Old Intermediate State and Arguments
         *
         *     These are only used as inputs to the oracle functions. There is nothing we can tie
         *     them to, we can only constrain them in lemmas, etc.
         *
         *   Function Outputs: Return, PartialReturn
         *
         *     These can be directly computed from the above. They should simply be constrained.
         *
         *   Convenience Values: New Gamestate, New Intermediate State, IsAbort, Return Value,
         *                       Randomness Counters, Random Values
         *
         *     These fall in two categories:
         *
         *     1.  Values where a convenient name would not be globally unique (e.g. new state, is abort)
         *
         *           Here I think using (let ..) bindings really is the best way to handle the
         *           ambiguity.
         *
         *     2.  Values that have unique names, but are rarely needed and are just copied from the
         *         gamestate (e.g. randomness)
         *
         *           Here I am not sure - From a "purity" standpoint it feels nice to me, but I see how
         *           that is not a very strong argument, so we may just declare and constrain them globally.
         *
         */

        let instance_resolver = SliceResolver(self.proof.instances());

        let left_game_inst_name = self.eq.left_name();
        let right_game_inst_name = self.eq.right_name();

        let left = instance_resolver.resolve(self.eq.left_name()).unwrap();
        let right = instance_resolver.resolve(self.eq.right_name()).unwrap();

        let gctx_left = contexts::GameInstanceContext::new(left);
        let gctx_right = contexts::GameInstanceContext::new(right);

        let left_game = gctx_left.game();
        let right_game = gctx_right.game();

        let left_game_name = &gctx_left.game().name;
        let right_game_name = &gctx_right.game().name;

        let mut out = Vec::new();

        /////// old state constants

        let decl_state_left = patterns::GameState {
            game_inst_name: left_game_inst_name,
            variant: "old",
        }
        .declare();

        let decl_state_right = patterns::GameState {
            game_inst_name: right_game_inst_name,
            variant: "old",
        }
        .declare();

        out.push(decl_state_left);
        out.push(decl_state_right);

        let mut processed = HashSet::new();
        for SplitExport(pkg_inst_offs, oracle_sig) in &left_game.split_exports {
            let pkg_inst_name = &left_game.pkgs[*pkg_inst_offs].name;
            let oracle_name = &oracle_sig.name;

            if processed.contains(oracle_name) {
                continue;
            }

            processed.insert(oracle_name);

            let decl_intermediate_state_left = patterns::IntermediateStateConst {
                game_inst_name: left_game_inst_name,
                pkg_inst_name,
                oracle_name,
                variant: "old",
            };

            out.push(decl_intermediate_state_left.declare());
        }

        let mut processed = HashSet::new();
        for SplitExport(pkg_inst_offs, oracle_sig) in &right_game.split_exports {
            let pkg_inst_name = &right_game.pkgs[*pkg_inst_offs].name;
            let oracle_name = &oracle_sig.name;

            if processed.contains(oracle_name) {
                continue;
            }

            processed.insert(oracle_name);

            let decl_intermediate_state_right = patterns::IntermediateStateConst {
                game_inst_name: right_game_inst_name,
                pkg_inst_name,
                oracle_name,
                variant: "old",
            };

            out.push(decl_intermediate_state_right.declare());
        }

        /////// arguments for non-split and split oracles

        let left_partial_datatypes = into_partial_dtypes(self.split_info_left());
        let right_partial_datatypes = into_partial_dtypes(self.split_info_right());

        // write declarations of arguments for the exports in left
        for Export(_, sig) in &left.game().exports {
            if let Some(orcl_ctx) = gctx_left.exported_oracle_ctx_by_name(&sig.name) {
                for (arg_name, arg_type) in &sig.args {
                    out.push(declare::declare_const(
                        orcl_ctx.smt_arg_name(arg_name),
                        arg_type,
                    ));
                }
            }

            if let Some(partial_dtype) = left_partial_datatypes
                .iter()
                .find(|dtype| dtype.real_oracle_sig.name == sig.name)
            {
                let orcl_ctx = gctx_left
                    .exported_split_oracle_ctx_by_name(&sig.name, &partial_dtype)
                    .unwrap();
                for (arg_name, arg_type) in &sig.args {
                    out.push(declare::declare_const(
                        orcl_ctx.smt_arg_name(arg_name),
                        arg_type,
                    ));
                }
            }
        }

        // write declarations of arguments for the split oracles of the right.
        // the non-split ones are shared between games and have already been
        // added for the loop above.
        for Export(_, sig) in &right.game().exports {
            if let Some(partial_dtype) = right_partial_datatypes
                .iter()
                .find(|dtype| dtype.real_oracle_sig.name == sig.name)
            {
                let orcl_ctx = gctx_right
                    .exported_split_oracle_ctx_by_name(&sig.name, &partial_dtype)
                    .unwrap();
                for (arg_name, arg_type) in &sig.args {
                    out.push(declare::declare_const(
                        orcl_ctx.smt_arg_name(arg_name),
                        arg_type,
                    ));
                }
            }
        }

        ////// return values

        for (decl_ret, constrain) in build_returns(left) {
            out.push(decl_ret);
            out.push(constrain);
        }

        for (decl_ret, constrain) in build_returns(right) {
            out.push(decl_ret);
            out.push(constrain);
        }

        for (decl_ret, constrain) in build_partial_returns(&left, &left_partial_datatypes) {
            out.push(decl_ret);
            out.push(constrain);
        }

        for (decl_ret, constrain) in build_partial_returns(&right, &right_partial_datatypes) {
            out.push(decl_ret);
            out.push(constrain);
        }

        /////// randomess counters

        for (decl_ctr, assert_ctr, decl_val, assert_val) in
            build_rands(self.sample_info_left(), left)
        {
            out.push(decl_ctr);
            out.push(assert_ctr);
            out.push(decl_val);
            out.push(assert_val);
        }

        for (decl_ctr, assert_ctr, decl_val, assert_val) in
            build_rands(self.sample_info_right(), right)
        {
            out.push(decl_ctr);
            out.push(assert_ctr);
            out.push(decl_val);
            out.push(assert_val);
        }

        /////////// helpers for working with randomness

        out.push(self.smt_define_randctr_function(left, self.sample_info_left()));
        out.push(self.smt_define_randctr_function(right, self.sample_info_right()));
        out.push(self.smt_define_randeq_function());

        ///// write expressions

        for expr in out {
            comm.write_smt(expr)?;
        }

        Ok(())
    }

    fn emit_invariant(&self, comm: &mut Communicator, oracle_name: &str) -> Result<()> {
        for file_name in &self.eq.invariants_by_oracle_name(oracle_name) {
            println!("reading file {file_name}");
            let file_contents = std::fs::read_to_string(file_name).map_err(|err| {
                let file_name = file_name.clone();
                error::new_invariant_file_read_error(oracle_name.to_string(), file_name, err)
            })?;
            println!("read file {file_name}");
            write!(comm, "{file_contents}").unwrap();
            println!("wrote contents of file {file_name}");

            if comm.check_sat()? != ProverResponse::Sat {
                return Err(Error::UnsatAfterInvariantRead {
                    equivalence: self.eq.clone(),
                    oracle_name: oracle_name.to_string(),
                });
            }
        }

        Ok(())
    }

    fn split_oracle_sig_by_exported_name(
        &'a self,
        oracle_name: &str,
    ) -> Option<&'a SplitOracleSig> {
        let gctx_left = self.left_game_ctx();

        gctx_left
            .game()
            .split_exports
            .iter()
            .find(|SplitExport(_, sig)| &sig.name == oracle_name)
            .map(|SplitExport(inst_idx, _)| {
                gctx_left.game().pkgs[*inst_idx]
                    .pkg
                    .split_oracles
                    .iter()
                    .find(|odef| &odef.sig.name == oracle_name)
                    .map(|odef| &odef.sig)
            })
            .flatten()
    }

    fn emit_split_claim_assert(
        &self,
        comm: &mut Communicator,
        oracle_name: &str,
        path: &SplitPath,
        claim: &Claim,
    ) -> Result<()> {
        println!("name: {oracle_name}");
        println!("path: {path:?}");

        let gctx_left = self.left_game_ctx();
        let gctx_right = self.right_game_ctx();

        let game_inst_name_left = self.eq.left_name();
        let game_inst_name_right = self.eq.right_name();

        let game_name_left = &gctx_left.game().name;
        let game_name_right = &gctx_right.game().name;

        let pkg_inst_ctx_left = gctx_left
            .pkg_inst_ctx_by_exported_split_oracle_name(oracle_name)
            .unwrap();
        let pkg_inst_ctx_right = gctx_right
            .pkg_inst_ctx_by_exported_split_oracle_name(oracle_name)
            .unwrap();

        let pkg_inst_name_left = pkg_inst_ctx_left.pkg_inst_name();
        let pkg_inst_name_right = pkg_inst_ctx_right.pkg_inst_name();

        let args: Vec<_> = self
            .split_oracle_sig_by_exported_name(oracle_name)
            .unwrap()
            .args
            .iter()
            .map(|(arg_name, arg_type)| patterns::OracleArgs {
                oracle_name,
                arg_name,
                arg_type,
            })
            .collect();

        // find the package instance which is marked as exporting
        // the oracle of this name, both left and right.
        let left_partial_return_name = patterns::PartialReturnConst {
            game_inst_name: game_inst_name_left,
            pkg_inst_name: pkg_inst_name_left,
            oracle_name,
        }
        .name();

        let right_partial_return_name = patterns::PartialReturnConst {
            game_inst_name: game_inst_name_right,
            pkg_inst_name: pkg_inst_name_right,
            oracle_name,
        }
        .name();

        let state_left_new_name = patterns::GameState {
            game_inst_name: game_inst_name_left,
            variant: &format!("new-{oracle_name}"),
        }
        .name();

        let state_left_old_name = patterns::GameState {
            game_inst_name: game_inst_name_left,
            variant: "old",
        }
        .name();

        let state_right_new_name = patterns::GameState {
            game_inst_name: game_inst_name_right,
            variant: &format!("new-{oracle_name}"),
        }
        .name();

        let state_right_old_name = patterns::GameState {
            game_inst_name: game_inst_name_right,
            variant: "old",
        }
        .name();

        let intermediate_state_left_new_name = patterns::IntermediateStateConst {
            game_inst_name: game_inst_name_left,
            pkg_inst_name: pkg_inst_name_left,
            oracle_name,
            variant: &format!("new-{oracle_name}"),
        }
        .name();

        let intermediate_state_left_old_name = patterns::IntermediateStateConst {
            game_inst_name: game_inst_name_left,
            pkg_inst_name: pkg_inst_name_left,
            oracle_name,
            variant: "old",
        }
        .name();

        let intermediate_state_right_new_name = patterns::IntermediateStateConst {
            game_inst_name: game_inst_name_right,
            pkg_inst_name: pkg_inst_name_right,
            oracle_name,
            variant: &format!("new-{oracle_name}"),
        }
        .name();

        let intermediate_state_right_old_name = patterns::IntermediateStateConst {
            game_inst_name: game_inst_name_right,
            pkg_inst_name: pkg_inst_name_right,
            oracle_name,
            variant: "old",
        }
        .name();

        let randomness_mapping = SmtForall {
            bindings: vec![
                ("randmap-sample-id-left".into(), Type::Integer.into()),
                ("randmap-sample-ctr-left".into(), Type::Integer.into()),
                ("randmap-sample-id-right".into(), Type::Integer.into()),
                ("randmap-sample-ctr-right".into(), Type::Integer.into()),
            ],
            body: (
                "=>",
                (
                    format!("randomness-mapping-{oracle_name}"),
                    (
                        format!("get-rand-ctr-{game_name_left}"),
                        "randmap-sample-id-left",
                    ),
                    (
                        format!("get-rand-ctr-{game_name_right}"),
                        "randmap-sample-id-right",
                    ),
                    "randmap-sample-id-left",
                    "randmap-sample-id-right",
                    "randmap-sample-ctr-left",
                    "randmap-sample-ctr-right",
                ),
                (
                    "rand-is-eq",
                    "randmap-sample-id-left",
                    "randmap-sample-id-right",
                    "randmap-sample-ctr-left",
                    "randmap-sample-ctr-right",
                ),
            ),
        };

        // this helper builds an smt expression that calls the
        // function with the given name with the old states,
        // return values and the respective arguments.
        // We expect that function to return a boolean, which makes
        // it a relation.
        let build_lemma_call = |name: &str| {
            let mut tmp: Vec<SmtExpr> = vec![
                name.into(),
                state_left_old_name.clone().into(),
                state_right_old_name.clone().into(),
                intermediate_state_left_old_name.clone().into(),
                intermediate_state_right_old_name.clone().into(),
                left_partial_return_name.clone().into(),
                right_partial_return_name.clone().into(),
            ];

            for arg in args {
                tmp.push(arg.name().into());
            }

            SmtExpr::List(tmp)
        };

        let build_relation_call = |name: &str| -> SmtExpr {
            (
                name,
                &state_left_new_name,
                &state_right_new_name,
                &intermediate_state_left_new_name,
                &intermediate_state_right_new_name,
            )
                .into()
        };

        let build_invariant_old_call = |name: &str| -> SmtExpr {
            (
                name,
                &state_left_old_name,
                &state_right_old_name,
                &intermediate_state_left_old_name,
                &intermediate_state_right_old_name,
            )
                .into()
        };

        let build_invariant_new_call = |name: &str| -> SmtExpr {
            (
                name,
                &state_left_new_name,
                &state_right_new_name,
                &intermediate_state_left_new_name,
                &intermediate_state_right_new_name,
            )
                .into()
        };

        let dep_calls: Vec<_> = claim
            .dependencies()
            .iter()
            .map(|dep_name| {
                let claim_type = ClaimType::guess_from_name(dep_name);
                match claim_type {
                    ClaimType::Lemma => build_lemma_call.clone()(&dep_name),
                    ClaimType::Relation => build_relation_call.clone()(&dep_name),
                    ClaimType::Invariant => unreachable!(),
                }
            })
            .collect();

        let postcond_call = match claim.tipe {
            ClaimType::Lemma => build_lemma_call.clone()(&claim.name),
            ClaimType::Relation => build_relation_call.clone()(&claim.name),
            ClaimType::Invariant => build_invariant_new_call.clone()(&claim.name),
        };

        let mut dependencies_code: Vec<SmtExpr> = vec![
            randomness_mapping.into(),
            build_invariant_old_call.clone()(&format!("invariant-{oracle_name}")),
        ];

        for dep in dep_calls {
            dependencies_code.push(dep)
        }

        comm.write_smt(crate::writers::smt::exprs::SmtAssert(SmtNot(SmtImplies(
            SmtAnd(dependencies_code),
            postcond_call,
        ))))?;

        Ok(())
    }

    fn oracle_sig_by_exported_name(&'a self, oracle_name: &str) -> Option<&'a OracleSig> {
        let gctx_left = self.left_game_ctx();

        gctx_left
            .game()
            .exports
            .iter()
            .find(|Export(_, sig)| &sig.name == oracle_name)
            .map(|Export(inst_idx, _)| {
                gctx_left.game().pkgs[*inst_idx]
                    .pkg
                    .oracles
                    .iter()
                    .find(|odef| &odef.sig.name == oracle_name)
                    .map(|odef| &odef.sig)
            })
            .flatten()
    }

    fn emit_no_abort_claim_definition(
        &self,
        comm: &mut Communicator,
        oracle_name: &str,
    ) -> Result<()> {
        let gctx_left = self.left_game_ctx();
        let gctx_right = self.right_game_ctx();

        let game_inst_name_left = self.eq.left_name();
        let game_inst_name_right = self.eq.right_name();

        let game_name_left = &gctx_left.game().name;
        let game_name_right = &gctx_right.game().name;

        let octx_left = gctx_left.exported_oracle_ctx_by_name(oracle_name).unwrap();
        let octx_right = gctx_right.exported_oracle_ctx_by_name(oracle_name).unwrap();

        let pkg_inst_name_left = octx_left.pkg_inst_ctx().pkg_inst_name();
        let pkg_inst_name_right = octx_right.pkg_inst_ctx().pkg_inst_name();

        todo!()
    }

    fn emit_claim_assert(
        &self,
        comm: &mut Communicator,
        oracle_name: &str,
        claim: &Claim,
    ) -> Result<()> {
        let gctx_left = self.left_game_ctx();
        let gctx_right = self.right_game_ctx();

        let game_inst_name_left = self.eq.left_name();
        let game_inst_name_right = self.eq.right_name();

        let game_name_left = &gctx_left.game().name;
        let game_name_right = &gctx_right.game().name;

        let octx_left = gctx_left.exported_oracle_ctx_by_name(oracle_name).unwrap();
        let octx_right = gctx_right.exported_oracle_ctx_by_name(oracle_name).unwrap();

        let pkg_inst_name_left = octx_left.pkg_inst_ctx().pkg_inst_name();
        let pkg_inst_name_right = octx_right.pkg_inst_ctx().pkg_inst_name();

        let args: Vec<_> = self
            .oracle_sig_by_exported_name(oracle_name)
            .unwrap()
            .args
            .iter()
            .map(|(arg_name, arg_type)| patterns::OracleArgs {
                oracle_name,
                arg_name,
                arg_type,
            })
            .collect();

        // find the package instance which is marked as exporting
        // the oracle of this name, both left and right.
        let left_return = patterns::ReturnConst {
            game_inst_name: game_inst_name_left,
            pkg_inst_name: pkg_inst_name_left,
            oracle_name,
        };

        let right_return = patterns::ReturnConst {
            game_inst_name: game_inst_name_right,
            pkg_inst_name: pkg_inst_name_right,
            oracle_name,
        };

        let state_left_new = patterns::GameState {
            game_inst_name: game_inst_name_left,
            variant: &format!("new-{oracle_name}"),
        };

        let state_left_old = patterns::GameState {
            game_inst_name: game_inst_name_left,
            variant: "old",
        };

        let state_right_new = patterns::GameState {
            game_inst_name: game_inst_name_right,
            variant: &format!("new-{oracle_name}"),
        };

        let state_right_old = patterns::GameState {
            game_inst_name: game_inst_name_right,
            variant: "old",
        };

        // this helper builds an smt expression that calls the
        // function with the given name with the old states,
        // return values and the respective arguments.
        // We expect that function to return a boolean, which makes
        // it a relation.
        let build_lemma_call = |name: &str| {
            let mut tmp: Vec<SmtExpr> = vec![
                name.into(),
                state_left_old.name().into(),
                state_right_old.name().into(),
                left_return.name().into(),
                right_return.name().into(),
            ];

            for arg in args {
                tmp.push(arg.name().into());
            }

            SmtExpr::List(tmp)
        };

        let build_relation_call = |name: &str| -> SmtExpr {
            (name, &state_left_new.name(), &state_right_new.name()).into()
        };

        let build_invariant_old_call = |name: &str| -> SmtExpr {
            (name, &state_left_old.name(), &state_right_old.name()).into()
        };

        let build_invariant_new_call = |name: &str| -> SmtExpr {
            (name, &state_left_new.name(), &state_right_new.name()).into()
        };

        let dep_calls: Vec<_> = claim
            .dependencies()
            .iter()
            .map(|dep_name| {
                let claim_type = ClaimType::guess_from_name(dep_name);
                match claim_type {
                    ClaimType::Lemma => build_lemma_call.clone()(&dep_name),
                    ClaimType::Relation => build_relation_call.clone()(&dep_name),
                    ClaimType::Invariant => unreachable!(),
                }
            })
            .collect();

        let postcond_call = match claim.tipe {
            ClaimType::Lemma => build_lemma_call.clone()(&claim.name),
            ClaimType::Relation => build_relation_call.clone()(&claim.name),
            ClaimType::Invariant => build_invariant_new_call.clone()(&claim.name),
        };

        let randomness_mapping = SmtForall {
            bindings: vec![
                ("randmap-sample-id-left".into(), Type::Integer.into()),
                ("randmap-sample-ctr-left".into(), Type::Integer.into()),
                ("randmap-sample-id-right".into(), Type::Integer.into()),
                ("randmap-sample-ctr-right".into(), Type::Integer.into()),
            ],
            body: (
                "=>",
                (
                    format!("randomness-mapping-{oracle_name}"),
                    (
                        format!("get-rand-ctr-{game_name_left}"),
                        "randmap-sample-id-left",
                    ),
                    (
                        format!("get-rand-ctr-{game_name_right}"),
                        "randmap-sample-id-right",
                    ),
                    "randmap-sample-id-left",
                    "randmap-sample-id-right",
                    "randmap-sample-ctr-left",
                    "randmap-sample-ctr-right",
                ),
                (
                    "rand-is-eq",
                    "randmap-sample-id-left",
                    "randmap-sample-id-right",
                    "randmap-sample-ctr-left",
                    "randmap-sample-ctr-right",
                ),
            ),
        };

        let mut dependencies_code: Vec<SmtExpr> = vec![
            randomness_mapping.into(),
            build_invariant_old_call.clone()(&format!("invariant-{oracle_name}")),
        ];

        for dep in dep_calls {
            dependencies_code.push(dep)
        }

        comm.write_smt(crate::writers::smt::exprs::SmtAssert(SmtNot(SmtImplies(
            SmtAnd(dependencies_code),
            postcond_call,
        ))))?;

        Ok(())
    }

    fn types(&self) -> Vec<Type> {
        let aux_resolver = SliceResolver(&self.auxs);
        let (_, (_, types_left, _, _)) = aux_resolver.resolve(self.eq.left_name()).unwrap();
        let (_, (_, types_right, _, _)) = aux_resolver.resolve(self.eq.right_name()).unwrap();
        let mut types: Vec<_> = types_left.union(types_right).cloned().collect();
        types.sort();
        types
    }

    fn left_game_ctx(&'a self) -> contexts::GameInstanceContext<'a> {
        let game_inst_name = self.eq.left_name();
        let insts = self.proof.instances();
        let resolver: SliceResolver<'a, _> = SliceResolver(insts);
        let game_inst: &'a _ = resolver.resolve(game_inst_name).unwrap();
        contexts::GameInstanceContext::new(game_inst)
    }

    fn right_game_ctx(&self) -> contexts::GameInstanceContext<'a> {
        let game_inst_name = self.eq.right_name();
        let game_inst = SliceResolver(self.proof.instances())
            .resolve(game_inst_name)
            .unwrap();
        contexts::GameInstanceContext::new(game_inst)
    }

    fn sample_info_left(&self) -> &'a SampleInfo {
        let aux_resolver = SliceResolver(&self.auxs);
        let (_, (_, _, sample_info, _)) = aux_resolver.resolve(self.eq.left_name()).unwrap();
        sample_info
    }

    fn sample_info_right(&self) -> &'a SampleInfo {
        let aux_resolver = SliceResolver(&self.auxs);
        let (_, (_, _, sample_info, _)) = aux_resolver.resolve(self.eq.right_name()).unwrap();
        sample_info
    }

    fn split_info_left(&self) -> &'a Vec<SplitInfoEntry> {
        let aux_resolver = SliceResolver(&self.auxs);
        let (_, (_, _, _, split_info)) = aux_resolver.resolve(self.eq.left_name()).unwrap();
        split_info
    }

    fn split_info_right(&self) -> &'a Vec<SplitInfoEntry> {
        let aux_resolver = SliceResolver(&self.auxs);
        let (_, (_, _, _, split_info)) = aux_resolver.resolve(self.eq.right_name()).unwrap();
        split_info
    }

    fn oracle_sequence(&self) -> Vec<&'a OracleSig> {
        let game_inst = SliceResolver(self.proof.instances())
            .resolve(self.eq.left_name())
            .unwrap();

        println!("oracle sequence: {:?}", game_inst.game().exports);

        game_inst
            .game()
            .exports
            .iter()
            .map(|Export(_, oracle_sig)| oracle_sig)
            .collect()
    }

    fn split_oracle_sequence(&self) -> Vec<&'a SplitOracleSig> {
        let game_inst = SliceResolver(self.proof.instances())
            .resolve(self.eq.left_name())
            .unwrap();

        println!("oracle sequence: {:?}", game_inst.game().exports);

        game_inst
            .game()
            .split_exports
            .iter()
            .map(|SplitExport(_, split_oracle_sig)| split_oracle_sig)
            .collect()
    }

    pub fn smt_define_randctr_function(
        &self,
        game_inst: &GameInstance,
        sample_info: &SampleInfo,
    ) -> SmtExpr {
        let game = game_inst.game();
        let game_inst_name = game_inst.as_name();
        let game_name = &game.name;

        let state_name = patterns::GameState {
            game_inst_name,
            variant: "old",
        }
        .name();

        let pattern = patterns::GameStatePattern { game_inst_name };
        let info = patterns::GameStateDeclareInfo {
            game_inst,
            sample_info,
        };

        let spec = pattern.datastructure_spec(&info);
        let (_, selectors) = &spec.0[0];

        let mut body = SmtExpr::Atom("0".to_string());

        for selector in selectors {
            body = match selector {
                patterns::GameStateSelector::Randomness { sample_id } => SmtIte {
                    cond: ("=", "sample-id", *sample_id),
                    then: (pattern.selector_name(selector), state_name.clone()),
                    els: body,
                }
                .into(),
                _ => body,
            };
        }

        (
            "define-fun",
            format!("get-rand-ctr-{game_name}"),
            (("sample-id", Type::Integer),),
            "Int",
            body,
        )
            .into()
    }

    pub fn smt_define_randeq_function(&self) -> SmtExpr {
        let left_game = self.left_game_ctx().game();
        let right_game = self.right_game_ctx().game();

        let left_game_name = &left_game.name;
        let right_game_name = &right_game.name;

        /*
         *
         *
         * (= (randfn_left left-id left-ctr) (randfn-right right-id right-ctr)))
         *
         * if ( = left-id 3) (randfn-Int id ctr) else if ( )
         *
         *
         * if (or [cases left is type A and right is type A]) (= (fn left id ctr) fn right id ctr)
         *
         */

        let left_positions = &self.sample_info_left().positions;
        let right_positions = &self.sample_info_right().positions;

        let left_types: HashSet<Type> =
            HashSet::from_iter(self.sample_info_left().tipes.iter().cloned());
        let right_types: HashSet<Type> =
            HashSet::from_iter(self.sample_info_right().tipes.iter().cloned());

        let types: Vec<&Type> = left_types.intersection(&right_types).collect();

        let mut left_positions_by_type: HashMap<_, Vec<_>> = HashMap::new();
        let mut right_positions_by_type: HashMap<_, Vec<_>> = HashMap::new();

        for pos in left_positions {
            left_positions_by_type
                .entry(&pos.tipe)
                .or_default()
                .push(pos);
        }

        for pos in right_positions {
            right_positions_by_type
                .entry(&pos.tipe)
                .or_default()
                .push(pos);
        }

        let mut body: SmtExpr = false.into();

        for tipe in types {
            let sort: SmtExpr = tipe.into();

            let left_has_type = left_positions_by_type
                .entry(tipe)
                .or_default()
                .iter()
                .map(|Position { sample_id, .. }| ("=", *sample_id, "sample-id-left").into());
            let mut left_or_case: Vec<SmtExpr> = vec!["or".into()];
            left_or_case.extend(left_has_type);

            let right_has_type = right_positions_by_type
                .entry(tipe)
                .or_default()
                .iter()
                .map(|Position { sample_id, .. }| ("=", *sample_id, "sample-id-right").into());

            let mut right_or_case: Vec<SmtExpr> = vec!["or".into()];
            right_or_case.extend(right_has_type);

            body = SmtIte {
                cond: SmtAnd(vec![
                    SmtExpr::List(left_or_case),
                    SmtExpr::List(right_or_case),
                ]),
                then: (
                    "=",
                    (
                        format!("__sample-rand-{left_game_name}-{sort}"),
                        "sample-id-left",
                        "sample-ctr-left",
                    ),
                    (
                        format!("__sample-rand-{right_game_name}-{sort}"),
                        "sample-id-right",
                        "sample-ctr-right",
                    ),
                ),
                els: body,
            }
            .into()
        }

        (
            "define-fun",
            format!("rand-is-eq"),
            (
                ("sample-id-left", Type::Integer),
                ("sample-id-right", Type::Integer),
                ("sample-ctr-left", Type::Integer),
                ("sample-ctr-right", Type::Integer),
            ),
            "Bool",
            body,
        )
            .into()
    }
}

// ResolvedEquivalence contains the composisitions/games and the invariant data,
// whereas the pure Equivalence just contains the names and file paths.
// TODO: explore if we can keep references to the games in the project hashmap

// This function gets the parameter names that both have and checks that
// both use the same types.
// The invariant should be used to make assertions on equality or other
// relations between the two.
//
// TODO figure out if there is a better mechanism we could use here
fn check_matching_parameters(left: &GameInstance, right: &GameInstance) -> Result<()> {
    use std::collections::hash_map::RandomState;

    // populate tables name -> type
    let left_params: HashMap<_, _, RandomState> =
        HashMap::from_iter(left.game().consts.clone().into_iter());
    let right_params: HashMap<_, _, RandomState> =
        HashMap::from_iter(right.game().consts.clone().into_iter());

    // prepare sets of names
    let left_params_set = HashSet::<_, RandomState>::from_iter(left_params.keys());
    let right_params_set = HashSet::<_, RandomState>::from_iter(right_params.keys());
    let common_params = left_params_set.intersection(&right_params_set);

    // check that the common params have the same type
    for param in common_params {
        if left_params[*param] != right_params[*param] {
            return Err(Error::CompositionParamMismatch {
                left_game_inst_name: left.name().to_string(),
                right_game_inst_name: right.name().to_string(),
                mismatching_param_name: (*param).clone(),
            });
        }
    }

    Ok(())
}

fn build_partial_returns(
    game_inst: &GameInstance,
    partial_dtypes: &[PartialsDatatype],
) -> Vec<(SmtExpr, SmtExpr)> {
    let gctx = contexts::GameInstanceContext::new(game_inst);
    let game = gctx.game();
    let game_inst_name = game_inst.name();

    let mut seen = HashSet::new();

    // write declarations of right return constants and constrain them
    game.split_exports
        .iter()
        .filter(|SplitExport(_, sig)| {
            let out = !seen.contains(&sig.name);
            seen.insert(&sig.name);
            out
        })
        .map(|SplitExport(inst_idx, sig)| {
            let game_name = &game.name;
            let pkg_inst_name = &game.pkgs[*inst_idx].name;
            let oracle_name = &sig.name;
            let pattern = patterns::PartialReturnConst {
                game_inst_name,
                pkg_inst_name,
                oracle_name,
            };

            let partial_dtype = partial_dtypes
                .iter()
                .find(|partials| {
                    &partials.pkg_inst_name == pkg_inst_name
                        && &partials.real_oracle_sig.name == oracle_name
                })
                .unwrap();

            let octx = gctx
                .exported_split_oracle_ctx_by_name(&sig.name, &partial_dtype)
                .expect(&format!(
                    "error looking up exported oracle with name {oracle_name} in game {game_name}"
                ));

            let args = sig
                .args
                .iter()
                .map(|(arg_name, _)| octx.smt_arg_name(arg_name));

            let game_state_name = patterns::GameState {
                game_inst_name,
                variant: "old",
            }
            .name();

            let intermediate_state_name = patterns::IntermediateStateConst {
                game_inst_name,
                pkg_inst_name,
                oracle_name,
                variant: "old",
            }
            .name();

            let invok = octx
                .smt_invoke_oracle(game_state_name, intermediate_state_name, args)
                .unwrap();

            let partial_return_name = pattern.name();
            let constrain_return: SmtExpr = SmtAssert(SmtEq2 {
                lhs: partial_return_name,
                rhs: invok,
            })
            .into();

            (pattern.declare(), constrain_return)
        })
        .collect()
}
fn build_returns(game: &GameInstance) -> Vec<(SmtExpr, SmtExpr)> {
    let gctx = contexts::GameInstanceContext::new(game);
    let game_name = &game.game().name;
    let game_inst_name = &game.name();

    // write declarations of right return constants and constrain them
    let mut out = vec![];
    for Export(inst_idx, sig) in &game.game().exports {
        let pkg_inst_name = &game.game().pkgs[*inst_idx].name;
        let oracle_name = &sig.name;
        let return_type = &sig.tipe;

        let octx = gctx.exported_oracle_ctx_by_name(&sig.name).expect(&format!(
            "error looking up exported oracle with name {oracle_name} in game {game_name}"
        ));

        let return_const = patterns::ReturnConst {
            game_inst_name,
            pkg_inst_name,
            oracle_name,
        };
        let return_value_const = patterns::ReturnValueConst {
            game_inst_name,
            pkg_inst_name,
            oracle_name,
            tipe: &sig.tipe,
        };
        let old_state_const = patterns::GameState {
            game_inst_name,
            variant: "old",
        };
        let new_state_const = patterns::GameState {
            game_inst_name,
            variant: &format!("new-{oracle_name}"),
        };

        let args = sig
            .args
            .iter()
            .map(|(arg_name, _)| octx.smt_arg_name(arg_name));

        let oracle_func_evaluation = octx
            .smt_invoke_oracle(old_state_const.name(), args)
            .unwrap();

        let return_pattern = patterns::ReturnPattern {
            game_inst_name,
            pkg_inst_name,
            oracle_name,
        };
        let return_spec = return_pattern.datastructure_spec(&return_type);

        let access_returnvalue = return_pattern
            .access(
                &return_spec,
                &patterns::ReturnSelector::ReturnValueOrAbort {
                    return_type: &sig.tipe,
                },
                return_const.name(),
            )
            .unwrap();

        let access_new_state = return_pattern
            .access(
                &return_spec,
                &patterns::ReturnSelector::GameState,
                return_const.name(),
            )
            .unwrap();

        let constrain_return = SmtAssert(SmtEq2 {
            lhs: return_const.name(),
            rhs: oracle_func_evaluation,
        });

        let constrain_return_value = SmtAssert(SmtEq2 {
            lhs: return_value_const.name(),
            rhs: access_returnvalue,
        });

        let constrain_new_state = SmtAssert(SmtEq2 {
            lhs: new_state_const.name(),
            rhs: access_new_state,
        });

        out.push((return_const.declare(), constrain_return.into()));
        out.push((return_value_const.declare(), constrain_return_value.into()));
        out.push((new_state_const.declare(), constrain_new_state.into()));
    }

    out
}

fn build_rands(
    sample_info: &SampleInfo,
    game_inst: &GameInstance,
) -> Vec<(SmtExpr, SmtExpr, SmtExpr, SmtExpr)> {
    let gctx = contexts::GameInstanceContext::new(game_inst);

    sample_info
        .positions
        .iter()
        .map(|sample_item| {
            let sample_id = sample_item.sample_id;
            let tipe = &sample_item.tipe;
            let game_inst_name = game_inst.name();
            let game_name = &game_inst.game().name;

            let state = patterns::GameState {
                game_inst_name,
                variant: "old",
            }
            .name();

            let randctr_name = format!("randctr-{game_inst_name}-{sample_id}");
            let randval_name = format!("randval-{game_inst_name}-{sample_id}");

            let decl_randctr = declare::declare_const(randctr_name.clone(), Type::Integer);
            let decl_randval = declare::declare_const(randval_name.clone(), tipe);

            // pull randomness counter for given sample_id out of the gamestate
            let randctr = gctx
                .smt_access_gamestate_rand(sample_info, state, sample_id)
                .unwrap();

            let constrain_randctr: SmtExpr = SmtAssert(SmtEq2 {
                lhs: randctr_name.as_str(),
                rhs: randctr.clone(),
            })
            .into();

            // apply respective randomness function (based on type) to the given counter
            let randval = gctx.smt_eval_randfn(sample_id, ("+", 0, randctr_name.as_str()), tipe);

            let constrain_randval: SmtExpr = SmtAssert(SmtEq2 {
                lhs: randval_name,
                rhs: randval,
            })
            .into();

            (
                decl_randctr,
                constrain_randctr,
                decl_randval,
                constrain_randval,
            )
        })
        .collect()
}
