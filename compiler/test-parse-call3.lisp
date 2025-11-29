; Test what quoted list evaluates to

(print "Test 1:")
(print '("42"))

(print "Test 2: car")
(print (car '("42")))

(print "Test 3: string->list on it")
(defun test-string (x)
  (string->list x))

(print (test-string (car '("42"))))
