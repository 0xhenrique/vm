; Test if 'let' causes issues

(defun is-digit-simple (c)
  (>= (char-code c) 48))

(defun is-digit-with-let (c)
  (let ((n (char-code c)))
    (>= n 48)))

(print "Test is-digit-simple:")
(print (is-digit-simple "5"))

(print "Test is-digit-with-let:")
(print (is-digit-with-let "5"))

(print "Test in recursion - simple:")
(defun tok1 (chars)
  (if (== chars '())
    '()
    (if (is-digit-simple (car chars))
      (cons "D" (tok1 (cdr chars)))
      (cons "X" (tok1 (cdr chars))))))

(print (tok1 (string->list "123")))

(print "Test in recursion - with let:")
(defun tok2 (chars)
  (if (== chars '())
    '()
    (if (is-digit-with-let (car chars))
      (cons "D" (tok2 (cdr chars)))
      (cons "X" (tok2 (cdr chars))))))

(print (tok2 (string->list "456")))
