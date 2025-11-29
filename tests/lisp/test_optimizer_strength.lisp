;; Test strength reduction optimizations
;; EXPECT: -42

;; Strength reduction replaces expensive operations with cheaper equivalents

;; x * -1 → -x (multiplication becomes negation)
(defun negate-by-mul (x)
  (* x -1))

;; Also works with floats
(defun negate-by-mul-float (x)
  (* x -1.0))

;; x * 0 → 0 (eliminate multiplication entirely)
(defun mul-by-zero (x)
  (* x 0))

(defun mul-by-zero-float (x)
  (* x 0.0))

;; Practical example: negating with multiplication
(defun absolute-value-inefficient (x)
  (if (< x 0)
      (* x -1)  ; This gets optimized to negation
      x))

;; Multiple strength reductions in one function
(defun strength-test (x y)
  (+ (* x -1)  ; Becomes: -x
     (* y 0))) ; Becomes: 0

;; Test it
(negate-by-mul 42)
