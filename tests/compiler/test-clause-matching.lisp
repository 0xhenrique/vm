; Test: check if clauses pattern matching works

(def test-clauses '(((0) true) ((n) false)))

(print "Test clauses:")
(print test-clauses)

(print "")
(print "Testing pattern: ((clause))")
(print "Does '(((0) true))' match?")
; This should match if clauses has exactly one element

(print "Testing pattern: ((clause . rest))")
(print "clauses = (((0) true) ((n) false))")
(print "clause should be: ((0) true)")
(print "rest should be: (((n) false))")

; Simulate the pattern matching
(defun test-match-single
  (((clause)) (cons "SINGLE" clause)))

(defun test-match-multiple
  (((clause . rest)) (cons "MULTIPLE" (cons clause rest))))

(print "")
(print "Result of matching:")
(print (test-match-multiple test-clauses))
