; Simplified test to isolate the issue

(defun is-digit? (c)
  (let ((n (char-code c)))
    (and (>= n 48) (<= n 57))))

(defun tok-simple (chars)
  (if (== chars '())
    '()
    (if (is-digit? (car chars))
      (cons "D" (tok-simple (cdr chars)))
      (cons "X" (tok-simple (cdr chars))))))

(print "Test 1: simple recursion")
(print (tok-simple (string->list "123")))

(defun tok-with-let (chars)
  (if (== chars '())
    '()
    (let ((res (cons (car chars) '())))
      (if (is-digit? (car chars))
        (cons "D" (tok-with-let (cdr chars)))
        (cons "X" (tok-with-let (cdr chars)))))))

(print "Test 2: with let")
(print (tok-with-let (string->list "123")))
