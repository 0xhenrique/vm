;; Test peephole optimizations with algebraic identities
;; EXPECT: 42

;; These functions demonstrate algebraic simplifications that
;; the peephole optimizer can eliminate at compile time.

;; Identity: x + 0 = x
(defun add-zero (x)
  (+ x 0))

;; Identity: x - 0 = x
(defun sub-zero (x)
  (- x 0))

;; Identity: x * 1 = x
(defun mul-one (x)
  (* x 1))

;; Identity: x / 1 = x
(defun div-one (x)
  (/ x 1))

;; Double negation: -(-(x)) = x
(defun double-neg (x)
  (- 0 (- 0 x)))

;; Multiple identities in one function
(defun multiple-identities (x)
  (* (+ (- x 0) 0) 1))

;; Float identities also work
(defun float-identities (x)
  (/ (* (+ x 0.0) 1.0) 1.0))

;; Complex expression with peephole opportunities
(defun complex-peephole (x y)
  (+ (* (+ x 0) 1)
     (- (* y 1) 0)))

;; Test with actual computation
(+ (add-zero 40) (sub-zero 2))
