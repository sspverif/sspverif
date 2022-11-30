(declare-fun randomness-mapping-GBLG (
        (Array Int CompositionState-Left)
        (Array Int CompositionState-Right)
        Return_Left_gate_GBLG
        Return_Right_simgate_GBLG
        Int ;h
        Int ;l
        Int ;r
        (Array (Tuple2 Bool Bool) (Maybe Bool)) ;op
        Int) ;j
    Bool)
(declare-fun invariant-GBLG          (
        (Array Int CompositionState-Left)
        (Array Int CompositionState-Right)
        Return_Left_gate_GBLG
        Return_Right_simgate_GBLG
        Int
        Int
        Int
        (Array (Tuple2 Bool Bool) (Maybe Bool))
        Int)
    Bool)
(declare-fun foo          (
        (Array Int CompositionState-Left)
        (Array Int CompositionState-Right)
        Return_Left_gate_GBLG
        Return_Right_simgate_GBLG
        Int
        Int
        Int
        (Array (Tuple2 Bool Bool) (Maybe Bool))
        Int)
    Bool)
(declare-fun bar          (
        (Array Int CompositionState-Left)
        (Array Int CompositionState-Right)
        Return_Left_gate_GBLG
        Return_Right_simgate_GBLG
        Int
        Int
        Int
        (Array (Tuple2 Bool Bool) (Maybe Bool))
        Int)
    Bool)