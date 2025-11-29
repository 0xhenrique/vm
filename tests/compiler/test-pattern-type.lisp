; Test: is 0 a symbol?

(print "Is 0 a symbol?")
(print (symbol? 0))

(print "Is 'n a symbol?")
(print (symbol? 'n))

(print "Is n a symbol (without quote)?")
(defconst n 5)
(print (symbol? n))

; Now test with actual patterns
(defconst clause1 '((0) true))
(defconst clause2 '((n) false))

(defun get-clause-patterns (((patterns body)) patterns))
(defun get-first-pattern ((patterns) (car patterns)))

(print "")
(print "First clause patterns:")
(print (get-clause-patterns clause1))

(print "First pattern from clause1:")
(print (get-first-pattern (get-clause-patterns clause1)))

(print "Is it a symbol?")
(print (symbol? (get-first-pattern (get-clause-patterns clause1))))

(print "")
(print "First pattern from clause2:")
(print (get-first-pattern (get-clause-patterns clause2)))

(print "Is it a symbol?")
(print (symbol? (get-first-pattern (get-clause-patterns clause2))))
