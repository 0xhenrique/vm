;; Math library for testing load functionality

(defun square (x)
  (* x x))

(defun cube (x)
  (* x (* x x)))

(defun sum-of-squares (a b)
  (+ (square a) (square b)))
