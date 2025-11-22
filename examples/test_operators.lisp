(print (> 10 5))
(print (> 5 10))
(print (< 5 10))
(print (< 10 5))
(print (>= 10 10))
(print (>= 10 5))
(print (>= 5 10))
(print (== 10 10))
(print (== 10 5))
(print (!= 10 5))
(print (!= 10 10))

(print (% 10 3))
(print (% 15 4))
(print (% 20 5))

(print (neg 5))
(print (neg (neg 3)))
(print (+ 10 (neg 5)))

(print (if (> 10 5) 100 200))
(print (if (< 10 5) 100 200))
(print (if (== 5 5) 111 222))
(print (if (!= 5 3) 333 444))

(defun is_even (n)
  (== (% n 2) 0))

(print (is_even 4))
(print (is_even 7))

(defun abs (n)
  (if (< n 0) (neg n) n))

(print (abs (neg 10)))
(print (abs 15))
