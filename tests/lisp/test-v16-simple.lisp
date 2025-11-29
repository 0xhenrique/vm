; Simple test using only comparison operators (no and/or/cond)
; This can be compiled with the Rust compiler

(print "Testing v16 comparison operators")
(print "")

(print (< 3 5))
(print (< 5 3))
(print (> 5 3))
(print (> 3 5))
(print (<= 3 5))
(print (<= 5 5))
(print (<= 5 3))
(print (>= 5 3))
(print (>= 5 5))
(print (>= 3 5))
(print (!= 3 5))
(print (!= 5 5))
(print "")

(defun max-v16 (a b)
  (if (> a b) a b))

(print (max-v16 10 5))
(print (max-v16 3 8))
(print "")

(print "Done!")
