; Simple test of is-number-string?

(defun all-digits? (chars)
  (if (== chars '())
    true
    (let ((c (car chars)))
      (let ((n (char-code c)))
        (if (and (>= n 48) (<= n 57))
          (all-digits? (cdr chars))
          false)))))

(defun is-number-string? (str)
  (if (== str "")
    false
    (all-digits? (string->list str))))

(print "Testing is-number-string?:")
(print (is-number-string? "42"))
(print (is-number-string? "foo"))
(print (is-number-string? ""))

(print "Testing parse-atom:")
(defun parse-atom (token)
  (if (is-number-string? token)
    (cons (cons "number" (cons token '())) '())
    (cons (cons "symbol" (cons token '())) '())))

(print (parse-atom "42"))
(print (parse-atom "foo"))
