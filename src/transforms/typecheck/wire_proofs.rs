use crate::{
    expressions::Expression,
    identifier::Identifier,
    package::{Composition, Edge, MultiInstanceEdge, OracleSig},
    parser::package::{ForSpec, MultiInstanceIndices},
    types::Type,
    util::resolver::Named,
    writers::smt::{
        self,
        declare::declare_const,
        exprs::{SmtAnd, SmtAssert, SmtEq2, SmtExpr, SmtImplies, SmtNot},
        patterns::{declare_datatype, DatastructurePattern, FunctionPattern},
    },
};

/**
 * This generates the following smt lib code:
 *
 * (declare-datatype Interval ((mk-interval (interval-start Int) (interval-end Int))))
 * (declare-datatype Group    ((mk-group    (group-src Int)
 *                                          (group-name String)
 *                                          (group-interval Interval))))
 * (declare-datatype Groups   ((groups-nil)
 *                             (groups-cons (groups-tip Group) (groups-tail Groups))))
 *
 * (define-fun interval-contains ((interval Interval) (x Int)) Bool
 *   (and (or (= (interval-start interval) x)
 *            (< (interval-start interval) x))
 *            (< x (interval-end interval))))
 *
 * (define-fun wire_groups () Groups
 *   (groups-cons (mk-group 0 "foo" (mk-interval 0 10))
 *                groups-nil)) ; List
 *
 * (define-fun import_ranges () Groups
 *   (groups-cons (mk-group 0 "foo" (mk-interval 0 10))
 *                groups-nil)) ; List
 *
 * (define-fun-rec groups-contains ((groups Groups) (needle Group)) Bool
 *   (match groups (
 *     ((groups-nil) false)
 *     ((groups-cons group groups)
 *       (let ((eq (= group needle)))
 *            (ite eq
 *                 true
 *                 (groups-contains groups needle)))))))
 *
 * (define-fun-rec f ((groups Groups) (source Int) (name String) (x Int) (count Int)) Int
 *   (match groups (
 *     ((groups-nil) count)
 *     ((groups-cons group groups)
 *       (let ((src-matches (= (group-src group) source))
 *             (name-matches (= (group-name group) name))
 *             (range-matches (interval-contains (group-interval group) x)))
 *            (ite (and src-matches name-matches range-matches)
 *                 (f groups source name x (+ 1 count))
 *                 (f groups source name x count)))))))
 *
 * (declare-const group Group)
 * (declare-const x Int)
 * (declare-const should-be-one Bool)
 * (declare-const is-one Bool)
 *
 * (assert (= is-one (= 1 (f wire_groups (group-src group) (group-name group) x 0))))
 * (assert (= should-be-one
 *            (and (groups-contains import_ranges group))
 *                 (interval-contains (group-interval group) x)))
 *
 * (assert (not (=> should-be-one is-one)))
 *
 * (check-sat)
 *
 */

// TODO:
//
// - a function that
//   - declares all types + functions
//   - generates the constants (wire-groups)
//   - declares constants (group, x, ...)
//   - asserts the constraintso

fn define_wire_group(varname: &str, groups: &[wire_group::WireGroup]) -> SmtExpr {
    let term: SmtExpr = groups.into();
    ("define-fun", varname, "()", wire_groups::Sort, term).into()
}

pub fn build_smt(comp: &Composition, mi_pkg_inst_idx: usize) -> Vec<SmtExpr> {
    let wire_groups = extract_wire_groups(comp, mi_pkg_inst_idx);
    let import_ranges = extract_import_ranges(comp, mi_pkg_inst_idx);

    let interval_pattern = interval::Pattern;
    let interval_spec = interval_pattern.datastructure_spec(&interval::DeclareInfo);
    let interval_dtdecl = declare_datatype(&interval_pattern, &interval_spec);

    let group_pattern = wire_group::Pattern;
    let group_spec = group_pattern.datastructure_spec(&wire_group::DeclareInfo);
    let group_dtdecl = declare_datatype(&group_pattern, &group_spec);

    let groups_pattern = wire_groups::Pattern;
    let groups_spec = groups_pattern.datastructure_spec(&wire_groups::DeclareInfo);
    let groups_dtdecl = declare_datatype(&groups_pattern, &groups_spec);

    let def_interal_contains = interval::define_contains_fun();
    let def_group_contains = wire_groups::define_contains_fun_rec();
    let def_f = define_wire_count_fun_rec();

    let def_wire_groups = define_wire_group("wire-groups", &wire_groups);
    let def_import_ranges = define_wire_group("import-ranges", &import_ranges);

    let decl_group = declare_const("group", wire_group::Sort);
    let decl_x = declare_const("x", smt::sorts::SmtInt);
    let decl_should_be_one = declare_const("should-be-one", smt::sorts::SmtBool);
    let decl_is_one = declare_const("is-one", smt::sorts::SmtBool);

    let first_assert: SmtExpr = SmtAssert(SmtEq2 {
        lhs: "is-one",
        rhs: SmtEq2 {
            lhs: 1,
            rhs: call_wire_count_fun(
                "wire-groups",
                wire_group::call_group_source("group"),
                wire_group::call_group_name("group"),
                "x",
                0,
            ),
        },
    })
    .into();

    let second_assert: SmtExpr = SmtAssert(SmtEq2 {
        lhs: "should-be-one",
        rhs: SmtAnd(vec![
            wire_groups::Contains.call(&["import-ranges".into(), "group".into()]),
            interval::Contains.call(&[wire_group::call_group_interval("group"), "x".into()]),
        ]),
    })
    .into();

    let third_assert: SmtExpr = SmtAssert(SmtNot(SmtImplies("should-be-one", "is-one"))).into();

    vec![
        interval_dtdecl,
        group_dtdecl,
        groups_dtdecl,
        def_interal_contains,
        def_group_contains,
        def_f,
        def_wire_groups,
        def_import_ranges,
        decl_group,
        decl_x,
        decl_should_be_one,
        decl_is_one,
        first_assert,
        second_assert,
        third_assert,
    ]
}

fn extract_integer_params(comp: &Composition) -> Vec<String> {
    comp.consts
        .iter()
        .filter_map(|param| {
            if param.1 == Type::Integer {
                Some(param.0.to_string())
            } else {
                None
            }
        })
        .collect()
}

// TODO handle add
fn extract_interval_from_index_expr<F: Fn(String) -> Option<Expression>>(
    params: &[String],
    forspecs: &[ForSpec],
    index: Expression,
    f: F,
) -> interval::Interval {
    match index {
        Expression::IntegerLiteral(num) => interval::Interval {
            start: num.into(),
            end: (num + 1).into(),
        },
        Expression::Identifier(ref ident) => {
            println!("hmmm {ident:?}");
            /*
             * two legal cases
             * - identifier can be a global param
             * - identifier can be from a forspec
             *
             * first check forspec, else look for param
             * if a param, just use it as is
             */
            let forspec = match forspecs
                .into_iter()
                .find(|forspec| forspec.as_name() == ident.ident_ref())
            {
                Some(forspec) => forspec,
                None => {
                    let expr = f(ident.ident()).unwrap();

                    return interval::Interval {
                        start: expr.clone().into(),
                        end: Expression::Add(
                            Box::new(Expression::IntegerLiteral(1)),
                            Box::new(expr),
                        )
                        .into(),
                    };
                }
            };

            let start: Option<SmtExpr> = match forspec.start() {
                Expression::Identifier(start_ident) => {
                    let ident_name = start_ident.ident();
                    if params.contains(&ident_name) {
                        Some(ident_name.into())
                    } else {
                        None
                    }
                }
                Expression::IntegerLiteral(num) => Some((*num).into()),
                _ => unreachable!(),
            };

            let end: Option<SmtExpr> = match forspec.end() {
                Expression::Identifier(start_ident) => {
                    let ident_name = start_ident.ident();
                    if params.contains(&ident_name) {
                        Some(ident_name.into())
                    } else {
                        None
                    }
                }
                Expression::IntegerLiteral(num) => Some((*num).into()),
                _ => unreachable!(),
            };

            let start = start.map(|start| match forspec.start_comp() {
                crate::parser::package::ForComp::Lt => ("+", start, 1).into(),
                crate::parser::package::ForComp::Lte => start,
            });

            let end = end.map(|end| match forspec.start_comp() {
                crate::parser::package::ForComp::Lt => end,
                crate::parser::package::ForComp::Lte => ("+", 1, end).into(),
            });

            interval::Interval {
                start: start.unwrap(),
                end: end.unwrap(),
            }
        }
        _ => unreachable!(),
    }
}

fn extract_interval_from_oraclesig(
    params: &[String],
    oracle_sig: &OracleSig,
) -> interval::Interval {
    let mi_indices = oracle_sig.multi_inst_idx.clone().unwrap();
    extract_interval_from_index_expr(
        params,
        &mi_indices.forspecs,
        mi_indices.indices[0].clone(),
        |ident_name| {
            // check params. if we find it, just return interval (p, p+1).
            assert!(params.contains(&ident_name));
            Some(Expression::Identifier(Identifier::Local(ident_name)))
        },
    )
}

pub fn extract_wire_groups(
    comp: &Composition,
    mi_pkg_inst_idx: usize,
) -> Vec<wire_group::WireGroup> {
    let params = extract_integer_params(comp);
    let mut out = vec![];

    for Edge(source, _, oracle_sig) in comp
        .edges
        .iter()
        .filter(|edge| edge.1 == mi_pkg_inst_idx && edge.2.multi_inst_idx.is_some())
        .cloned()
    {
        let interval = extract_interval_from_oraclesig(&params, &oracle_sig);
        out.push(wire_group::WireGroup {
            source,
            name: oracle_sig.name,
            interval,
        })
    }

    for MultiInstanceEdge {
        source_pkgidx,
        oracle_sig,
        ..
    } in comp
        .multi_inst_edges
        .iter()
        .filter(|edge| {
            edge.source_pkgidx == mi_pkg_inst_idx && edge.oracle_sig.multi_inst_idx.is_some()
        })
        .cloned()
    {
        let interval = extract_interval_from_oraclesig(&params, &oracle_sig);
        out.push(wire_group::WireGroup {
            source: source_pkgidx,
            name: oracle_sig.name,
            interval,
        })
    }

    out
}

pub fn extract_import_ranges(
    comp: &Composition,
    mi_pkg_inst_idx: usize,
) -> Vec<wire_group::WireGroup> {
    let params = extract_integer_params(comp);
    let mut out = vec![];

    for Edge(src, dst, _) in &comp.edges {
        if *dst != mi_pkg_inst_idx {
            continue;
        }

        let pkg_inst = &comp.pkgs[*src];
        for import in &pkg_inst.pkg.imports {
            let oracle_sig: &OracleSig = &import.0;
            let indices = oracle_sig.multi_inst_idx.clone().unwrap();
            let MultiInstanceIndices {
                mut indices,
                forspecs,
            } = indices;
            let index = indices.remove(0);
            let interval =
                extract_interval_from_index_expr(&params, &forspecs, index, |ident_name| {
                    pkg_inst
                        .params
                        .iter()
                        .find(|(name, expr)| name == &ident_name)
                        .map(|(name, expr)| {
                            match expr {
                                Expression::IntegerLiteral(num) => expr,
                                Expression::Identifier(ident) => {
                                    // this is the identifier that is assigned to the param in the game.
                                    // it could either be a proof-level constant, a literal or a variable of the for loop in which
                                    // the package instance is instantiated.

                                    match ident {
                                        Identifier::Scalar(_) => todo!(),
                                        Identifier::State(_) => todo!(),
                                        Identifier::Parameter(_) => todo!(),
                                        Identifier::ComposeLoopVar(_) => todo!(),
                                        Identifier::Local(_) => todo!(),
                                        Identifier::GameInstanceConst(_) => todo!(),
                                    }
                                }
                                _ => unreachable!(),
                            }
                        })
                        .cloned()
                });

            out.push(wire_group::WireGroup {
                interval,
                source: *src,
                name: import.0.name.clone(),
            });
        }
    }

    for mi_edge in &comp.multi_inst_edges {
        if mi_edge.dest_pkgidx != mi_pkg_inst_idx {
            continue;
        }

        let source = mi_edge.source_pkgidx;
        let pkg_inst = &comp.pkgs[source];
        for import in &pkg_inst.pkg.imports {
            let oracle_sig: &OracleSig = &import.0;
            let indices = oracle_sig.multi_inst_idx.clone().unwrap();
            let MultiInstanceIndices {
                mut indices,
                forspecs,
            } = indices;
            let index = indices.remove(0);

            let interval =
                extract_interval_from_index_expr(&params, &forspecs, index, |ident_name| {
                    pkg_inst
                        .params
                        .iter()
                        .find(|(name, expr)| name == &ident_name)
                        .map(|(name, expr)| {
                            match expr {
                                Expression::IntegerLiteral(num) => expr,
                                Expression::Identifier(ident) => {
                                    // this is the identifier that is assigned to the param in the game.
                                    // it could either be a proof-level constant, a literal or a variable of the for loop in which
                                    // the package instance is instantiated.

                                    todo!()
                                    // match ident {
                                    //     Identifier::Scalar(name) => {
                                    //         if let Some(forspec) = pkg_inst
                                    //             .multi_instance_indices
                                    //             .and_then(|indices| {
                                    //                 indices.forspecs.iter().find(|forspec| {
                                    //                     forspec.ident().ident_ref() == name
                                    //                 })
                                    //             })
                                    //         {}
                                    //     }
                                    //     _ => unreachable!(),
                                    // }
                                }
                                _ => unreachable!(),
                            }
                        })
                        .cloned()
                });
            let name = import.0.name.clone();

            out.push(wire_group::WireGroup {
                interval,
                source,
                name,
            });
        }
    }

    out
}

// f
struct WireCountFn;

impl FunctionPattern for WireCountFn {
    type ReturnSort = smt::sorts::SmtInt;

    fn function_name(&self) -> String {
        "count-wires".to_string()
    }

    fn function_args(&self) -> Vec<(String, crate::writers::smt::exprs::SmtExpr)> {
        vec![
            ("wire-groups".to_string(), wire_groups::Sort.into()),
            ("source".to_string(), smt::sorts::SmtInt.into()),
            ("name".to_string(), smt::sorts::SmtString.into()),
            ("x".to_string(), smt::sorts::SmtInt.into()),
            ("count".to_string(), smt::sorts::SmtInt.into()),
        ]
    }

    fn function_return_sort(&self) -> Self::ReturnSort {
        smt::sorts::SmtInt
    }
}

pub fn define_wire_count_fun_rec() -> smt::exprs::SmtExpr {
    let wire_groups_spec = &wire_groups::Pattern.datastructure_spec(&wire_groups::DeclareInfo);
    let wire_group_spec = &wire_group::Pattern.datastructure_spec(&wire_group::DeclareInfo);

    let groups = "wire-groups";
    let source = "source";
    let name = "name";
    let x = "x";
    let count = "count";

    let body = wire_groups::Pattern.match_expr(groups, wire_groups_spec, |con| match con {
        wire_groups::Constructor::Nil => count.into(),
        wire_groups::Constructor::Cons => {
            let tip = wire_groups::Pattern.matchfield_name(&wire_groups::Selector::Tip);
            let tail = wire_groups::Pattern.matchfield_name(&wire_groups::Selector::Tail);

            let group_src = wire_group::Pattern
                .access(wire_group_spec, &wire_group::Selector::Source, &tip)
                .unwrap();
            let group_name = wire_group::Pattern
                .access(wire_group_spec, &wire_group::Selector::Name, &tip)
                .unwrap();
            let group_interval = wire_group::Pattern
                .access(wire_group_spec, &wire_group::Selector::Interval, &tip)
                .unwrap();
            let src_matches = smt::exprs::SmtEq2 {
                lhs: group_src,
                rhs: source,
            };
            let name_matches = smt::exprs::SmtEq2 {
                lhs: group_name,
                rhs: name,
            };
            let interval_matches = interval::Contains.call(&[group_interval.into(), x.into()]);

            smt::exprs::SmtIte {
                cond: smt::exprs::SmtAnd(vec![
                    src_matches.into(),
                    name_matches.into(),
                    interval_matches.into(),
                ]),
                then: WireCountFn.call(&[
                    tail.clone().into(),
                    source.into(),
                    name.into(),
                    x.into(),
                    ("+", 1, count).into(),
                ]),
                els: WireCountFn.call(&[
                    tail.into(),
                    source.into(),
                    name.into(),
                    x.into(),
                    count.into(),
                ]),
            }
            .into()
        }
    });

    WireCountFn.define_fun_rec(body).into()
}

pub fn call_wire_count_fun<Groups, Source, Name, X, Count>(
    groups: Groups,
    source: Source,
    name: Name,
    x: X,
    count: Count,
) -> SmtExpr
where
    Groups: Into<SmtExpr>,
    Source: Into<SmtExpr>,
    Name: Into<SmtExpr>,
    X: Into<SmtExpr>,
    Count: Into<SmtExpr>,
{
    WireCountFn.call(&[
        groups.into(),
        source.into(),
        name.into(),
        x.into(),
        count.into(),
    ])
}

mod interval {
    use crate::writers::smt;
    use crate::writers::smt::exprs::{SmtAnd, SmtExpr, SmtLt, SmtLte};
    use crate::writers::smt::patterns::{self, DatastructurePattern, FunctionPattern};

    #[derive(Clone)]
    pub struct Interval {
        pub start: SmtExpr,
        pub end: SmtExpr,
    }

    impl From<Interval> for SmtExpr {
        fn from(value: Interval) -> Self {
            let spec = &Pattern.datastructure_spec(&DeclareInfo);
            Pattern
                .call_constructor(spec, &Constructor, |sel| match sel {
                    Selector::Start => Some(value.start.clone().into()),
                    Selector::End => Some(value.end.clone().into()),
                })
                .unwrap()
        }
    }

    pub struct Pattern;

    #[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq)]
    pub struct Sort;
    #[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq)]
    pub struct Constructor;
    #[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq)]
    pub enum Selector {
        Start,
        End,
    }

    pub struct DeclareInfo;

    impl From<Sort> for SmtExpr {
        fn from(_: Sort) -> Self {
            SmtExpr::Atom(Pattern::CAMEL_CASE.to_string())
        }
    }
    impl smt::sorts::SmtSort for Sort {}
    impl smt::sorts::SmtPlainSort for Sort {
        fn sort_name(&self) -> String {
            "Interval".to_string()
        }
    }

    impl<'a> patterns::DatastructurePattern<'a> for Pattern {
        type Sort = Sort;

        type Constructor = Constructor;

        type Selector = Selector;

        type DeclareInfo = DeclareInfo;

        const CAMEL_CASE: &'static str = "Interval";

        const KEBAB_CASE: &'static str = "interval";

        fn sort(&self) -> Self::Sort {
            Sort
        }

        fn constructor_name(&self, _: &Self::Constructor) -> String {
            "mk-interval".to_string()
        }

        fn selector_name(&self, sel: &Self::Selector) -> String {
            match sel {
                Selector::Start => "interval-start",
                Selector::End => "interval-end",
            }
            .to_string()
        }

        fn selector_sort(&self, _: &Self::Selector) -> crate::writers::smt::exprs::SmtExpr {
            smt::sorts::SmtInt.into()
        }

        fn datastructure_spec(
            &self,
            _: &'a Self::DeclareInfo,
        ) -> patterns::DatastructureSpec<'a, Self> {
            patterns::DatastructureSpec(vec![(Constructor, vec![Selector::Start, Selector::End])])
        }

        fn matchfield_name(&self, sel: &Self::Selector) -> String {
            match sel {
                Selector::Start => "match_start",
                Selector::End => "match_end",
            }
            .into()
        }
    }

    pub(crate) struct Contains;

    impl patterns::FunctionPattern for Contains {
        type ReturnSort = smt::sorts::SmtBool;

        fn function_name(&self) -> String {
            "interval-contains".to_string()
        }

        fn function_args(&self) -> Vec<(String, SmtExpr)> {
            vec![
                ("interval".to_string(), Sort.into()),
                ("x".to_string(), smt::sorts::SmtInt.into()),
            ]
        }

        fn function_return_sort(&self) -> Self::ReturnSort {
            smt::sorts::SmtBool
        }
    }

    pub fn define_contains_fun() -> SmtExpr {
        let spec = Pattern.datastructure_spec(&DeclareInfo);
        let interval = "interval";
        let x = "x";

        let body = SmtAnd(vec![
            SmtLte(
                Pattern.access(&spec, &Selector::Start, interval).unwrap(),
                x,
            )
            .into(),
            SmtLt(x, Pattern.access(&spec, &Selector::End, interval).unwrap()).into(),
        ]);

        Contains.define_fun(body)
    }
}

mod wire_group {
    use crate::writers::smt;
    use crate::writers::smt::exprs::SmtExpr;
    use crate::writers::smt::patterns::{DatastructurePattern, DatastructureSpec};

    #[derive(Clone)]
    pub struct WireGroup {
        pub source: usize,
        pub name: String,
        pub interval: super::interval::Interval,
    }

    impl From<WireGroup> for SmtExpr {
        fn from(value: WireGroup) -> Self {
            let spec = Pattern.datastructure_spec(&DeclareInfo);
            Pattern
                .call_constructor(&spec, &Constructor, |sel| match sel {
                    Selector::Source => Some(value.source.into()),
                    Selector::Name => Some(value.name.clone().into()),
                    Selector::Interval => Some(value.interval.clone().into()),
                })
                .unwrap()
        }
    }

    pub struct Pattern;

    #[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq)]
    pub struct Sort;
    #[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq)]
    pub struct Constructor;
    #[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq)]
    pub enum Selector {
        Source,
        Name,
        Interval,
    }

    pub struct DeclareInfo;

    impl From<Sort> for SmtExpr {
        fn from(_: Sort) -> Self {
            SmtExpr::Atom(Pattern::CAMEL_CASE.to_string())
        }
    }
    impl smt::sorts::SmtSort for Sort {}

    impl smt::sorts::SmtPlainSort for Sort {
        fn sort_name(&self) -> String {
            "WireGroup".to_string()
        }
    }

    impl<'a> DatastructurePattern<'a> for Pattern {
        type Sort = Sort;

        type Constructor = Constructor;

        type Selector = Selector;

        type DeclareInfo = DeclareInfo;

        const CAMEL_CASE: &'static str = "WireGroup";

        const KEBAB_CASE: &'static str = "wire-group";

        fn sort(&self) -> Self::Sort {
            Sort
        }

        fn constructor_name(&self, _: &Self::Constructor) -> String {
            "mk-wire-group".to_string()
        }

        fn selector_name(&self, sel: &Self::Selector) -> String {
            match sel {
                Selector::Source => "wire-group-src",
                Selector::Name => "wire-group-name",
                Selector::Interval => "wire-group-interval",
            }
            .to_string()
        }

        fn selector_sort(&self, sel: &Self::Selector) -> SmtExpr {
            match sel {
                Selector::Source => SmtExpr::Atom("Int".to_string()),
                Selector::Name => SmtExpr::Atom("String".to_string()),
                Selector::Interval => super::interval::Sort.into(),
            }
        }

        fn datastructure_spec(&self, _: &'a Self::DeclareInfo) -> DatastructureSpec<'a, Self> {
            DatastructureSpec(vec![(
                Constructor,
                vec![Selector::Source, Selector::Name, Selector::Interval],
            )])
        }

        fn matchfield_name(&self, sel: &Self::Selector) -> String {
            match sel {
                Selector::Source => "match-wire-group-src",
                Selector::Name => "match-wire-group-name",
                Selector::Interval => "match-wire-group-interval",
            }
            .to_string()
        }
    }

    pub fn call_group_name<Group>(group: Group) -> SmtExpr
    where
        Group: Into<SmtExpr>,
    {
        let spec = &Pattern.datastructure_spec(&DeclareInfo);
        Pattern.access(spec, &Selector::Name, group).unwrap()
    }

    pub fn call_group_source<Group>(group: Group) -> SmtExpr
    where
        Group: Into<SmtExpr>,
    {
        let spec = &Pattern.datastructure_spec(&DeclareInfo);
        Pattern.access(spec, &Selector::Source, group).unwrap()
    }

    pub fn call_group_interval<Group>(group: Group) -> SmtExpr
    where
        Group: Into<SmtExpr>,
    {
        let spec = &Pattern.datastructure_spec(&DeclareInfo);
        Pattern.access(spec, &Selector::Interval, group).unwrap()
    }
}

mod wire_groups {
    use crate::writers::smt::exprs::{SmtEq2, SmtExpr, SmtIte};
    use crate::writers::smt::patterns::{DatastructurePattern, DatastructureSpec, FunctionPattern};
    use crate::writers::smt::{self, patterns};

    pub struct Pattern;

    impl From<&[super::wire_group::WireGroup]> for SmtExpr {
        fn from(value: &[super::wire_group::WireGroup]) -> Self {
            let spec = &Pattern.datastructure_spec(&DeclareInfo);
            let nil = Pattern
                .call_constructor(spec, &Constructor::Nil, |_| None)
                .unwrap();

            value.iter().fold(nil, |acc, wire_group| {
                Pattern
                    .call_constructor(spec, &Constructor::Cons, |sel| match sel {
                        Selector::Tip => Some(wire_group.clone().into()),
                        Selector::Tail => Some(acc.clone()),
                    })
                    .unwrap()
            })
        }
    }

    #[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq)]
    pub struct Sort;
    #[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq)]
    pub enum Constructor {
        Nil,
        Cons,
    }

    #[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq)]
    pub enum Selector {
        Tip,
        Tail,
    }

    pub struct DeclareInfo;

    impl From<Sort> for SmtExpr {
        fn from(_: Sort) -> Self {
            SmtExpr::Atom(Pattern::CAMEL_CASE.to_string())
        }
    }
    impl smt::sorts::SmtSort for Sort {}
    impl smt::sorts::SmtPlainSort for Sort {
        fn sort_name(&self) -> String {
            "WireGroups".to_string()
        }
    }

    impl<'a> DatastructurePattern<'a> for Pattern {
        type Sort = Sort;

        type Constructor = Constructor;

        type Selector = Selector;

        type DeclareInfo = DeclareInfo;

        const CAMEL_CASE: &'static str = "WireGroups";

        const KEBAB_CASE: &'static str = "wire-groups";

        fn sort(&self) -> Self::Sort {
            Sort
        }

        fn constructor_name(&self, cons: &Self::Constructor) -> String {
            match cons {
                Constructor::Nil => "wire-groups-nil",
                Constructor::Cons => "wire-groups-cons",
            }
            .to_string()
        }

        fn selector_name(&self, sel: &Self::Selector) -> String {
            match sel {
                Selector::Tip => "wire-groups-tip",
                Selector::Tail => "wire-groups-tail",
            }
            .to_string()
        }

        fn selector_sort(&self, sel: &Self::Selector) -> SmtExpr {
            match sel {
                Selector::Tip => super::wire_group::Sort.into(),
                Selector::Tail => Sort.into(),
            }
        }

        fn datastructure_spec(&self, info: &'a Self::DeclareInfo) -> DatastructureSpec<'a, Self> {
            DatastructureSpec(vec![
                (Constructor::Nil, vec![]),
                (Constructor::Cons, vec![Selector::Tip, Selector::Tail]),
            ])
        }

        fn matchfield_name(&self, sel: &Self::Selector) -> String {
            match sel {
                Selector::Tip => "match-wire-groups-tip",
                Selector::Tail => "match-wire-groups-tail",
            }
            .to_string()
        }
    }

    pub struct Contains;

    impl patterns::FunctionPattern for Contains {
        type ReturnSort = smt::sorts::SmtBool;

        fn function_name(&self) -> String {
            "wire-groups-contains".to_string()
        }

        fn function_args(&self) -> Vec<(String, SmtExpr)> {
            vec![
                ("groups".to_string(), Sort.into()),
                ("needle".to_string(), super::wire_group::Sort.into()),
            ]
        }

        fn function_return_sort(&self) -> Self::ReturnSort {
            smt::sorts::SmtBool
        }
    }

    pub fn define_contains_fun_rec() -> SmtExpr {
        let spec = Pattern.datastructure_spec(&DeclareInfo);
        let groups = "groups";
        let needle = "needle";

        let body = Pattern.match_expr(groups, &spec, |con| match con {
            Constructor::Nil => false.into(),
            Constructor::Cons => {
                let tip = Pattern.matchfield_name(&Selector::Tip);
                let tail = Pattern.matchfield_name(&Selector::Tail);

                SmtIte {
                    cond: SmtEq2 {
                        lhs: needle,
                        rhs: tip,
                    },
                    then: true,
                    els: Contains.call(&[tail.into(), needle.into()]),
                }
                .into()
            }
        });

        Contains.define_fun_rec(body)
    }
}
