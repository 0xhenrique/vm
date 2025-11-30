; Test partial application

(print "Testing partial:")

(defun add (a b) (+ a b))
(defun mul (a b) (* a b))

; Use let to bind the partial functions and call them directly
(print (let ((add5 (partial add 5)))
         (add5 3)))

(print (let ((add5 (partial add 5)))
         (add5 7)))

(print (let ((mul10 (partial mul 10)))
         (mul10 2)))

(print (let ((mul10 (partial mul 10)))
         (mul10 5)))

(print "")
(print "Testing compose with partial:")

(print (let ((add5 (partial add 5))
             (double (partial mul 2)))
         (let ((double-then-add5 (compose add5 double)))
           (double-then-add5 10))))

(print "")
(print "Testing map with partial:")

(print (map (partial add 5) '(1 2 3 4 5)))

(print "")
(print "Testing inline partial application:")
(print ((partial add 100) 23))

(print "")
(print "Done!")
