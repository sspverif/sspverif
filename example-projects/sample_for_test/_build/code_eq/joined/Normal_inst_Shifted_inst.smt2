(set-logic ALL)
(declare-sort Bits_n 0)
(declare-datatypes ((Maybe 1)
)
 ((par (T)
 ((mk-some (maybe-get T)
)
 (mk-none)
)
)
)
)
(declare-datatypes ((Tuple1 1)
)
 ((par (T1)
 ((mk-tuple1 (el1 T1)
)
)
)
)
)
(declare-datatypes ((Tuple2 2)
)
 ((par (T1 T2)
 ((mk-tuple2 (el1 T1)
 (el2 T2)
)
)
)
)
)
(declare-datatypes ((Tuple3 3)
)
 ((par (T1 T2 T3)
 ((mk-tuple3 (el1 T1)
 (el2 T2)
 (el3 T3)
)
)
)
)
)
(declare-datatypes ((Tuple4 4)
)
 ((par (T1 T2 T3 T4)
 ((mk-tuple4 (el1 T1)
 (el2 T2)
 (el3 T3)
 (el4 T4)
)
)
)
)
)
(declare-datatypes ((Tuple5 5)
)
 ((par (T1 T2 T3 T4 T5)
 ((mk-tuple5 (el1 T1)
 (el2 T2)
 (el3 T3)
 (el4 T4)
 (el5 T5)
)
)
)
)
)
(declare-datatypes ((Tuple6 6)
)
 ((par (T1 T2 T3 T4 T5 T6)
 ((mk-tuple6 (el1 T1)
 (el2 T2)
 (el3 T3)
 (el4 T4)
 (el5 T5)
 (el6 T6)
)
)
)
)
)
(declare-datatypes ((Tuple7 7)
)
 ((par (T1 T2 T3 T4 T5 T6 T7)
 ((mk-tuple7 (el1 T1)
 (el2 T2)
 (el3 T3)
 (el4 T4)
 (el5 T5)
 (el6 T6)
 (el7 T7)
)
)
)
)
)
(declare-datatypes ((Tuple8 8)
)
 ((par (T1 T2 T3 T4 T5 T6 T7 T8)
 ((mk-tuple8 (el1 T1)
 (el2 T2)
 (el3 T3)
 (el4 T4)
 (el5 T5)
 (el6 T6)
 (el7 T7)
 (el8 T8)
)
)
)
)
)
(declare-datatypes ((Tuple9 9)
)
 ((par (T1 T2 T3 T4 T5 T6 T7 T8 T9)
 ((mk-tuple9 (el1 T1)
 (el2 T2)
 (el3 T3)
 (el4 T4)
 (el5 T5)
 (el6 T6)
 (el7 T7)
 (el8 T8)
 (el9 T9)
)
)
)
)
)
(declare-datatypes ((Tuple10 10)
)
 ((par (T1 T2 T3 T4 T5 T6 T7 T8 T9 T10)
 ((mk-tuple10 (el1 T1)
 (el2 T2)
 (el3 T3)
 (el4 T4)
 (el5 T5)
 (el6 T6)
 (el7 T7)
 (el8 T8)
 (el9 T9)
 (el10 T10)
)
)
)
)
)
(declare-datatypes ((Tuple11 11)
)
 ((par (T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11)
 ((mk-tuple11 (el1 T1)
 (el2 T2)
 (el3 T3)
 (el4 T4)
 (el5 T5)
 (el6 T6)
 (el7 T7)
 (el8 T8)
 (el9 T9)
 (el10 T10)
 (el11 T11)
)
)
)
)
)
(declare-datatypes ((Tuple12 12)
)
 ((par (T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12)
 ((mk-tuple12 (el1 T1)
 (el2 T2)
 (el3 T3)
 (el4 T4)
 (el5 T5)
 (el6 T6)
 (el7 T7)
 (el8 T8)
 (el9 T9)
 (el10 T10)
 (el11 T11)
 (el12 T12)
)
)
)
)
)
(declare-datatypes ((Tuple13 13)
)
 ((par (T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13)
 ((mk-tuple13 (el1 T1)
 (el2 T2)
 (el3 T3)
 (el4 T4)
 (el5 T5)
 (el6 T6)
 (el7 T7)
 (el8 T8)
 (el9 T9)
 (el10 T10)
 (el11 T11)
 (el12 T12)
 (el13 T13)
)
)
)
)
)
(declare-datatypes ((Tuple14 14)
)
 ((par (T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14)
 ((mk-tuple14 (el1 T1)
 (el2 T2)
 (el3 T3)
 (el4 T4)
 (el5 T5)
 (el6 T6)
 (el7 T7)
 (el8 T8)
 (el9 T9)
 (el10 T10)
 (el11 T11)
 (el12 T12)
 (el13 T13)
 (el14 T14)
)
)
)
)
)
(declare-datatypes ((Tuple15 15)
)
 ((par (T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15)
 ((mk-tuple15 (el1 T1)
 (el2 T2)
 (el3 T3)
 (el4 T4)
 (el5 T5)
 (el6 T6)
 (el7 T7)
 (el8 T8)
 (el9 T9)
 (el10 T10)
 (el11 T11)
 (el12 T12)
 (el13 T13)
 (el14 T14)
 (el15 T15)
)
)
)
)
)
(declare-datatypes ((Tuple16 16)
)
 ((par (T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15 T16)
 ((mk-tuple16 (el1 T1)
 (el2 T2)
 (el3 T3)
 (el4 T4)
 (el5 T5)
 (el6 T6)
 (el7 T7)
 (el8 T8)
 (el9 T9)
 (el10 T10)
 (el11 T11)
 (el12 T12)
 (el13 T13)
 (el14 T14)
 (el15 T15)
 (el16 T16)
)
)
)
)
)
(declare-datatypes ((Tuple17 17)
)
 ((par (T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15 T16 T17)
 ((mk-tuple17 (el1 T1)
 (el2 T2)
 (el3 T3)
 (el4 T4)
 (el5 T5)
 (el6 T6)
 (el7 T7)
 (el8 T8)
 (el9 T9)
 (el10 T10)
 (el11 T11)
 (el12 T12)
 (el13 T13)
 (el14 T14)
 (el15 T15)
 (el16 T16)
 (el17 T17)
)
)
)
)
)
(declare-datatypes ((Tuple18 18)
)
 ((par (T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15 T16 T17 T18)
 ((mk-tuple18 (el1 T1)
 (el2 T2)
 (el3 T3)
 (el4 T4)
 (el5 T5)
 (el6 T6)
 (el7 T7)
 (el8 T8)
 (el9 T9)
 (el10 T10)
 (el11 T11)
 (el12 T12)
 (el13 T13)
 (el14 T14)
 (el15 T15)
 (el16 T16)
 (el17 T17)
 (el18 T18)
)
)
)
)
)
(declare-datatypes ((Tuple19 19)
)
 ((par (T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15 T16 T17 T18 T19)
 ((mk-tuple19 (el1 T1)
 (el2 T2)
 (el3 T3)
 (el4 T4)
 (el5 T5)
 (el6 T6)
 (el7 T7)
 (el8 T8)
 (el9 T9)
 (el10 T10)
 (el11 T11)
 (el12 T12)
 (el13 T13)
 (el14 T14)
 (el15 T15)
 (el16 T16)
 (el17 T17)
 (el18 T18)
 (el19 T19)
)
)
)
)
)
(declare-datatypes ((Tuple20 20)
)
 ((par (T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15 T16 T17 T18 T19 T20)
 ((mk-tuple20 (el1 T1)
 (el2 T2)
 (el3 T3)
 (el4 T4)
 (el5 T5)
 (el6 T6)
 (el7 T7)
 (el8 T8)
 (el9 T9)
 (el10 T10)
 (el11 T11)
 (el12 T12)
 (el13 T13)
 (el14 T14)
 (el15 T15)
 (el16 T16)
 (el17 T17)
 (el18 T18)
 (el19 T19)
 (el20 T20)
)
)
)
)
)
(declare-datatypes ((Tuple21 21)
)
 ((par (T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15 T16 T17 T18 T19 T20 T21)
 ((mk-tuple21 (el1 T1)
 (el2 T2)
 (el3 T3)
 (el4 T4)
 (el5 T5)
 (el6 T6)
 (el7 T7)
 (el8 T8)
 (el9 T9)
 (el10 T10)
 (el11 T11)
 (el12 T12)
 (el13 T13)
 (el14 T14)
 (el15 T15)
 (el16 T16)
 (el17 T17)
 (el18 T18)
 (el19 T19)
 (el20 T20)
 (el21 T21)
)
)
)
)
)
(declare-datatypes ((Tuple22 22)
)
 ((par (T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15 T16 T17 T18 T19 T20 T21 T22)
 ((mk-tuple22 (el1 T1)
 (el2 T2)
 (el3 T3)
 (el4 T4)
 (el5 T5)
 (el6 T6)
 (el7 T7)
 (el8 T8)
 (el9 T9)
 (el10 T10)
 (el11 T11)
 (el12 T12)
 (el13 T13)
 (el14 T14)
 (el15 T15)
 (el16 T16)
 (el17 T17)
 (el18 T18)
 (el19 T19)
 (el20 T20)
 (el21 T21)
 (el22 T22)
)
)
)
)
)
(declare-datatypes ((Tuple23 23)
)
 ((par (T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15 T16 T17 T18 T19 T20 T21 T22 T23)
 ((mk-tuple23 (el1 T1)
 (el2 T2)
 (el3 T3)
 (el4 T4)
 (el5 T5)
 (el6 T6)
 (el7 T7)
 (el8 T8)
 (el9 T9)
 (el10 T10)
 (el11 T11)
 (el12 T12)
 (el13 T13)
 (el14 T14)
 (el15 T15)
 (el16 T16)
 (el17 T17)
 (el18 T18)
 (el19 T19)
 (el20 T20)
 (el21 T21)
 (el22 T22)
 (el23 T23)
)
)
)
)
)
(declare-datatypes ((Tuple24 24)
)
 ((par (T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15 T16 T17 T18 T19 T20 T21 T22 T23 T24)
 ((mk-tuple24 (el1 T1)
 (el2 T2)
 (el3 T3)
 (el4 T4)
 (el5 T5)
 (el6 T6)
 (el7 T7)
 (el8 T8)
 (el9 T9)
 (el10 T10)
 (el11 T11)
 (el12 T12)
 (el13 T13)
 (el14 T14)
 (el15 T15)
 (el16 T16)
 (el17 T17)
 (el18 T18)
 (el19 T19)
 (el20 T20)
 (el21 T21)
 (el22 T22)
 (el23 T23)
 (el24 T24)
)
)
)
)
)
(declare-datatypes ((Tuple25 25)
)
 ((par (T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15 T16 T17 T18 T19 T20 T21 T22 T23 T24 T25)
 ((mk-tuple25 (el1 T1)
 (el2 T2)
 (el3 T3)
 (el4 T4)
 (el5 T5)
 (el6 T6)
 (el7 T7)
 (el8 T8)
 (el9 T9)
 (el10 T10)
 (el11 T11)
 (el12 T12)
 (el13 T13)
 (el14 T14)
 (el15 T15)
 (el16 T16)
 (el17 T17)
 (el18 T18)
 (el19 T19)
 (el20 T20)
 (el21 T21)
 (el22 T22)
 (el23 T23)
 (el24 T24)
 (el25 T25)
)
)
)
)
)
(declare-datatypes ((Tuple26 26)
)
 ((par (T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15 T16 T17 T18 T19 T20 T21 T22 T23 T24 T25 T26)
 ((mk-tuple26 (el1 T1)
 (el2 T2)
 (el3 T3)
 (el4 T4)
 (el5 T5)
 (el6 T6)
 (el7 T7)
 (el8 T8)
 (el9 T9)
 (el10 T10)
 (el11 T11)
 (el12 T12)
 (el13 T13)
 (el14 T14)
 (el15 T15)
 (el16 T16)
 (el17 T17)
 (el18 T18)
 (el19 T19)
 (el20 T20)
 (el21 T21)
 (el22 T22)
 (el23 T23)
 (el24 T24)
 (el25 T25)
 (el26 T26)
)
)
)
)
)
(declare-datatypes ((Tuple27 27)
)
 ((par (T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15 T16 T17 T18 T19 T20 T21 T22 T23 T24 T25 T26 T27)
 ((mk-tuple27 (el1 T1)
 (el2 T2)
 (el3 T3)
 (el4 T4)
 (el5 T5)
 (el6 T6)
 (el7 T7)
 (el8 T8)
 (el9 T9)
 (el10 T10)
 (el11 T11)
 (el12 T12)
 (el13 T13)
 (el14 T14)
 (el15 T15)
 (el16 T16)
 (el17 T17)
 (el18 T18)
 (el19 T19)
 (el20 T20)
 (el21 T21)
 (el22 T22)
 (el23 T23)
 (el24 T24)
 (el25 T25)
 (el26 T26)
 (el27 T27)
)
)
)
)
)
(declare-datatypes ((Tuple28 28)
)
 ((par (T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15 T16 T17 T18 T19 T20 T21 T22 T23 T24 T25 T26 T27 T28)
 ((mk-tuple28 (el1 T1)
 (el2 T2)
 (el3 T3)
 (el4 T4)
 (el5 T5)
 (el6 T6)
 (el7 T7)
 (el8 T8)
 (el9 T9)
 (el10 T10)
 (el11 T11)
 (el12 T12)
 (el13 T13)
 (el14 T14)
 (el15 T15)
 (el16 T16)
 (el17 T17)
 (el18 T18)
 (el19 T19)
 (el20 T20)
 (el21 T21)
 (el22 T22)
 (el23 T23)
 (el24 T24)
 (el25 T25)
 (el26 T26)
 (el27 T27)
 (el28 T28)
)
)
)
)
)
(declare-datatypes ((Tuple29 29)
)
 ((par (T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15 T16 T17 T18 T19 T20 T21 T22 T23 T24 T25 T26 T27 T28 T29)
 ((mk-tuple29 (el1 T1)
 (el2 T2)
 (el3 T3)
 (el4 T4)
 (el5 T5)
 (el6 T6)
 (el7 T7)
 (el8 T8)
 (el9 T9)
 (el10 T10)
 (el11 T11)
 (el12 T12)
 (el13 T13)
 (el14 T14)
 (el15 T15)
 (el16 T16)
 (el17 T17)
 (el18 T18)
 (el19 T19)
 (el20 T20)
 (el21 T21)
 (el22 T22)
 (el23 T23)
 (el24 T24)
 (el25 T25)
 (el26 T26)
 (el27 T27)
 (el28 T28)
 (el29 T29)
)
)
)
)
)
(declare-datatypes ((Tuple30 30)
)
 ((par (T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15 T16 T17 T18 T19 T20 T21 T22 T23 T24 T25 T26 T27 T28 T29 T30)
 ((mk-tuple30 (el1 T1)
 (el2 T2)
 (el3 T3)
 (el4 T4)
 (el5 T5)
 (el6 T6)
 (el7 T7)
 (el8 T8)
 (el9 T9)
 (el10 T10)
 (el11 T11)
 (el12 T12)
 (el13 T13)
 (el14 T14)
 (el15 T15)
 (el16 T16)
 (el17 T17)
 (el18 T18)
 (el19 T19)
 (el20 T20)
 (el21 T21)
 (el22 T22)
 (el23 T23)
 (el24 T24)
 (el25 T25)
 (el26 T26)
 (el27 T27)
 (el28 T28)
 (el29 T29)
 (el30 T30)
)
)
)
)
)
(declare-datatypes ((Tuple31 31)
)
 ((par (T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15 T16 T17 T18 T19 T20 T21 T22 T23 T24 T25 T26 T27 T28 T29 T30 T31)
 ((mk-tuple31 (el1 T1)
 (el2 T2)
 (el3 T3)
 (el4 T4)
 (el5 T5)
 (el6 T6)
 (el7 T7)
 (el8 T8)
 (el9 T9)
 (el10 T10)
 (el11 T11)
 (el12 T12)
 (el13 T13)
 (el14 T14)
 (el15 T15)
 (el16 T16)
 (el17 T17)
 (el18 T18)
 (el19 T19)
 (el20 T20)
 (el21 T21)
 (el22 T22)
 (el23 T23)
 (el24 T24)
 (el25 T25)
 (el26 T26)
 (el27 T27)
 (el28 T28)
 (el29 T29)
 (el30 T30)
 (el31 T31)
)
)
)
)
)
(declare-datatype Empty ((mk-empty)
)
)
(declare-datatype IntermediateState_Normal ((pkg!Eval!ForStepi0..1/pkg!Eval!Plain0..2 (intermediate-state-Normal-pkg!Eval!ForStepi0..1/pkg!Eval!Plain0..2-local-rand Bits_n)
 (pkg!Eval!ForStepi0..1/pkg!Eval!Plain0..2-parent IntermediateState_Normal)
)
 (Normal/None)
)
)
(declare-datatype IntermediateState_Shifted ((pkg!Eval!Plain0..1 (intermediate-state-Shifted-pkg!Eval!Plain0..1-local-rand Bits_n)
 (pkg!Eval!Plain0..1-parent IntermediateState_Shifted)
)
 (pkg!Eval!ForStepi1..2/pkg!Eval!Plain0..2 (intermediate-state-Shifted-pkg!Eval!ForStepi1..2/pkg!Eval!Plain0..2-local-rand Bits_n)
 (pkg!Eval!ForStepi1..2/pkg!Eval!Plain0..2-parent IntermediateState_Shifted)
)
 (Shifted/None)
)
)
(declare-fun __sample-rand-Normal-Bits_n (Int Int)
 Bits_n)
(declare-datatype State_Normal_pkg ((mk-state-Normal-pkg (state-Normal-pkg-T (Array Int (Maybe Bits_n)
)
)
)
)
)
(declare-datatype CompositionState-Normal ((mk-composition-state-Normal (composition-pkgstate-Normal-pkg State_Normal_pkg)
 (composition-param-Normal-m Int)
 (composition-param-Normal-n Int)
 (composition-rand-Normal-0 Int)
 (composition-rand-Normal-1 Int)
 (composition-intermediate-state-Normal IntermediateState_Normal)
)
)
)
(declare-datatype Return_Normal_pkg_pkg!Eval!ForStepi0..1/pkg!Eval!Plain0..2 ((mk-return-Normal-pkg-pkg!Eval!ForStepi0..1/pkg!Eval!Plain0..2 (return-Normal-pkg-pkg!Eval!ForStepi0..1/pkg!Eval!Plain0..2-state (Array Int CompositionState-Normal)
)
 (return-Normal-pkg-pkg!Eval!ForStepi0..1/pkg!Eval!Plain0..2-state-length Int)
 (return-Normal-pkg-pkg!Eval!ForStepi0..1/pkg!Eval!Plain0..2-value (Maybe Empty)
)
 (return-Normal-pkg-pkg!Eval!ForStepi0..1/pkg!Eval!Plain0..2-is-abort Bool)
)
)
)
; Composition of Normal
(define-fun oracle-Normal-pkg-pkg!Eval!ForStepi0..1/pkg!Eval!Plain0..2 ((__global_state (Array Int CompositionState-Normal)
)
 (__state_length Int)
 (i Int)
)
 Return_Normal_pkg_pkg!Eval!ForStepi0..1/pkg!Eval!Plain0..2 (let ((rand (intermediate-state-Normal-pkg!Eval!ForStepi0..1/pkg!Eval!Plain0..2-local-rand (composition-intermediate-state-Normal (select __global_state __state_length)
)
)
)
 (__self_state (composition-pkgstate-Normal-pkg (select __global_state __state_length)
)
)
)
 (let ((rand (__sample-rand-Normal-Bits_n 1 (composition-rand-Normal-1 (select __global_state __state_length)
)
)
)
)
 (let ((__global_state (store __global_state __state_length (mk-composition-state-Normal (composition-pkgstate-Normal-pkg (select __global_state __state_length)
)
 (composition-param-Normal-m (select __global_state __state_length)
)
 (composition-param-Normal-n (select __global_state __state_length)
)
 (composition-rand-Normal-0 (select __global_state __state_length)
)
 (+ 1 (composition-rand-Normal-1 (select __global_state __state_length)
)
)
 (composition-intermediate-state-Normal (select __global_state __state_length)
)
)
)
)
)
 (let ((__self_state (mk-state-Normal-pkg (store (state-Normal-pkg-T __self_state)
 i (mk-some rand)
)
)
)
)
 (let ((__global_state (store __global_state (+ 1 __state_length)
 (mk-composition-state-Normal __self_state (composition-param-Normal-m (select __global_state __state_length)
)
 (composition-param-Normal-n (select __global_state __state_length)
)
 (composition-rand-Normal-0 (select __global_state __state_length)
)
 (composition-rand-Normal-1 (select __global_state __state_length)
)
 (composition-intermediate-state-Normal (select __global_state __state_length)
)
)
)
)
 (__state_length (+ 1 __state_length)
)
)
 (mk-return-Normal-pkg-pkg!Eval!ForStepi0..1/pkg!Eval!Plain0..2 __global_state __state_length (mk-some mk-empty)
 false)
)
)
)
)
)
)
(declare-fun __sample-rand-Shifted-Bits_n (Int Int)
 Bits_n)
(declare-datatype State_Shifted_pkg ((mk-state-Shifted-pkg (state-Shifted-pkg-T (Array Int (Maybe Bits_n)
)
)
)
)
)
(declare-datatype CompositionState-Shifted ((mk-composition-state-Shifted (composition-pkgstate-Shifted-pkg State_Shifted_pkg)
 (composition-param-Shifted-m Int)
 (composition-param-Shifted-n Int)
 (composition-rand-Shifted-0 Int)
 (composition-rand-Shifted-1 Int)
 (composition-rand-Shifted-2 Int)
 (composition-intermediate-state-Shifted IntermediateState_Shifted)
)
)
)
(declare-datatype Return_Shifted_pkg_pkg!Eval!Plain0..1 ((mk-return-Shifted-pkg-pkg!Eval!Plain0..1 (return-Shifted-pkg-pkg!Eval!Plain0..1-state (Array Int CompositionState-Shifted)
)
 (return-Shifted-pkg-pkg!Eval!Plain0..1-state-length Int)
 (return-Shifted-pkg-pkg!Eval!Plain0..1-value (Maybe Empty)
)
 (return-Shifted-pkg-pkg!Eval!Plain0..1-is-abort Bool)
)
)
)
(declare-datatype Return_Shifted_pkg_pkg!Eval!ForStepi1..2/pkg!Eval!Plain0..2 ((mk-return-Shifted-pkg-pkg!Eval!ForStepi1..2/pkg!Eval!Plain0..2 (return-Shifted-pkg-pkg!Eval!ForStepi1..2/pkg!Eval!Plain0..2-state (Array Int CompositionState-Shifted)
)
 (return-Shifted-pkg-pkg!Eval!ForStepi1..2/pkg!Eval!Plain0..2-state-length Int)
 (return-Shifted-pkg-pkg!Eval!ForStepi1..2/pkg!Eval!Plain0..2-value (Maybe Empty)
)
 (return-Shifted-pkg-pkg!Eval!ForStepi1..2/pkg!Eval!Plain0..2-is-abort Bool)
)
)
)
; Composition of Shifted
(define-fun oracle-Shifted-pkg-pkg!Eval!Plain0..1 ((__global_state (Array Int CompositionState-Shifted)
)
 (__state_length Int)
)
 Return_Shifted_pkg_pkg!Eval!Plain0..1 (let ((rand (intermediate-state-Shifted-pkg!Eval!Plain0..1-local-rand (composition-intermediate-state-Shifted (select __global_state __state_length)
)
)
)
 (__self_state (composition-pkgstate-Shifted-pkg (select __global_state __state_length)
)
)
)
 (let ((rand (__sample-rand-Shifted-Bits_n 1 (composition-rand-Shifted-1 (select __global_state __state_length)
)
)
)
)
 (let ((__global_state (store __global_state __state_length (mk-composition-state-Shifted (composition-pkgstate-Shifted-pkg (select __global_state __state_length)
)
 (composition-param-Shifted-m (select __global_state __state_length)
)
 (composition-param-Shifted-n (select __global_state __state_length)
)
 (composition-rand-Shifted-0 (select __global_state __state_length)
)
 (+ 1 (composition-rand-Shifted-1 (select __global_state __state_length)
)
)
 (composition-rand-Shifted-2 (select __global_state __state_length)
)
 (composition-intermediate-state-Shifted (select __global_state __state_length)
)
)
)
)
)
 (let ((__global_state (store __global_state (+ 1 __state_length)
 (mk-composition-state-Shifted __self_state (composition-param-Shifted-m (select __global_state __state_length)
)
 (composition-param-Shifted-n (select __global_state __state_length)
)
 (composition-rand-Shifted-0 (select __global_state __state_length)
)
 (composition-rand-Shifted-1 (select __global_state __state_length)
)
 (composition-rand-Shifted-2 (select __global_state __state_length)
)
 (composition-intermediate-state-Shifted (select __global_state __state_length)
)
)
)
)
 (__state_length (+ 1 __state_length)
)
)
 (mk-return-Shifted-pkg-pkg!Eval!Plain0..1 __global_state __state_length (mk-some mk-empty)
 false)
)
)
)
)
)
(define-fun oracle-Shifted-pkg-pkg!Eval!ForStepi1..2/pkg!Eval!Plain0..2 ((__global_state (Array Int CompositionState-Shifted)
)
 (__state_length Int)
 (i Int)
)
 Return_Shifted_pkg_pkg!Eval!ForStepi1..2/pkg!Eval!Plain0..2 (let ((rand (intermediate-state-Shifted-pkg!Eval!ForStepi1..2/pkg!Eval!Plain0..2-local-rand (composition-intermediate-state-Shifted (select __global_state __state_length)
)
)
)
 (__self_state (composition-pkgstate-Shifted-pkg (select __global_state __state_length)
)
)
)
 (let ((__self_state (mk-state-Shifted-pkg (store (state-Shifted-pkg-T __self_state)
 i (mk-some rand)
)
)
)
)
 (let ((rand (__sample-rand-Shifted-Bits_n 2 (composition-rand-Shifted-2 (select __global_state __state_length)
)
)
)
)
 (let ((__global_state (store __global_state __state_length (mk-composition-state-Shifted (composition-pkgstate-Shifted-pkg (select __global_state __state_length)
)
 (composition-param-Shifted-m (select __global_state __state_length)
)
 (composition-param-Shifted-n (select __global_state __state_length)
)
 (composition-rand-Shifted-0 (select __global_state __state_length)
)
 (composition-rand-Shifted-1 (select __global_state __state_length)
)
 (+ 1 (composition-rand-Shifted-2 (select __global_state __state_length)
)
)
 (composition-intermediate-state-Shifted (select __global_state __state_length)
)
)
)
)
)
 (let ((__global_state (store __global_state (+ 1 __state_length)
 (mk-composition-state-Shifted __self_state (composition-param-Shifted-m (select __global_state __state_length)
)
 (composition-param-Shifted-n (select __global_state __state_length)
)
 (composition-rand-Shifted-0 (select __global_state __state_length)
)
 (composition-rand-Shifted-1 (select __global_state __state_length)
)
 (composition-rand-Shifted-2 (select __global_state __state_length)
)
 (composition-intermediate-state-Shifted (select __global_state __state_length)
)
)
)
)
 (__state_length (+ 1 __state_length)
)
)
 (mk-return-Shifted-pkg-pkg!Eval!ForStepi1..2/pkg!Eval!Plain0..2 __global_state __state_length (mk-some mk-empty)
 false)
)
)
)
)
)
)
(declare-const state-left (Array Int CompositionState-Normal)
)
(declare-const state-right (Array Int CompositionState-Shifted)
)
(declare-const state-length-left-old Int)
(declare-const state-length-left-new Int)
(declare-const state-length-right-old Int)
(declare-const state-length-right-new Int)
(declare-const arg-Normal-pkg!Eval!ForStepi0..1/pkg!Eval!Plain0..2-i Int)
(declare-const arg-Shifted-pkg!Eval!ForStepi1..2/pkg!Eval!Plain0..2-i Int)
(declare-const return-left-pkg-pkg!Eval!ForStepi0..1/pkg!Eval!Plain0..2 Return_Normal_pkg_pkg!Eval!ForStepi0..1/pkg!Eval!Plain0..2)
(assert (= return-left-pkg-pkg!Eval!ForStepi0..1/pkg!Eval!Plain0..2 (oracle-Normal-pkg-pkg!Eval!ForStepi0..1/pkg!Eval!Plain0..2 state-left state-length-left-old arg-Normal-pkg!Eval!ForStepi0..1/pkg!Eval!Plain0..2-i)
)
)
(declare-const return-right-pkg-pkg!Eval!Plain0..1 Return_Shifted_pkg_pkg!Eval!Plain0..1)
(assert (= return-right-pkg-pkg!Eval!Plain0..1 (oracle-Shifted-pkg-pkg!Eval!Plain0..1 state-right state-length-right-old)
)
)
(declare-const return-right-pkg-pkg!Eval!ForStepi1..2/pkg!Eval!Plain0..2 Return_Shifted_pkg_pkg!Eval!ForStepi1..2/pkg!Eval!Plain0..2)
(assert (= return-right-pkg-pkg!Eval!ForStepi1..2/pkg!Eval!Plain0..2 (oracle-Shifted-pkg-pkg!Eval!ForStepi1..2/pkg!Eval!Plain0..2 state-right state-length-right-old arg-Shifted-pkg!Eval!ForStepi1..2/pkg!Eval!Plain0..2-i)
)
)
(declare-const randctr-left-1 Int)
(assert (= randctr-left-1 (composition-rand-Normal-1 (select state-left state-length-left-old)
)
)
)
(declare-const randval-left-1 Bits_n)
(assert (= randval-left-1 (__sample-rand-Normal-Bits_n 1 (+ 0 randctr-left-1)
)
)
)
(declare-const randctr-right-1 Int)
(assert (= randctr-right-1 (composition-rand-Shifted-1 (select state-right state-length-right-old)
)
)
)
(declare-const randval-right-1 Bits_n)
(assert (= randval-right-1 (__sample-rand-Shifted-Bits_n 1 (+ 0 randctr-right-1)
)
)
)
(declare-const randctr-right-2 Int)
(assert (= randctr-right-2 (composition-rand-Shifted-2 (select state-right state-length-right-old)
)
)
)
(declare-const randval-right-2 Bits_n)
(assert (= randval-right-2 (__sample-rand-Shifted-Bits_n 2 (+ 0 randctr-right-2)
)
)
)
(push 1)