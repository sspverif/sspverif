; Main idea of this invariant proof
; If ltk (or K) are equal (or use the same randomness), then both games 
;    - produce the same output
;    - abort iff the other aborts
;    - have same ltk and same table K afterwards

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;
; Randomness mapping --- there is only randomness for ltk
;
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

(define-fun
    randomness-mapping-PRF
    (
        (ctr-mon     Int)
        (ctr-mod     Int) 
        (id-mon      Int)
        (id-mod      Int) 
        (icr-mon     Int)
        (icr-mod     Int)
    )
    Bool
(and
(= ctr-mon ctr-mod)
(= ctr-mon icr-mod)
(= ctr-mon icr-mod)
(= id-mon  1)
(= id-mod 1)
)
)

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;
; Invariant --- note that the invariant needs to be game-global 
;               Having different variants for EVAL & GET allows to prove wrong things.
;
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

(define-fun invariant-PRF      (
        (state-mod  <GameState_Modprfreal_<$<!n!>$>> )
        (state-mon  <GameState_Monprfreal_<$<!n!>$>>)
)
    Bool
    (let

; getting ltk and table K out of state
(
(ltk-mod (<state-modPRF-<$<!n!>$>-ltk>     (<game-Modprfreal-<$<!n!>$>-pkgstate-mod_prf> state-mod)))
(  K-mod (<state-KeyReal-<$<!n!>$>-K>      (<game-Modprfreal-<$<!n!>$>-pkgstate-key> state-mod)))
(ltk-mon (<state-PRFReal-<$<!n!>$>-ltk>    (<game-Monprfreal-<$<!n!>$>-pkgstate-prf> state-mon)))
(  K-mon (<state-ReductionPRF-<$<!n!>$>-K> (<game-Monprfreal-<$<!n!>$>-pkgstate-red> state-mon)))
)

; ltk are equal
; K   are equal
; randomness counters are equal

(and
(= ltk-mod ltk-mon)
(= K-mod   K-mon)
(= ctr-mon ctr-mod)
)

))
