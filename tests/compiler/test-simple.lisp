; Very simple test to understand the issue

(defun test-pair (() (cons 1 2)))

(print "Test: test-pair")
(let ((p (test-pair)))
  (print (car p)))
