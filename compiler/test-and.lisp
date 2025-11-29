; Test and operator

(print "Testing and:")
(print (and true true))
(print (and true false))
(print (and (> 5 3) (< 2 10)))

(defun test-and (n)
  (and (>= n 48) (<= n 57)))

(print (test-and 50))
(print (test-and 30))
