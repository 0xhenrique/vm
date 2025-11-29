; Test: is-old-style-params detection

(defun is-old-style-params
  ((clause)
    (if (list? clause)
      (if (list? (car clause))
        false  ; New style: ((pattern) ...)
        true)  ; Old style: (x y z)
      false)))

(print "Test 1: Old-style (x y z)")
(print (is-old-style-params '(x y z)))
(print "Expected: true")

(print "")
(print "Test 2: New-style ((0) true)")
(print (is-old-style-params '((0) true)))
(print "Expected: false")

(print "")
(print "Test 3: New-style ((n) false)")
(print (is-old-style-params '((n) false)))
(print "Expected: false")
