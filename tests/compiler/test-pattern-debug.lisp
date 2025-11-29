; Debug test for pattern matching

; Test clause extraction
(defun get-clause-pattern (((pattern body)) pattern))
(defun get-clause-body (((pattern body)) body))

; Test with simple clause
(print "Pattern from ((0) true):")
(print (get-clause-pattern '((0) true)))

(print "Body from ((0) true):")
(print (get-clause-body '((0) true)))

; Test with list of clauses
(def test-clauses '(((0) true) ((n) false)))

(print "All clauses:")
(print test-clauses)

(print "First clause:")
(print (car test-clauses))

(print "Rest clauses:")
(print (cdr test-clauses))

(print "Pattern from first:")
(print (get-clause-pattern (car test-clauses)))

(print "Body from first:")
(print (get-clause-body (car test-clauses)))
