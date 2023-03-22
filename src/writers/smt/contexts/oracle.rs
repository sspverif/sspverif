use crate::expressions::Expression;
use crate::identifier::Identifier;
use crate::package::OracleDef;
use crate::statement::{CodeBlock, Statement};
use crate::types::Type;

use super::super::exprs::SmtExpr;
use super::super::{declare, names};

use super::{OracleContext, PackageInstanceContext};

impl<'a> OracleContext<'a> {
    pub fn pkg_inst_ctx(&self) -> PackageInstanceContext {
        PackageInstanceContext {
            game_ctx: self.game_ctx.clone(),
            inst_offs: self.inst_offs,
        }
    }

    pub fn oracle_def(&self) -> &OracleDef {
        &self.game_ctx.game.pkgs[self.inst_offs].pkg.oracles[self.oracle_offs]
    }

    pub fn smt_sort_return(&self) -> SmtExpr {
        let game = self.game_ctx.game;
        let inst = &game.pkgs[self.inst_offs];
        let osig = &inst.pkg.oracles[self.oracle_offs].sig;

        let game_name = &game.name;
        let inst_name = &inst.name;
        let oracle_name = &osig.name;

        names::return_sort_name(game_name, inst_name, oracle_name).into()
    }

    pub fn smt_declare_return(&self) -> SmtExpr {
        let game = self.game_ctx.game;
        let inst = &game.pkgs[self.inst_offs];
        let osig = &inst.pkg.oracles[self.oracle_offs].sig;

        let game_name = &game.name;
        let inst_name = &inst.name;
        let oracle_name = &osig.name;

        use crate::types::Type;

        let fields = vec![
            (
                names::return_selector_state_name(game_name, inst_name, oracle_name),
                self.game_ctx.smt_sort_gamestates(),
            ),
            (
                names::return_selector_state_length_name(game_name, inst_name, oracle_name),
                Type::Integer.into(),
            ),
            (
                names::return_selector_value_name(game_name, inst_name, oracle_name),
                Type::Maybe(Box::new(osig.tipe.clone())).into(),
            ),
            (
                names::return_selector_is_abort_name(game_name, inst_name, oracle_name),
                Type::Boolean.into(),
            ),
        ];

        declare::declare_single_constructor_datatype(
            &names::return_sort_name(game_name, inst_name, oracle_name),
            &names::return_constructor_name(game_name, inst_name, oracle_name),
            fields.into_iter(),
        )
    }

    pub fn smt_construct_return<S, SL, V, ISAB>(
        &self,
        state: S,
        state_len: SL,
        value: V,
        is_abort: ISAB,
    ) -> SmtExpr
    where
        S: Into<SmtExpr>,
        SL: Into<SmtExpr>,
        V: Into<SmtExpr>,
        ISAB: Into<SmtExpr>,
    {
        let game = self.game_ctx.game;
        let inst = &game.pkgs[self.inst_offs];
        let osig = &inst.pkg.oracles[self.oracle_offs].sig;

        let game_name = &game.name;
        let inst_name = &inst.name;
        let oracle_name = &osig.name;

        (
            names::return_constructor_name(game_name, inst_name, oracle_name),
            state,
            state_len,
            value,
            is_abort,
        )
            .into()
    }

    pub fn smt_construct_abort<S, SL>(&self, state: S, state_len: SL) -> SmtExpr
    where
        S: Into<SmtExpr>,
        SL: Into<SmtExpr>,
    {
        let game = self.game_ctx.game;
        let inst = &game.pkgs[self.inst_offs];
        let osig = &inst.pkg.oracles[self.oracle_offs].sig;

        let game_name = &game.name;
        let inst_name = &inst.name;
        let oracle_name = &osig.name;

        (
            names::return_constructor_name(game_name, inst_name, oracle_name),
            state,
            state_len,
            Expression::None(osig.tipe.clone()),
            "true",
        )
            .into()
    }

    pub fn smt_access_return_state<R>(&self, ret: R) -> SmtExpr
    where
        R: Into<SmtExpr>,
    {
        let game = self.game_ctx.game;
        let inst = &game.pkgs[self.inst_offs];
        let osig = &inst.pkg.oracles[self.oracle_offs].sig;

        let game_name = &game.name;
        let inst_name = &inst.name;
        let oracle_name = &osig.name;

        (
            names::return_selector_state_name(game_name, inst_name, oracle_name),
            ret,
        )
            .into()
    }

    pub fn smt_select_return_state<R, L>(&self, ret: R, state_length: L) -> SmtExpr
    where
        R: Into<SmtExpr>,
        L: Into<SmtExpr>,
    {
        ("select", self.smt_access_return_state(ret), state_length).into()
    }

    pub fn smt_store_return_state<R: Into<SmtExpr>, L: Into<SmtExpr>, S: Into<SmtExpr>>(
        &self,
        ret: R,
        state_length: L,
        state: S,
    ) -> SmtExpr {
        (
            "store",
            self.smt_access_return_state(ret),
            state_length,
            state,
        )
            .into()
    }

    pub fn smt_access_return_state_length<R: Into<SmtExpr>>(&self, ret: R) -> SmtExpr {
        let game = self.game_ctx.game;
        let inst = &game.pkgs[self.inst_offs];
        let osig = &inst.pkg.oracles[self.oracle_offs].sig;

        let game_name = &game.name;
        let inst_name = &inst.name;
        let oracle_name = &osig.name;

        (
            names::return_selector_state_length_name(game_name, inst_name, oracle_name),
            ret,
        )
            .into()
    }

    pub fn smt_access_return_is_abort<R: Into<SmtExpr>>(&self, ret: R) -> SmtExpr {
        let game = self.game_ctx.game;
        let inst = &game.pkgs[self.inst_offs];
        let osig = &inst.pkg.oracles[self.oracle_offs].sig;

        let game_name = &game.name;
        let inst_name = &inst.name;
        let oracle_name = &osig.name;

        (
            names::return_selector_is_abort_name(game_name, inst_name, oracle_name),
            ret,
        )
            .into()
    }

    pub fn smt_access_return_value<R: Into<SmtExpr>>(&self, ret: R) -> SmtExpr {
        let game = self.game_ctx.game;
        let inst = &game.pkgs[self.inst_offs];
        let osig = &inst.pkg.oracles[self.oracle_offs].sig;

        let game_name = &game.name;
        let inst_name = &inst.name;
        let oracle_name = &osig.name;

        (
            names::return_selector_value_name(game_name, inst_name, oracle_name),
            ret,
        )
            .into()
    }

    // returns none if the wrong number of arguments were provided
    pub fn smt_invoke_oracle<S, SLEN, ARGS>(
        &self,
        gamestates: S,
        state_length: SLEN,
        args: ARGS,
    ) -> Option<SmtExpr>
    where
        S: Into<SmtExpr>,
        SLEN: Into<SmtExpr>,
        ARGS: Iterator<Item = SmtExpr>,
    {
        let game = self.game_ctx.game;
        let inst = &game.pkgs[self.inst_offs];
        let osig = &inst.pkg.oracles[self.oracle_offs].sig;

        let game_name = &game.name;
        let inst_name = &inst.name;
        let oracle_name = &osig.name;

        let expected_len = 3 + osig.args.len();

        let mut cmdline = Vec::with_capacity(expected_len);
        cmdline.push(names::oracle_function_name(game_name, inst_name, oracle_name).into());
        cmdline.push(gamestates.into());
        cmdline.push(state_length.into());
        cmdline.extend(args);

        if cmdline.len() != expected_len {
            return None;
        }

        Some(SmtExpr::List(cmdline))
    }

    // funciton die fuer jeden checkpoint eine liste aller bisher definierten variablen zurueckgibt
    // sounds like a job for scope, but that is long gone at this poitn...
    pub fn checkpoints(&self) -> Vec<(String, Vec<(String, SmtExpr)>)> {
        let game = self.game_ctx.game;
        let inst = &game.pkgs[self.inst_offs];
        let odef = &inst.pkg.oracles[self.oracle_offs];
        let CodeBlock(code) = &odef.code;

        let mut locals = vec![];
        let mut checkpoints = vec![];

        self.checkpoints_inner(code, &mut locals, &mut checkpoints, &[]);

        checkpoints
    }

    // function that generates a single datatype with one constructor per checkpoint (i.e. one of the above lists)
    pub fn smt_declare_intermediate_states(&self) -> SmtExpr {
        let game = self.game_ctx.game;
        let inst = &game.pkgs[self.inst_offs];
        let osig = &inst.pkg.oracles[self.oracle_offs].sig;

        let game_name = &game.name;
        let inst_name = &inst.name;
        let oracle_name = &osig.name;

        let checkpoints = self.checkpoints();

        declare::declare_datatype(
            &names::intermediate_oracle_state_sort_name(game_name, inst_name, oracle_name),
            checkpoints.into_iter(),
        )
    }

    fn checkpoints_inner(
        &self,
        code: &[Statement],
        locals: &mut Vec<(String, SmtExpr)>,
        checkpoints: &mut Vec<(String, Vec<(String, SmtExpr)>)>,
        path: &[usize],
    ) {
        let gctx = &self.game_ctx;
        let ictx = gctx.pkg_inst_ctx_by_offs(self.inst_offs).unwrap();
        let octx = ictx.oracle_ctx_by_oracle_offs(self.oracle_offs).unwrap();

        let game_name = &self.game_ctx.game.name;
        let inst_name = ictx.pkg_inst_name();
        let oracle_name = &octx.oracle_def().sig.name;

        for (i, stmt) in code.iter().enumerate() {
            let mut new_path = path.to_vec();
            new_path.push(i);

            let path_str: Vec<String> = new_path.iter().map(usize::to_string).collect();
            let path_str = path_str.join("-");

            match &stmt {
                Statement::Sample(id, opt_idx, _, tipe)
                | Statement::InvokeOracle {
                    id,
                    opt_idx,
                    tipe: Some(tipe),
                    ..
                } => match opt_idx {
                    Some(Expression::Typed(idx_type, _)) => locals.push((
                        id.ident(),
                        Type::Table(Box::new(tipe.clone()), Box::new(idx_type.clone())).into(),
                    )),

                    None => locals.push((id.ident(), tipe.into())),
                    Some(_) => unreachable!(),
                },

                Statement::Assign(id, opt_idx, value) => match (opt_idx, value) {
                    (Some(Expression::Typed(idx_type, _)), Expression::Typed(tipe, _)) => locals
                        .push((
                            id.ident(),
                            Type::Table(Box::new(tipe.clone()), Box::new(idx_type.clone())).into(),
                        )),

                    (None, Expression::Typed(tipe, _)) => locals.push((id.ident(), tipe.into())),
                    (Some(_), _) => unreachable!(),
                    (None, _) => unreachable!(),
                },
                Statement::Parse(ids, Expression::Typed(Type::Tuple(tipes), _)) => {
                    assert_eq!(ids.len(), tipes.len());

                    let pairs = ids
                        .iter()
                        .map(Identifier::ident)
                        .zip(tipes.iter().map(|t| t.into()));
                    locals.extend(pairs);
                }
                Statement::Parse(ids, _) => unreachable!(),

                Statement::IfThenElse(_, CodeBlock(ifcode), CodeBlock(elsecode)) => {
                    self.checkpoints_inner(ifcode, locals, checkpoints, &new_path);
                    self.checkpoints_inner(elsecode, locals, checkpoints, &new_path);
                }

                Statement::Abort => checkpoints.push((
                    names::oracle_intermediate_state_abort_constructor_name(
                        game_name,
                        inst_name,
                        oracle_name,
                        &new_path,
                    ),
                    locals
                        .iter()
                        .map(|(var_name, expr)| {
                            let name = names::oracle_intermediate_state_abort_selector_name(
                                game_name,
                                inst_name,
                                oracle_name,
                                &new_path,
                                var_name,
                            );
                            (name, expr.clone())
                        })
                        .collect(),
                )),
                Statement::Return(_) => checkpoints.push((
                    names::oracle_intermediate_state_return_constructor_name(
                        game_name,
                        inst_name,
                        oracle_name,
                        &new_path,
                    ),
                    locals
                        .iter()
                        .map(|(var_name, expr)| {
                            let name = names::oracle_intermediate_state_return_selector_name(
                                game_name,
                                inst_name,
                                oracle_name,
                                &new_path,
                                var_name,
                            );
                            (name, expr.clone())
                        })
                        .collect(),
                )),
                Statement::InvokeOracle {
                    tipe: None, name, ..
                } => checkpoints.push((
                    names::oracle_intermediate_state_oracleinvoc_constructor_name(
                        game_name,
                        inst_name,
                        &oracle_name,
                        name,
                        &path,
                    ),
                    locals
                        .iter()
                        .map(|(var_name, expr)| {
                            let name = names::oracle_intermediate_state_oracleinvoc_selector_name(
                                game_name,
                                inst_name,
                                oracle_name,
                                name,
                                &new_path,
                                var_name,
                            );
                            (name, expr.clone())
                        })
                        .collect(),
                )),
            };
        }
    }
}
