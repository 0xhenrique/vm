(defun double (x)
  (* x 2))

(defun add (a b)
  (+ a b))

(defun fib (n)
  (if (<= n 1)
    n
    (+ (fib (- n 1)) (fib (- n 2)))))

(print (double 5))
(print (add 3 7))
(print (fib 0))
(print (fib 1))
(print (fib 5))
(print (fib 8))
