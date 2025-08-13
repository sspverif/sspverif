(define-fun invariant
  ( (l <GameState_Simple>)
    (r <GameState_Simple>))
  Bool
  (= l r))

(define-fun randomness-mapping-Sample
  ( (base-ctr-left Int) 
    (base-ctr-right Int)
    (stmt-left  Int) 
    (stmt-right  Int)
    (ctr-left Int)
    (ctr-right Int))
  Bool
  (and
    (= stmt-left stmt-right)
    (= (- ctr-left base-ctr-left)
       (- ctr-right base-ctr-right))))
