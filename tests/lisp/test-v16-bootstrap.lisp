; Test that v16 can compile a simple Lisp program
; This tests the new operators without requiring the Rust compiler to support them

(print "Testing v16 bootstrap capability")
(print "")

; Test factorial using pattern matching (from v15)
(defun fact ((0) 1) ((n) (* n (fact (- n 1)))))

(print "Factorial:")
(print (fact 0))
(print (fact 1))
(print (fact 5))
(print "")

; Test simple comparison operators
(defun max-simple (a b)
  (if (> a b) a b))

(print "Max function using >:")
(print (max-simple 10 5))
(print (max-simple 3 8))
(print "")

; Test nested comparisons
(defun clamp (x low high)
  (if (< x low)
    low
    (if (> x high)
      high
      x)))

(print "Clamp function:")
(print (clamp 5 0 10))
(print (clamp -5 0 10))
(print (clamp 15 0 10))
(print "")

(print "Bootstrap test complete!")
