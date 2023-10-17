(set-logic ALL)
(declare-sort Bits_p 0)
(declare-sort Bits_n 0)
(declare-sort Bits_m 0)
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
(declare-datatypes ((ReturnValue 1)
)
 ((par (T)
 ((mk-return-value (return-value T)
)
 (mk-abort)
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
(declare-fun __sample-rand-Indcpamod0-Bits_n (Int Int)
 Bits_n)
(declare-fun __func-Indcpamod0-encm (Bits_n Bits_m Bits_n)
 Bits_p)
(declare-fun __func-Indcpamod0-encn (Bits_n Bits_n Bits_n)
 Bits_m)
(declare-datatype State_Indcpamod0_keys_top ((mk-state-Indcpamod0-keys_top (state-Indcpamod0-keys_top-T (Maybe (Array Bool (Maybe Bits_n)
)
)
)
 (state-Indcpamod0-keys_top-z (Maybe Bool)
)
 (state-Indcpamod0-keys_top-flag (Maybe Bool)
)
)
)
)
(declare-datatype State_Indcpamod0_enc ((mk-state-Indcpamod0-enc)
)
)
(declare-datatype CompositionState-Indcpamod0 ((mk-composition-state-Indcpamod0 (composition-pkgstate-Indcpamod0-keys_top State_Indcpamod0_keys_top)
 (composition-pkgstate-Indcpamod0-enc State_Indcpamod0_enc)
 (composition-param-Indcpamod0-m Int)
 (composition-param-Indcpamod0-n Int)
 (composition-param-Indcpamod0-p Int)
 (composition-param-Indcpamod0-zerom Bits_m)
 (composition-param-Indcpamod0-zeron Bits_n)
 (composition-rand-Indcpamod0-0 Int)
 (composition-rand-Indcpamod0-1 Int)
 (composition-rand-Indcpamod0-2 Int)
 (composition-rand-Indcpamod0-3 Int)
 (composition-rand-Indcpamod0-4 Int)
 (composition-rand-Indcpamod0-5 Int)
 (composition-rand-Indcpamod0-6 Int)
)
)
)
(declare-datatype Return_Indcpamod0_keys_top_GETKEYSIN ((mk-return-Indcpamod0-keys_top-GETKEYSIN (return-Indcpamod0-keys_top-GETKEYSIN-state CompositionState-Indcpamod0)
 (return-Indcpamod0-keys_top-GETKEYSIN-value (Maybe (Array Bool (Maybe Bits_n)
)
)
)
 (return-Indcpamod0-keys_top-GETKEYSIN-is-abort Bool)
)
)
)
(declare-datatype Return_Indcpamod0_keys_top_GETAIN ((mk-return-Indcpamod0-keys_top-GETAIN (return-Indcpamod0-keys_top-GETAIN-state CompositionState-Indcpamod0)
 (return-Indcpamod0-keys_top-GETAIN-value (Maybe Bits_n)
)
 (return-Indcpamod0-keys_top-GETAIN-is-abort Bool)
)
)
)
(declare-datatype Return_Indcpamod0_keys_top_GETINAIN ((mk-return-Indcpamod0-keys_top-GETINAIN (return-Indcpamod0-keys_top-GETINAIN-state CompositionState-Indcpamod0)
 (return-Indcpamod0-keys_top-GETINAIN-value (Maybe Bits_n)
)
 (return-Indcpamod0-keys_top-GETINAIN-is-abort Bool)
)
)
)
(declare-datatype Return_Indcpamod0_keys_top_GETAOUT ((mk-return-Indcpamod0-keys_top-GETAOUT (return-Indcpamod0-keys_top-GETAOUT-state CompositionState-Indcpamod0)
 (return-Indcpamod0-keys_top-GETAOUT-value (Maybe Bits_n)
)
 (return-Indcpamod0-keys_top-GETAOUT-is-abort Bool)
)
)
)
(declare-datatype Return_Indcpamod0_keys_top_GETKEYSOUT ((mk-return-Indcpamod0-keys_top-GETKEYSOUT (return-Indcpamod0-keys_top-GETKEYSOUT-state CompositionState-Indcpamod0)
 (return-Indcpamod0-keys_top-GETKEYSOUT-value (Maybe (Array Bool (Maybe Bits_n)
)
)
)
 (return-Indcpamod0-keys_top-GETKEYSOUT-is-abort Bool)
)
)
)
(declare-datatype Return_Indcpamod0_keys_top_GETBIT ((mk-return-Indcpamod0-keys_top-GETBIT (return-Indcpamod0-keys_top-GETBIT-state CompositionState-Indcpamod0)
 (return-Indcpamod0-keys_top-GETBIT-value (Maybe Bool)
)
 (return-Indcpamod0-keys_top-GETBIT-is-abort Bool)
)
)
)
(declare-datatype Return_Indcpamod0_keys_top_SETBIT ((mk-return-Indcpamod0-keys_top-SETBIT (return-Indcpamod0-keys_top-SETBIT-state CompositionState-Indcpamod0)
 (return-Indcpamod0-keys_top-SETBIT-value (Maybe Empty)
)
 (return-Indcpamod0-keys_top-SETBIT-is-abort Bool)
)
)
)
(declare-datatype Return_Indcpamod0_enc_ENCN ((mk-return-Indcpamod0-enc-ENCN (return-Indcpamod0-enc-ENCN-state CompositionState-Indcpamod0)
 (return-Indcpamod0-enc-ENCN-value (Maybe Bits_m)
)
 (return-Indcpamod0-enc-ENCN-is-abort Bool)
)
)
)
(declare-datatype Return_Indcpamod0_enc_ENCM ((mk-return-Indcpamod0-enc-ENCM (return-Indcpamod0-enc-ENCM-state CompositionState-Indcpamod0)
 (return-Indcpamod0-enc-ENCM-value (Maybe Bits_p)
)
 (return-Indcpamod0-enc-ENCM-is-abort Bool)
)
)
)
; Composition of Indcpamod0
(define-fun oracle-Indcpamod0-keys_top-GETKEYSIN ((__global_state (Array Int CompositionState-Indcpamod0)
)
)
 Return_Indcpamod0_keys_top_GETKEYSIN (let ((__self_state (composition-pkgstate-Indcpamod0-keys_top __global_state)
)
)
 