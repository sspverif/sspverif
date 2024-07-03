(declare-datatype Interval ((mk-interval (interval-start Int) (interval-end Int))))
(declare-datatype Group    ((mk-group    (group-src Int)
                                         (group-name String)
                                         (group-interval Interval))))
(declare-datatype Groups   ((groups-nil)
                            (groups-cons (groups-tip Group) (groups-tail Groups))))

(define-fun interval-contains ((interval Interval) (x Int)) Bool
  (and (or (= (interval-start interval) x)
           (< (interval-start interval) x))
           (< x (interval-end interval))))

(define-fun wire_groups () Groups
  (groups-cons (mk-group 0 "foo" (mk-interval 0 10))
               groups-nil)) ; List

(define-fun import_ranges () Groups
  (groups-cons (mk-group 0 "foo" (mk-interval 0 10))
               groups-nil)) ; List

(define-fun-rec f ((groups Groups) (source Int) (name String) (x Int) (count Int)) Int
  (match groups (
    ((groups-nil) count)
    ((groups-cons group groups)
      (let ((src-matches (= (group-src group) source))
            (name-matches (= (group-name group) name))
            (range-matches (interval-contains (group-interval group) x)))
           (ite (and src-matches name-matches range-matches)
                (f groups source name x (+ 1 count))
                (f groups source name x count)))))))

(define-fun-rec groups-contains ((groups Groups) (needle Group)) Bool
  (match groups (
    ((groups-nil) false)
    ((groups-cons group groups)
      (let ((eq (= group needle)))
           (ite eq 
                true
                (groups-contains groups needle)))))))

(declare-const group Group)
(declare-const x Int)
(declare-const should-be-one Bool)
(declare-const is-one Bool)

(assert (= is-one (= 1 (f wire_groups (group-src group) (group-name group) x 0))))
(assert (= should-be-one
           (and (groups-contains import_ranges group))
                (interval-contains (group-interval group) x)))

(assert (not (=> should-be-one is-one)))

(check-sat)
