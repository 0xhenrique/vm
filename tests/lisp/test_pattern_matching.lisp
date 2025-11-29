;; EXPECT: 42
(defun fact
  ((0) 1)
  ((n) (* n (fact (- n 1)))))

(print (fact 0))
(print (fact 1))
(print (fact 5))

(defun is_zero
  ((0) true)
  ((n) false))

(print (is_zero 0))
(print (is_zero 5))

(defun add_if_first_zero
  ((0 y) y)
  ((x y) (+ x y)))

(print (add_if_first_zero 0 10))
(print (add_if_first_zero 5 10))

(defun always_42
  ((_) 42))

(print (always_42 0))
(print (always_42 100))
