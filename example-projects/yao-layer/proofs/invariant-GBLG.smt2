(define-fun randomness-mapping-GETAOUT
  ((base-ctr-0 Int)
   (base-ctr-1 Int)
   (id-0  Int)     
   (id-1  Int)     
   (scr-0 Int)     
   (scr-1 Int))    
  Bool
  (and
   ;;(= base-ctr-0 scr-0)
   ;;(= base-ctr-1 scr-1)
   (= id-0 id-1)))

(define-fun randomness-mapping-SETBIT
  ((base-ctr-0 Int)
   (base-ctr-1 Int)
   (id-0  Int)     
   (id-1  Int)     
   (scr-0 Int)     
   (scr-1 Int))    
  Bool
  false)

(define-fun randomness-mapping-GBLG
  ((base-ctr-0 Int)
   (base-ctr-1 Int)
   (id-0  Int)     
   (id-1  Int)     
   (scr-0 Int)     
   (scr-1 Int))    
  Bool
  false)



(define-fun invariant
    ((left-game <GameState_Left_<$<!n!><!m!><!p!>$>>)
     (right-game <GameState_Right_<$<!n!><!m!><!p!>$>>))
  Bool
  (let ((KeyTopLeft (<game-Left-<$<!n!><!m!><!p!>$>-pkgstate-keys_top> left-game))
        (KeyTopRight  (<game-Right-<$<!n!><!m!><!p!>$>-pkgstate-keys_top> right-game))
        (KeyBottomLeft  (<game-Left-<$<!n!><!m!><!p!>$>-pkgstate-keys_bottom> left-game))
        (KeyBottomRight  (<game-Right-<$<!n!><!m!><!p!>$>-pkgstate-keys_bottom> right-game)))
    (and
     (= KeyTopLeft KeyTopRight)
     (= KeyBottomLeft KeyBottomRight)

    )))
