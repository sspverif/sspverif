use crate::writers::smt::{
    self,
    patterns::{DatastructurePattern, FunctionPattern},
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
 * (define-fun-rec groups-contains ((groups Groups) (needle Group)) Bool
 *   (match groups (
 *     ((groups-nil) false)
 *     ((groups-cons group groups)
 *       (let ((eq (= group needle)))
 *            (ite eq
 *                 true
 *                 (groups-contains groups needle)))))))
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
//   - asserts the constraints

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

    WireCountFn.define_fun(body).into()
}

mod interval {
    use crate::writers::smt;
    use crate::writers::smt::exprs::{SmtAnd, SmtExpr, SmtLt, SmtLte};
    use crate::writers::smt::patterns::{self, DatastructurePattern, FunctionPattern};

    #[derive(Clone, Copy)]
    pub struct Interval {
        pub start: usize,
        pub end: usize,
    }

    impl From<Interval> for SmtExpr {
        fn from(value: Interval) -> Self {
            let spec = &Pattern.datastructure_spec(&DeclareInfo);
            Pattern
                .call_constructor(spec, &Constructor, |sel| match sel {
                    Selector::Start => Some(value.start.into()),
                    Selector::End => Some(value.end.into()),
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
                    Selector::Interval => Some(value.interval.into()),
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

    struct Contains;

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
