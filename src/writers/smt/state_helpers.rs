use crate::package::OracleSig;
use crate::transforms::samplify::SampleInfo;
use crate::types::Type;

use crate::writers::smt::exprs::{smt_to_string, SmtExpr, SmtLet, SspSmtVar};

/**
* composition state smt gen helper type
*
* what do we need?
* - composition state sort name
* - composition state constructor name
* - composition state accessor names
* - composition state sort definition
* - composition state accessor helpers
* - overwrite/copy-on-write helper
*
* for that we need...
* - composition name
* - SmtPackageState values for each instance
*
*
   (declare-datatype State_right-pkey (
        (mk-state-right-pkey   (state-right-pkey-pk   (Array Int String))
                               (state-right-pkey-sk   (Array Int String))
                               (state-right-pkey-id   (Array String Int))
                               (state-right-pkey-ctr  Int)
                               (state-right-pkey-rand RandState))))
*
*/

pub struct SmtCompositionContext<'a> {
    comp_name: &'a str,
    pkg_names: Vec<&'a str>,
    params: Vec<(&'a String, &'a Type)>,
    sample_info: &'a SampleInfo,
}

impl<'a> SmtCompositionContext<'a> {
    pub fn new(
        comp_name: &'a str,
        substate_names: Vec<&'a str>,
        params: &'a [(String, Type)],
        sample_info: &'a SampleInfo,
    ) -> SmtCompositionContext<'a> {
        // skip functions

        let mut params: Vec<(&String, &Type)> = params
            .iter()
            .filter(|(_, tipe)| !matches!(tipe, Type::Fn(..)))
            .map(|(a, b)| (a, b))
            .collect();
        params.sort();

        SmtCompositionContext {
            comp_name,
            pkg_names: substate_names,
            params: params,
            sample_info,
        }
    }

    pub fn smt_sort(&self) -> SmtExpr {
        format!("CompositionState-{}", self.comp_name).into()
    }

    pub fn smt_constructor(&self) -> SmtExpr {
        format!("mk-composition-state-{}", self.comp_name).into()
    }

    pub fn smt_accessor_pkg(&self, inst_name: &str) -> SmtExpr {
        // TODO should we check that inst_name is in inst_names? same for SmtPackageState
        format!("composition-pkgstate-{}-{}", self.comp_name, inst_name).into()
    }

    pub fn smt_access_pkg(&self, inst_name: &str, term: SmtExpr) -> SmtExpr {
        (self.smt_accessor_pkg(inst_name), term).into()
    }

    pub fn smt_accessor_param(&self, param_name: &str) -> SmtExpr {
        // TODO check that param exists
        // Note: when changing this, make sure you also change expr_exprs!
        format!("composition-param-{}-{}", self.comp_name, param_name).into()
    }
    pub fn smt_access_param(&self, param_name: &str, term: SmtExpr) -> SmtExpr {
        (self.smt_accessor_param(param_name), term).into()
    }

    pub fn smt_accessor_rand(&self, sample_id: usize) -> SmtExpr {
        // TODO check that sample id is in bounds
        format!("composition-rand-{}-{}", self.comp_name, sample_id).into()
    }
    pub fn smt_access_rand(&self, sample_id: usize, term: SmtExpr) -> SmtExpr {
        (self.smt_accessor_rand(sample_id), term).into()
    }

    pub fn smt_declare_datatype(&self) -> SmtExpr {
        // initialize list with constructor name
        let mut tmp = vec![self.smt_constructor()];

        // add accessors for the package states
        for inst_name in &self.pkg_names {
            let pkg_state = SmtPackageState {
                comp_name: self.comp_name,
                inst_name,
                state: vec![],
            };

            tmp.push((self.smt_accessor_pkg(inst_name), pkg_state.smt_sort()).into())
        }

        // add accessors for parameters/consts
        for (param_name, param_type) in &self.params {
            tmp.push(
                (
                    self.smt_accessor_param(param_name),
                    (*param_type).to_owned(),
                )
                    .into(),
            )
        }

        // add accessors for randomness sample counters
        for sample_id in 0..(self.sample_info.count) {
            tmp.push((self.smt_accessor_rand(sample_id), Type::Integer).into())
        }

        // build the actual declare-datatype expression
        (
            "declare-datatype",
            self.smt_sort(),
            SmtExpr::List(vec![SmtExpr::List(tmp)]),
        )
            .into()
    }

    pub fn smt_set_pkg_state(&self, target: &str, new: &SmtExpr, body: SmtExpr) -> SmtExpr {
        // initialize list with constructor name
        let mut tmp = vec![self.smt_constructor()];
        let latest_state = SmtExpr::List(vec![
            SmtExpr::Atom("select".into()),
            SspSmtVar::CompositionContext.into(),
            SspSmtVar::ContextLength.into(),
        ]);

        // add values for the package states, replacing target
        for inst_name in &self.pkg_names {
            tmp.push(if *inst_name == target {
                new.clone()
            } else {
                self.smt_access_pkg(inst_name, latest_state.clone())
            });
        }

        // copy values for parameters/consts
        for (param_name, _) in &self.params {
            tmp.push(self.smt_access_param(param_name, latest_state.clone()));
        }

        // copy values for randomness sample counters
        for sample_id in 0..(self.sample_info.count) {
            tmp.push(self.smt_access_rand(sample_id, latest_state.clone()));
        }

        SmtLet {
            bindings: vec![
                (
                    smt_to_string(SspSmtVar::CompositionContext),
                    SmtExpr::List(vec![
                        SmtExpr::Atom("store".into()),
                        SspSmtVar::CompositionContext.into(),
                        SmtExpr::List(vec![
                            SmtExpr::Atom("+".into()),
                            SmtExpr::Atom("1".into()),
                            SspSmtVar::ContextLength.into(),
                        ]),
                        SmtExpr::List(tmp),
                    ]),
                ),
                (
                    smt_to_string(SspSmtVar::ContextLength),
                    SmtExpr::List(vec![
                        SmtExpr::Atom("+".into()),
                        SmtExpr::Atom("1".into()),
                        SspSmtVar::ContextLength.into(),
                    ]),
                ),
            ],
            body,
        }
        .into()
    }

    pub fn smt_set_rand_ctr(
        &self,
        target_sample_id: usize,
        new: &SmtExpr,
        body: SmtExpr,
    ) -> SmtExpr {
        // initialize list with constructor name
        let mut tmp = vec![self.smt_constructor()];
        let latest_state = SmtExpr::List(vec![
            SmtExpr::Atom("select".into()),
            SspSmtVar::CompositionContext.into(),
            SspSmtVar::ContextLength.into(),
        ]);

        // copy values for the package states
        for inst_name in &self.pkg_names {
            tmp.push(self.smt_access_pkg(inst_name, latest_state.clone()));
        }

        // copy values for parameters/consts
        for (param_name, _) in &self.params {
            tmp.push(self.smt_access_param(param_name, latest_state.clone()));
        }

        // add values for randomness sample counters, but replace target_sample_id
        for sample_id in 0..(self.sample_info.count) {
            let new = if sample_id == target_sample_id {
                new.clone()
            } else {
                self.smt_access_rand(sample_id, latest_state.clone())
            };

            tmp.push(new);
        }

        SmtLet {
            bindings: vec![(
                smt_to_string(SspSmtVar::CompositionContext),
                SmtExpr::List(vec![
                    SmtExpr::Atom("store".into()),
                    SspSmtVar::CompositionContext.into(),
                    SspSmtVar::ContextLength.into(),
                    SmtExpr::List(tmp),
                ]),
            )],
            body,
        }
        .into()
    }
}

/**
 * packages state smt gen helper type
 *
 * what do we need?
 * - state type name ✅
 * - state type definition ✅
 * - state type constructor name ✅
 * - state type accessors ✅
 * - overwrite/copy-on-write helper ✅
 *
 * for that we need...
 * - composition name
 * - package instance name
 * - state variables
 */
#[derive(Clone, Debug)]
pub struct SmtPackageState<'a> {
    comp_name: &'a str,
    inst_name: &'a str,
    state: Vec<(String, Type)>,
}

/**
 * comp = mod_prf_game
 * inst = multi_key
 */

impl<'a> SmtPackageState<'a> {
    pub fn new(
        comp_name: &'a str,
        inst_name: &'a str,
        state: Vec<(String, Type)>,
    ) -> SmtPackageState<'a> {
        SmtPackageState {
            comp_name,
            inst_name,
            state,
        }
    }

    pub fn smt_constructor(&self) -> SmtExpr {
        SmtExpr::Atom(format!("mk-state-{}-{}", self.comp_name, self.inst_name))
    }

    pub fn smt_sort(&self) -> SmtExpr {
        SmtExpr::Atom(format!("State_{}_{}", self.comp_name, self.inst_name))
    }

    pub fn smt_accessor(&self, id: &str) -> SmtExpr {
        SmtExpr::Atom(format!(
            "state-{}-{}-{}",
            self.comp_name, self.inst_name, id
        ))
    }

    pub fn smt_access(&self, id: &str, term: SmtExpr) -> SmtExpr {
        SmtExpr::List(vec![self.smt_accessor(id), term])
    }

    pub fn smt_declare_datatype(&self) -> SmtExpr {
        let mut tmp = vec![self.smt_constructor()];

        for (id, tipe) in &self.state {
            tmp.push(SmtExpr::List(vec![
                self.smt_accessor(id),
                tipe.clone().into(),
            ]))
        }
        /*
        for (id, tipe) in &self.params {
            tmp.push(SmtExpr::List(vec![
                self.smt_accessor(id),
                tipe.clone().into(),
            ]))
        }
        */

        SmtExpr::List(vec![
            SmtExpr::Atom("declare-datatype".to_string()),
            self.smt_sort(),
            SmtExpr::List(vec![SmtExpr::List(tmp)]),
        ])
    }

    pub fn smt_set(&self, id: &str, new: &SmtExpr) -> SmtExpr {
        self.smt_set_on(id, new, &SspSmtVar::SelfState.into())
    }

    pub fn smt_set_random(&self, id: &str, new: &SmtExpr, access: &SmtExpr) -> SmtExpr {
        self.smt_set_on(id, new, access)
    }

    fn smt_set_on(&self, id: &str, new: &SmtExpr, target: &SmtExpr) -> SmtExpr {
        let mut tmp = vec![self.smt_constructor()];

        for (varname, _) in self.state.clone() {
            if varname == *id {
                tmp.push(new.clone());
            } else {
                tmp.push(SmtExpr::List(vec![
                    self.smt_accessor(&varname),
                    target.clone(),
                ]));
            }
        }

        SmtExpr::List(tmp)
    }
}

#[derive(Clone, Debug)]
pub struct SmtReturnState<'a> {
    comp_name: &'a str,
    inst_name: &'a str,
    sig: OracleSig,
}

/**
 * comp = mod_prf_game
 * inst = multi_key
 */

impl<'a> SmtReturnState<'a> {
    pub fn new(comp_name: &'a str, inst_name: &'a str, sig: OracleSig) -> SmtReturnState<'a> {
        SmtReturnState {
            comp_name,
            inst_name,
            sig,
        }
    }

    pub fn smt_declare_datatype(&self, comp_sort: SmtExpr) -> SmtExpr {
        let constructor = vec![
            self.smt_constructor_atom(),
            SmtExpr::List(vec![
                self.smt_access_states_atom(),
                SmtExpr::List(vec![
                    SmtExpr::Atom("Array".into()),
                    SmtExpr::Atom("Int".into()),
                    comp_sort,
                ]),
            ]),
            SmtExpr::List(vec![
                self.smt_access_states_length_atom(),
                SmtExpr::Atom("Int".into()),
            ]),
            SmtExpr::List(vec![
                self.smt_access_value_atom(),
                Type::Maybe(Box::new(self.sig.tipe.clone())).into(),
            ]),
            SmtExpr::List(vec![self.smt_access_is_abort_atom(), Type::Boolean.into()]),
        ];

        SmtExpr::List(vec![
            SmtExpr::Atom("declare-datatype".to_string()),
            self.smt_sort(),
            SmtExpr::List(vec![SmtExpr::List(constructor)]),
        ])
    }

    pub fn smt_constructor(
        &self,
        state: SmtExpr,
        statelen: SmtExpr,
        value: SmtExpr,
        isabort: SmtExpr,
    ) -> SmtExpr {
        SmtExpr::List(vec![
            SmtExpr::Atom(format!(
                "mk-return-{}-{}-{}",
                self.comp_name, self.inst_name, self.sig.name
            )),
            state,
            statelen,
            value,
            isabort,
        ])
    }

    pub fn smt_constructor_atom(&self) -> SmtExpr {
        SmtExpr::Atom(format!(
            "mk-return-{}-{}-{}",
            self.comp_name, self.inst_name, self.sig.name
        ))
    }

    pub fn smt_sort(&self) -> SmtExpr {
        SmtExpr::Atom(format!(
            "Return_{}_{}_{}",
            self.comp_name, self.inst_name, self.sig.name
        ))
    }

    pub fn smt_access_states_atom(&self) -> SmtExpr {
        SmtExpr::Atom(format!(
            "return-{}-{}-{}-state",
            self.comp_name, self.inst_name, self.sig.name
        ))
    }

    pub fn smt_access_states_fn(&self, on: SmtExpr) -> SmtExpr {
        SmtExpr::List(vec![self.smt_access_states_atom(), on])
    }

    pub fn smt_access_states_length_atom(&self) -> SmtExpr {
        SmtExpr::Atom(format!(
            "return-{}-{}-{}-state-length",
            self.comp_name, self.inst_name, self.sig.name
        ))
    }

    pub fn smt_access_states_length_fn(&self, on: SmtExpr) -> SmtExpr {
        SmtExpr::List(vec![self.smt_access_states_length_atom(), on])
    }

    pub fn smt_access_is_abort_atom(&self) -> SmtExpr {
        SmtExpr::Atom(format!(
            "return-{}-{}-{}-is-abort",
            self.comp_name, self.inst_name, self.sig.name
        ))
    }

    pub fn smt_access_is_abort_fn(&self, on: SmtExpr) -> SmtExpr {
        SmtExpr::List(vec![self.smt_access_is_abort_atom(), on])
    }

    pub fn smt_access_value_atom(&self) -> SmtExpr {
        SmtExpr::Atom(format!(
            "return-{}-{}-{}-value",
            self.comp_name, self.inst_name, self.sig.name
        ))
    }

    pub fn smt_access_value_fn(&self, on: SmtExpr) -> SmtExpr {
        SmtExpr::List(vec![self.smt_access_value_atom(), on])
    }
}
