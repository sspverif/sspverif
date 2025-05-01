(define-fun invariant
  ( (gstate-prot <GameState_Prot_<$<!false!>$>>)
    (gstate-h1   <GameState_H1_<$<!false!>$>>))
  Bool
  (let
    ; bind package states
    ( (pstate-prot-prot   (<game-Prot-<$<!false!>$>-pkgstate-Prot>         gstate-prot))
      (pstate-h1-corr_red (<game-H1-<$<!false!>$>-pkgstate-Corr_reduction> gstate-h1))
      (pstate-h1-corr_kem (<game-H1-<$<!false!>$>-pkgstate-Corr_KEM> gstate-h1)))
    (let
      ; bind state of both games
      ( (var-prot-ctr (<pkg-state-Prot-<$<!isideal!>$>-ctr> pstate-prot-prot))
        (var-prot-TESTED  (<pkg-state-Prot-<$<!isideal!>$>-TESTED>  pstate-prot-prot))
        (var-prot-RECEIVEDKEY  (<pkg-state-Prot-<$<!isideal!>$>-RECEIVEDKEY>  pstate-prot-prot))
        (var-prot-RECEIVEDCTXT  (<pkg-state-Prot-<$<!isideal!>$>-RECEIVEDCTXT>  pstate-prot-prot))
        (var-prot-SENTKEY  (<pkg-state-Prot-<$<!isideal!>$>-SENTKEY>  pstate-prot-prot))
        (var-prot-SENTCTXT  (<pkg-state-Prot-<$<!isideal!>$>-SENTCTXT>  pstate-prot-prot))
        (var-prot-sk  (<pkg-state-Prot-<$<!isideal!>$>-sk>  pstate-prot-prot))
        (var-prot-pk  (<pkg-state-Prot-<$<!isideal!>$>-pk>  pstate-prot-prot))

        (var-h1-ctr (<pkg-state-Corr_reduction-<$<!isideal!>$>-ctr> pstate-h1-corr_red))
        (var-h1-TESTED (<pkg-state-Corr_reduction-<$<!isideal!>$>-TESTED> pstate-h1-corr_red))
        (var-h1-RECEIVEDKEY (<pkg-state-Corr_reduction-<$<!isideal!>$>-RECEIVEDKEY> pstate-h1-corr_red))
        (var-h1-RECEIVEDCTXT (<pkg-state-Corr_reduction-<$<!isideal!>$>-RECEIVEDCTXT> pstate-h1-corr_red))
        (var-h1-SENTKEY (<pkg-state-Corr_reduction-<$<!isideal!>$>-SENTKEY> pstate-h1-corr_red))
        (var-h1-SENTCTXT (<pkg-state-Corr_reduction-<$<!isideal!>$>-SENTCTXT> pstate-h1-corr_red))
        (var-h1-sk (<pkg-state-Corr_KEM-<$<!isideal!>$>-sk> pstate-h1-corr_kem))
        (var-h1-pk (<pkg-state-Corr_KEM-<$<!isideal!>$>-pk> pstate-h1-corr_kem)))

        ; state variables are equal
        (and (= var-prot-ctr var-h1-ctr)
             (= var-prot-TESTED var-h1-TESTED)
             (= var-prot-RECEIVEDKEY var-h1-RECEIVEDKEY)
             (= var-prot-RECEIVEDCTXT var-h1-RECEIVEDCTXT)
             (= var-prot-SENTKEY var-h1-SENTKEY)
             (= var-prot-SENTCTXT var-h1-SENTCTXT)
             (= var-prot-sk var-h1-sk)
             (= var-prot-pk var-h1-pk)))))
