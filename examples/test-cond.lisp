; Test cond to see if it works properly

(print "Testing cond:")

(defun test-cond (x)
  (cond
    ((< x 0) "negative")
    ((== x 0) "zero")
    ((> x 0) "positive")
    (else "unknown")))

(print (test-cond -5))
(print (test-cond 0))
(print (test-cond 5))

(print "")
(print "Testing nested cond:")

(defun classify (x y)
  (cond
    ((< x 0) (cond
               ((< y 0) "both negative")
               (else "x negative")))
    ((== x 0) "x is zero")
    (else "x positive")))

(print (classify -1 -1))
(print (classify -1 1))
(print (classify 0 5))
(print (classify 5 5))
