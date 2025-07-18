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

(define-fun randomness-mapping-Send1
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
    (= id-0      0)  ; This is the 2nd sampling in KX and samples ni.
    (= id-1      0)  ; This sampling happens in the Nonces package and is the 2nd sampling (in fact the last sampling, because Nonces is defined last).
    ))

(define-fun randomness-mapping-Send2
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
    (= id-0     0)   ; This is the 3rd sampling in KX and samples nr.
    (= id-1     0)   ; This sampling happens in the Nonces package and is the 2nd sampling (in fact the last sampling, because Nonces is defined last).
  ))

(define-fun randomness-mapping-Send3
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

(define-fun randomness-mapping-Send4
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

(define-fun randomness-mapping-Send5
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

(define-fun randomness-mapping-Reveal
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

(define-fun randomness-mapping-Test
  ( (base-ctr-0 Int) ; This is the counter in the beginning of the oracle call on the left.
    (base-ctr-1 Int) ; This is the counter in the beginning of the oracle call on the left.
    (id-0  Int)      ; This is the sample-id, see LaTeX export for which id corresponds to which sampling.
    (id-1  Int)      ; This is the sample-id, see LaTeX export for which id corresponds to which sampling.
    (scr-0 Int)      ; This is the counter which gets incremented each time a sampling is done with the same sample id.
    (scr-1 Int))     ; This is the counter which gets incremented each time a sampling is done with the same sample id.
  Bool
  (and
    (= scr-1 base-ctr-1)
    (= id-0     2)   ; This is the 1st sampling in KX   and samples the random key in Test.
    (= id-1     2)   ; This is the 1st sampling in H1_0 and samples the random key in Test.
))

(define-fun randomness-mapping-NewKey
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
    (= id-0     1)   ; This is the 0th sampling in KX   and samples the random key in NewKey.
    (= id-1     1)   ; This is the 0th sampling in H1_0 and samples the random key in NewKey.
  ))

(define-fun randomness-mapping-NewSession
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

(define-fun randomness-mapping-SameKey
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
(define-fun randomness-mapping-AtMost
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
(define-fun randomness-mapping-AtLeast
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
  ( (state-H3  <GameState_H3_<$<!n!><!b!><!false!><!zeron!>$>>)
    (state-H4  <GameState_H4_<$<!n!><!b!><!zeron!>$>>))
  Bool
  (and (= (<game-H3-<$<!n!><!b!><!false!><!zeron!>$>-pkgstate-Game_nochecks> state-H3)
          (<game-H4-<$<!n!><!b!><!zeron!>$>-pkgstate-Game_nochecks>          state-H4))
       (= (<pkg-state-Nonces-<$<!true!><!n!>$>-Nonces>
	       (<game-H3-<$<!n!><!b!><!false!><!zeron!>$>-pkgstate-Nonces>     state-H3))
	      (<pkg-state-Nonces-<$<!true!><!n!>$>-Nonces>
		   (<game-H4-<$<!n!><!b!><!zeron!>$>-pkgstate-Nonces> state-H4))))

;  (let
;    ; getting ctr out of state
;    ( (ctr-kxred (<pkg-state-Rand-<$<!n!>$>-ctr> (<game-SmallComposition-<$<!n!>$>-pkgstate-rand> state-0)))
;      (ctr-kx (<pkg-state-Rand-<$<!n!>$>-ctr> (<game-MediumComposition-<$<!n!>$>-pkgstate-rand> state-1))))
;
;    ; ctr are equal
;    (= ctr-kxred ctr-kx))

)
