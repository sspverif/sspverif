(define-fun no-overwriting-state
    ((max-ctr Int)
     (State (Array Int (Maybe (Tuple10 Int Bool Int Int (Maybe Bool)
                                       (Maybe Bits_256) (Maybe Bits_256) (Maybe Bits_256)
                                       (Maybe (Tuple5 Int Int Bits_256 Bits_256 Bits_256)) Int)))))
  Bool
  (forall ((ctr Int))
          (=> (> ctr max-ctr)
              (is-mk-none (select State ctr)))))


(define-fun referenced-kid-exist
    ((State (Array Int (Maybe (Tuple10 Int Bool Int Int (Maybe Bool)
                                       (Maybe Bits_256) (Maybe Bits_256) (Maybe Bits_256)
                                       (Maybe (Tuple5 Int Int Bits_256 Bits_256 Bits_256)) Int))))
     (Ltk (Array Int (Maybe Bits_256))))
  Bool
  (forall ((ctr Int))
          (let ((state (select State ctr)))
                 (=> (not (is-mk-none state))
                  (let  ((U    (el10-1  (maybe-get state)))
                         (u    (el10-2  (maybe-get state)))
                         (V    (el10-3  (maybe-get state)))
                         (kid  (el10-4  (maybe-get state)))
                         (acc  (el10-5  (maybe-get state)))
                         (ni   (el10-6  (maybe-get state)))
                         (nr   (el10-7  (maybe-get state)))
                         (kmac (el10-8  (maybe-get state)))
                         (sid  (el10-9  (maybe-get state)))
                         (mess (el10-10 (maybe-get state))))
                    (not (is-mk-none (select Ltk kid))))))))

(define-fun ltk-and-h-set-together
    ((Ltk (Array Int (Maybe Bits_256)))
     (H (Array Int (Maybe Bool))))
  Bool
  (forall ((kid Int))
          (= (is-mk-none (select Ltk kid))
             (is-mk-none (select H kid)))))



(define-fun invariant
    ((left-game <GameState_KX_NoPrf_<$<!n!>$>>)
     (right-game <GameState_KX_NoPrfB_<$<!n!>$>>))
  Bool
  (let ((left-prf-pkg (<game-KX_NoPrf-<$<!n!>$>-pkgstate-Prf> left-game))
        (left-game-pkg (<game-KX_NoPrf-<$<!n!>$>-pkgstate-Game> left-game))
        (right-prf-pkg (<game-KX_NoPrfB-<$<!n!>$>-pkgstate-Prf> right-game))
        (right-game-pkg (<game-KX_NoPrfB-<$<!n!>$>-pkgstate-Game> right-game)))
    (let ((left-LTK (<pkg-state-PRF-<$<!n!>$>-LTK> left-prf-pkg))
          (left-PRF (<pkg-state-PRF-<$<!n!>$>-PRF> left-prf-pkg))
          (left-State (<pkg-state-Game_NoPrf-<$<!n!>$>-State> left-game-pkg))
          (left-Fresh (<pkg-state-Game_NoPrf-<$<!n!>$>-Fresh> left-game-pkg))
          (left-RevTested (<pkg-state-Game_NoPrf-<$<!n!>$>-RevTested> left-game-pkg))
          (left-H (<pkg-state-PRF-<$<!n!>$>-H> left-prf-pkg))
          (left-kid (<pkg-state-PRF-<$<!n!>$>-kid_> left-prf-pkg))
          (left-ctr (<pkg-state-Game_NoPrf-<$<!n!>$>-ctr_> left-game-pkg))
          ;;
          (right-LTK (<pkg-state-PRFB-<$<!n!>$>-LTK> right-prf-pkg))
          (right-PRF (<pkg-state-PRFB-<$<!n!>$>-PRF> right-prf-pkg))
          (right-State (<pkg-state-Game_NoPrfB-<$<!n!>$>-State> right-game-pkg))
          (right-Fresh (<pkg-state-Game_NoPrfB-<$<!n!>$>-Fresh> right-game-pkg))
          (right-RevTested (<pkg-state-Game_NoPrfB-<$<!n!>$>-RevTested> right-game-pkg))
          (right-H (<pkg-state-PRFB-<$<!n!>$>-H> right-prf-pkg))
          (right-kid (<pkg-state-PRFB-<$<!n!>$>-kid_> right-prf-pkg))
          (right-ctr (<pkg-state-Game_NoPrfB-<$<!n!>$>-ctr_> right-game-pkg)))
      (and
       ;; (= left-game right-game)
       (= left-kid right-kid)
       (= left-ctr right-ctr)
       (= left-LTK right-LTK)
       (= left-PRF right-PRF)
       (= left-H right-H)
       (= left-Fresh right-Fresh)
       (= left-RevTested right-RevTested)
       (= left-State right-State)

       (no-overwriting-state left-ctr left-State)
       
       (referenced-kid-exist left-State left-LTK)
       (ltk-and-h-set-together left-LTK left-H)
       
       ))))
