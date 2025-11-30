; Test when and unless

(print "Testing when:")

(defun test-when (x)
  (when (> x 0)
    "positive"))

(print (test-when 5))
(print (test-when -3))
(print (test-when 0))

(print "")
(print "Testing unless:")

(defun test-unless (x)
  (unless (> x 0)
    "not positive"))

(print (test-unless 5))
(print (test-unless -3))
(print (test-unless 0))

(print "")
(print "Testing when with do block:")

(defun when-with-side-effect (x)
  (when (> x 0)
    (do
      (print "It's positive!")
      42)))

(print (when-with-side-effect 10))
(print (when-with-side-effect -5))
