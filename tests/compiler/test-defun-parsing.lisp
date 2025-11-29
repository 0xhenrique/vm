; Test: how defun-form parses clauses

(defconst test-defun '(defun is-zero ((0) true) ((n) false)))

(print "Full defun:")
(print test-defun)

(print "")
(print "Car (should be 'defun'):")
(print (car test-defun))

(print "Cadr (should be name 'is-zero'):")
(print (car (cdr test-defun)))

(print "Cddr (should be clauses):")
(print (cdr (cdr test-defun)))

(print "")
(print "Extracting with pattern:")

(defun test-extract
  (((defun-kw name . clauses))
    (cons name clauses)))

(print (test-extract test-defun))

(print "")
(print "Expected: (is-zero ((0) true) ((n) false))")
