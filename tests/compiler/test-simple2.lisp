; Very simple test without let

(defun test-pair (() (cons 1 2)))

(print "Test: test-pair")
(print (car (test-pair)))
(print "Done")
