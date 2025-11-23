(defmacro when (cond body)
  `(if ,cond ,body false))

(defmacro unless (cond body)
  `(if ,cond false ,body))

(print (when true (+ 1 2)))
(print (when false (+ 1 2)))
(print (when (> 10 5) (* 2 3)))

(print (unless false (+ 10 20)))
(print (unless true (+ 10 20)))
(print (unless (< 10 5) (* 4 5)))

(defun test-control (x)
  (when (> x 0)
    (unless (> x 100)
      x)))

(print (test-control 50))
(print (test-control -5))
(print (test-control 200))
