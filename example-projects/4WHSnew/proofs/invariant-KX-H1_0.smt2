; Main idea of this invariant proof
; If ctr are equal in both games and they use the same randomness, then both games 
;    - produce the same output
;    - abort iff the other aborts
;    - have same ctr afterwards

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;
; Randomness mapping
;
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

(define-fun Send1
  ( (base-ctr-0 Int) ; This is the counter in the beginning of the oracle call on the left.
    (base-ctr-1 Int) ; This is the counter in the beginning of the oracle call on the left.
    (id-0  Int)      ; This is the sample-id, see LaTeX export for which id corresponds to which sampling.
    (id-1  Int)      ; This is the sample-id, see LaTeX export for which id corresponds to which sampling.
    (scr-0 Int)      ; This is the counter which gets incremented each time a sampling is done with the same sample id.
    (scr-1 Int))     ; This is the counter which gets incremented each time a sampling is done with the same sample id.
  Bool
  (and
    (= scr-1 base-ctr-1)
    (= scr-0 base-ctr-0)
    (= id-0      2)  ; Sampling of ni in Nonces
    (= id-1      2)  ; Sampling of ni in Nonces
    ))

(define-fun Send2
  ( (base-ctr-0 Int) ; This is the counter in the beginning of the oracle call on the left.
    (base-ctr-1 Int) ; This is the counter in the beginning of the oracle call on the left.
    (id-0  Int)      ; This is the sample-id, see LaTeX export for which id corresponds to which sampling.
    (id-1  Int)      ; This is the sample-id, see LaTeX export for which id corresponds to which sampling.
    (scr-0 Int)      ; This is the counter which gets incremented each time a sampling is done with the same sample id.
    (scr-1 Int))     ; This is the counter which gets incremented each time a sampling is done with the same sample id.
  Bool
  (and
    (= scr-1 base-ctr-1)
    (= scr-0 base-ctr-0)
    (= id-0      2)  ; Sampling of nr in Nonces
    (= id-1      2)  ; Sampling of nr in Nonces
  ))

(define-fun Send3
  ( (base-ctr-0 Int) ; This is the counter in the beginning of the oracle call on the left.
    (base-ctr-1 Int) ; This is the counter in the beginning of the oracle call on the left.
    (id-0  Int)      ; This is the sample-id, see LaTeX export for which id corresponds to which sampling.
    (id-1  Int)      ; This is the sample-id, see LaTeX export for which id corresponds to which sampling.
    (scr-0 Int)      ; This is the counter which gets incremented each time a sampling is done with the same sample id.
    (scr-1 Int))     ; This is the counter which gets incremented each time a sampling is done with the same sample id.
  Bool
                     ; There is no randomness used in this oracle.
					 false
)

(define-fun Send4
  ( (base-ctr-0 Int) ; This is the counter in the beginning of the oracle call on the left.
    (base-ctr-1 Int) ; This is the counter in the beginning of the oracle call on the left.
    (id-0  Int)      ; This is the sample-id, see LaTeX export for which id corresponds to which sampling.
    (id-1  Int)      ; This is the sample-id, see LaTeX export for which id corresponds to which sampling.
    (scr-0 Int)      ; This is the counter which gets incremented each time a sampling is done with the same sample id.
    (scr-1 Int))     ; This is the counter which gets incremented each time a sampling is done with the same sample id.
  Bool
                     ; There is no randomness used in this oracle.
					 false
)

(define-fun Send5
  ( (base-ctr-0 Int) ; This is the counter in the beginning of the oracle call on the left.
    (base-ctr-1 Int) ; This is the counter in the beginning of the oracle call on the left.
    (id-0  Int)      ; This is the sample-id, see LaTeX export for which id corresponds to which sampling.
    (id-1  Int)      ; This is the sample-id, see LaTeX export for which id corresponds to which sampling.
    (scr-0 Int)      ; This is the counter which gets incremented each time a sampling is done with the same sample id.
    (scr-1 Int))     ; This is the counter which gets incremented each time a sampling is done with the same sample id.
  Bool
                     ; There is no randomness used in this oracle.
					 false
)

(define-fun Reveal
  ( (base-ctr-0 Int) ; This is the counter in the beginning of the oracle call on the left.
    (base-ctr-1 Int) ; This is the counter in the beginning of the oracle call on the left.
    (id-0  Int)      ; This is the sample-id, see LaTeX export for which id corresponds to which sampling.
    (id-1  Int)      ; This is the sample-id, see LaTeX export for which id corresponds to which sampling.
    (scr-0 Int)      ; This is the counter which gets incremented each time a sampling is done with the same sample id.
    (scr-1 Int))     ; This is the counter which gets incremented each time a sampling is done with the same sample id.
  Bool
                     ; There is no randomness used in this oracle.
					 false
)

(define-fun Test
  ( (base-ctr-0 Int) ; This is the counter in the beginning of the oracle call on the left.
    (base-ctr-1 Int) ; This is the counter in the beginning of the oracle call on the left.
    (id-0  Int)      ; This is the sample-id, see LaTeX export for which id corresponds to which sampling.
    (id-1  Int)      ; This is the sample-id, see LaTeX export for which id corresponds to which sampling.
    (scr-0 Int)      ; This is the counter which gets incremented each time a sampling is done with the same sample id.
    (scr-1 Int))     ; This is the counter which gets incremented each time a sampling is done with the same sample id.
  Bool
  (and
    (= scr-1 base-ctr-1)
    (= id-0     1)   ; This is the 1st sampling in H1_1 and samples the random key in Test.
    (= id-1     1)   ; This is the 1st sampling in H2_0 and samples the random key in Test.
))

(define-fun NewKey
  ( (base-ctr-0 Int) ; This is the counter in the beginning of the oracle call on the left.
    (base-ctr-1 Int) ; This is the counter in the beginning of the oracle call on the left.
    (id-0  Int)      ; This is the sample-id, see LaTeX export for which id corresponds to which sampling.
    (id-1  Int)      ; This is the sample-id, see LaTeX export for which id corresponds to which sampling.
    (scr-0 Int)      ; This is the counter which gets incremented each time a sampling is done with the same sample id.
    (scr-1 Int))     ; This is the counter which gets incremented each time a sampling is done with the same sample id.
  Bool
  (and
    (= scr-1 base-ctr-1)
    (= scr-0 base-ctr-0)
    (= id-0     0)   ; This is the 0th sampling in H1_1 and samples the random key in NewKey.
    (= id-1     0)   ; This is the 0th sampling in H2_0 and samples the random key in NewKey.
  ))

(define-fun NewSession
  ( (base-ctr-0 Int) ; This is the counter in the beginning of the oracle call on the left.
    (base-ctr-1 Int) ; This is the counter in the beginning of the oracle call on the left.
    (id-0  Int)      ; This is the sample-id, see LaTeX export for which id corresponds to which sampling.
    (id-1  Int)      ; This is the sample-id, see LaTeX export for which id corresponds to which sampling.
    (scr-0 Int)      ; This is the counter which gets incremented each time a sampling is done with the same sample id.
    (scr-1 Int))     ; This is the counter which gets incremented each time a sampling is done with the same sample id.
  Bool
                     ; There is no randomness used in this oracle.
					 false
)

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;                                                                                                      ;
; Invariant --- note that the invariant needs to be game-global and not per oracle,                    ;
;               so that induction over the oracle calls remains meaningful.                            ;
;                                                                                                      ;
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

(define-fun invariant
  ( (state-kx     <GameState_KX_<$<!n!><!b!><!zeron!>$>>)
    (state-kxred  <GameState_H1_<$<!n!><!b!><!false!><!zeron!>$>>))
  Bool
; getting package-state out of game-state and demanding equality, they should be exactly the same in this case.
(=
(<game-KX-<$<!n!><!b!><!zeron!>$>-pkgstate-Game>     state-kx)    ;some params are still missing.
(<game-H1-<$<!n!><!b!><!false!><!zeron!>$>-pkgstate-Game> state-kxred) ;some params are still missing.
)

;  (let
;    ; getting ctr out of state
;    ( (ctr-kxred (<pkg-state-Rand-<$<!n!>$>-ctr> (<game-SmallComposition-<$<!n!>$>-pkgstate-rand> state-0)))
;      (ctr-kx (<pkg-state-Rand-<$<!n!>$>-ctr> (<game-MediumComposition-<$<!n!>$>-pkgstate-rand> state-1))))
;
;    ; ctr are equal
;    (= ctr-kxred ctr-kx))

)
