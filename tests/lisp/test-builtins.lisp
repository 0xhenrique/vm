; Test which built-ins are callable from Lisp code
(print "Testing built-ins...")

; Test cons (should work - it's a VM instruction)
(print (cons 1 (cons 2 '())))

; Test car (should work)
(print (car (cons 1 (cons 2 '()))))

; Test cdr (should work)
(print (cdr (cons 1 (cons 2 '()))))

; Test list-length (should work)
(print (list-length (cons 1 (cons 2 '()))))

(print "All tests passed!")
