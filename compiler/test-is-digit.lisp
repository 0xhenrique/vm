; Test is-digit?

(defun is-digit? (c)
  (let ((n (char-code c)))
    (and (>= n 48) (<= n 57))))

(print "Testing is-digit?:")
(print (is-digit? "1"))
(print (is-digit? "a"))
(print (is-digit? "("))

(print "With string->list:")
(print (is-digit? (car (string->list "123"))))
