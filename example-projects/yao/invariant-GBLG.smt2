; Goal:
; Use equality on KEYS packages as invariant
; prove that on same inputs, all oracles give same output

; possible input for all (and sufficient for GETKEYSIN/GETKEYSOUT):
(declare-const handle Int)

; possible input for SETBIT
(declare-const bit Bool)

; possible input for GBLG     oracle GBLG(h: Integer, l: Integer, r: Integer, op: fn Bool,Bool -> Bool, j: Integer) -> Table(Bits(p),Bool) {
(declare-const l Int)
(declare-const r Int)
(declare-fun op (Bool Bool) Bool)

; possible state
(declare-const state-left-old CompositionState-Left)
(declare-const state-right-old CompositionState-Right)
(declare-const state-left-new CompositionState-Left)
(declare-const state-right-new CompositionState-Right)

; return value - type depends on oracle call; only GBLG is interesting, actually.
(declare-const return-left Return_Left_gate_GBLG)
(declare-const return-right Return_Right_simgate_GBLG)
(declare-const is-abort-left Bool)
(declare-const is-abort-right Bool)


(assert (and  ;assignment of return (value,state)
              (= return-left      (oracle-Left-gate-GBLG state-left-old handle l r op))
              (= return-right     (oracle-Right-simgate-GBLG state-right-old handle l r op))

              ;assignment of return values
              (= value-left       (return-Left-gate-GBLG-value return-left))
              (= value-right      (return-Right-simgate-GBLG-value return-right))

              ;assignment of abort values
              (= is-abort-left    (= mk-abort-Left-gate-GBLG return-left))
              (= is-abort-right   (= mk-abort-Right-simgate-GBLG return-right))

              ;assignment of return state
              (= state-left-new   (return-Left-gate-GBLG-state return-left))
              (= state-right-new  (return-Right-simgate-GBLG-state return-right))))

              ;Here, we need to additionally say something about sample instructions for lower Key package


; The 2 top key packages should have the same state on the left and the right.
(define-fun key-top-lr-eq ((left CompositionState-Left) (right CompositionState-Right)) Bool
  (forall ((h Int)) (=  (select (state-Left-key_top-T (composition-state-Left-key_top left))
                                h)
                        (select (state-Right-key_top-T (composition-state-Right-key_top right))
                                h))))

; Left: The state of the top key package does not change when GBLG is called
(define-fun key-top-ll-eq ((old CompositionState-Left) (new CompositionState-Left)) Bool
  (forall ((h Int)) (=  (select (state-Left-key_top-T (composition-state-Left-key_top old))
                                h)
                        (select (state-Left-key_top-T (composition-state-Left-key_top new))
                                h))))

; Right: The state of the top key package does not change when GBLG is called
(define-fun key-top-rr-eq ((old CompositionState-Right) (new CompositionState-Right)) Bool
  (forall ((h Int)) (=  (select (state-Right-key_top-T (composition-state-Right-key_top old))
                                h)
                        (select (state-Right-key_top-T (composition-state-Right-key_top new))
                                h))))

; Left: The state of the bottom key package is mostly the same as before except at the point h j r op that was written to
(define-fun key-bottom-l-mostly-eq ((old CompositionState-Left) (new CompositionState-Left) 
                                    (h Int) ) Bool
  (forall ((hh Int))  (or (= h hh)
                                      (=  (select (state-Left-key_bottom-T (composition-state-Left-key_bottom new))
                                                  hh)
                                          (select (state-Left-key_bottom-T (composition-state-Left-key_bottom old))
                                                  hh)))))


; The state of the bottom key package on the right is mostly the same as before except at the point h that was written to
(define-fun key-bottom-r-mostly-eq ((old CompositionState-Right) (new CompositionState-Right) 
                                    (h Int)) Bool
  (forall ((hh Int))  (or (= h hh)
                                      (=  (select (state-Right-key_bottom-T (composition-state-Right-key_bottom new))
                                                  hh)
                                          (select (state-Right-key_bottom-T (composition-state-Right-key_bottom old))
                                                  hh)))))

; Right: The state of the bottom key package at position h is equal to what was sampled (or was defined before).
(define-fun key-bottom-r-ok-after-call ((old CompositionState-Right) (new CompositionState-Right) (h Int)) Bool 
  (=      (maybe-get (select  (state-Right-key_bottom-T (composition-state-Right-key_bottom new))
                              h))
                              ; put randomness sampling here XXX
                              ; if it was none before, then it is equal to sample now
      (XXX)))

; The state of the bottom key package on the left at position h is equal to what was sampled (or was defined before)).
(define-fun key-bottom-l-ok-after-call ((old CompositionState-Left) (new CompositionState-Left) (h Int)) Bool 
  (=      (maybe-get (select  (state-Left-key_bottom-T (composition-state-Left-key_bottom new))
                              h))
                              ; put randomness sampling here XXX
                              ; if it was none before, then it is equal to sample now
                              
      (XXX)))

; should this really use the old state?? Not here
(define-fun post-condition ((left CompositionState-Left) (right CompositionState-Right) (h Int)) Bool
  (forall ((h Int)) (=  (select (state-Left-key_top-T (composition-state-Left-key_top left))
                                h)
                        (select (state-Right-key_top-T (composition-state-Right-key_top  right))
                                h))))


(declare-const precondition-holds Bool)
(assert (= precondition-holds (and  (not is-abort-right)
                                    (not is-abort-left)
                                    (key-bottom-ok state-right-old)
                                    (key-top-lr-eq state-left-old state-right-old))))

;; This is just to verify that the current state is satisfiable. It definitely should be.
(check-sat)

(push 1)
;;; prove right bottom key mostly equal lemma
(assert (and  precondition-holds
              (not
                ;; proved statement starts here
                (key-bottom-mostly-eq state-right-old state-right-new handle l r op))))
(check-sat)
(pop 1)

(push 1)
;;; prove right bottom keys wellformed after call lemma
(assert (and  precondition-holds
              ;; lemmata start here
              (not (key-bottom-ok-after-call state-right-old state-right-new handle l r op))))
(check-sat)
(pop 1)



(push 1)
;; prove left-left lemma
(assert (and  precondition-holds
              (not
                ;; proved statement starts here
                (key-top-ll-eq state-left-old state-left-new))))
(check-sat)
(pop 1)
;
;; prove right-right lemma
(push 1)
(assert (and  precondition-holds
              (not
                ;; proved statement starts here
                (key-top-rr-eq state-right-old state-right-new))))
(check-sat)
(pop 1)



(push 1)
; prove left-right lemma
(assert (and    precondition-holds
                ;; lemmata start here
                (key-top-rr-eq state-right-old state-right-new)
                (key-top-ll-eq state-left-old state-left-new)
                (not
                    ;; proved statement starts here
                    (key-top-lr-eq state-left-new state-right-new))))
(check-sat)
(pop 1)


;; check that the post-condition follows
(push 1)
(assert (and    precondition-holds
                ;;; lemmata start here
                (key-top-ll-eq state-left-old state-left-new)
                (key-top-rr-eq state-right-old state-right-new)
                (key-top-lr-eq state-left-new state-right-new)
                (key-bottom-mostly-eq state-right-old state-right-new handle l r op)
                (key-bottom-ok-after-call state-right-old state-right-new handle l r op)
                (or
                    (not (post-condition state-left-new state-right-new handle l r op))
                    (not (key-bottom-ok state-right-new)))))
(check-sat)
(pop 1)

; this should not be a problem.
; the fact that this is a problem might be informative.
(push 1)
;(assert (key-tables-empty state-right-old))

;(assert (and  (right-key-bottom-set-implies-top-set state-right-old)))
              ;(right-key-top-set-implies-bottom-set state-right-old)))
      
(assert (key-bottom-ok state-right-old))
(check-sat)
;(get-model)
(pop 1)

;; this also shouldn't be a problem, but probably t his problem is just caused by the above problem.
(push 1)
;; check that there is a valid assignment for the precondition
(assert precondition-holds)
(check-sat)
;(get-model)
(pop 1)