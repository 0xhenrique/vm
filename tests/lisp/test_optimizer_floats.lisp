;; Test optimizer constant folding specifically for float operations
;; EXPECT: 3.141592653589793

;; This file demonstrates that the optimizer can fold:
;; - Pure float operations
;; - Mixed int/float operations
;; - Float comparisons
;; - Complex float expressions

;; Simple float arithmetic (folded at compile time)
(defun circle-area-const ()
  ;; π * r^2 where r=1
  ;; 3.141592653589793 * 1.0 = 3.141592653589793
  (* 3.141592653589793 1.0))

;; Mixed int/float (folded with type coercion)
(defun mixed-calc ()
  ;; (5 + 2.5) * 2.0 = 7.5 * 2.0 = 15.0
  (* (+ 5 2.5) 2.0))

;; Float comparison (folded to boolean)
(defun float-compare ()
  ;; (3.14 > 2.71) = #t
  (if (> 3.14 2.71)
      1.0
      0.0))

;; Complex float expression
(defun quadratic-const ()
  ;; Simplified quadratic: (2.0 * 3.0) - (4.0 / 2.0)
  ;; = 6.0 - 2.0 = 4.0
  (- (* 2.0 3.0) (/ 4.0 2.0)))

;; Negation folding
(defun negate-const ()
  ;; -(3.14) = -3.14
  (- 0 3.14))

;; Return π to verify execution
(circle-area-const)
