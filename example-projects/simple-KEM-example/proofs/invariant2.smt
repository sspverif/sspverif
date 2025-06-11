(define-fun invariant ( 
  (prot    <GameState_Prot_<$$>>)
  (prot2   <GameState_Prot_<$$>>)
) Bool
; BEGIN FUNCTION BODY
  (= prot prot2))

; Each sample operation is fully indexec by the pair (statement id, sample counter)
; "stmt" – Each instructions containing a sampling operation in the game is assigned a statement id number; check the generated latex code for the proof (not games/compositions or packages) to find the statement ids.
; "ctr" – Each sample operation also has a counter
;
; Additionally, we are given a zero-counter; this would somehow be useful if we did more complex induction stuff
; "base-ctr"
;
; To add some precision, the "ctr" explained above is really the offset from the zero counter. I.e. to derive the underlying counter,
; you could calculate (+ ctr-left ctr-left-0), reversing the difference.
;
; These indices are given for both games; the game on the left and the game on the right.
(define-fun randomness-mapping-GetPK (
  (base-ctr-left Int) 
  (base-ctr-right Int)
  (stmt-left  Int) 
  (stmt-right  Int)
  (ctr-left Int)
  (ctr-right Int)
) Bool
; BEGIN FUNCTION BODY
  (and
    (= stmt-left stmt-right)
    (= ctr-left  ctr-right)
  )
)

(define-fun randomness-mapping-Run (
  (base-ctr-left Int) 
  (base-ctr-right Int)
  (stmt-left  Int) 
  (stmt-right  Int)
  (ctr-left Int)
  (ctr-right Int)
) Bool
; BEGIN FUNCTION BODY
  (and
    (= stmt-left stmt-right)
    (= ctr-left  ctr-right)
  )
)

(define-fun randomness-mapping-TestSender (
  (base-ctr-left Int) 
  (base-ctr-right Int)
  (stmt-left  Int) 
  (stmt-right  Int)
  (ctr-left Int)
  (ctr-right Int)
) Bool
; BEGIN FUNCTION BODY
  (and
    (= stmt-left stmt-right)
    (= ctr-left  ctr-right)
  )
)

(define-fun randomness-mapping-TestReceiver (
  (base-ctr-left Int) 
  (base-ctr-right Int)
  (stmt-left  Int) 
  (stmt-right  Int)
  (ctr-left Int)
  (ctr-right Int)
) Bool
; BEGIN FUNCTION BODY
  (and
    (= stmt-left stmt-right)
    (= ctr-left  ctr-right)
  )
)
