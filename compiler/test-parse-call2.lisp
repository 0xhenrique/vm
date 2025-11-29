; Test what '("42") evaluates to

(print "Test 1: what is '(\"42\")?")
(print '("42"))

(print "Test 2: car of '(\"42\")")
(print (car '("42")))

(print "Test 3: is it a string?")
(defun test-string (x)
  (string->list x))

(print (test-string (car '("42"))))
