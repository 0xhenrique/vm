; Minimal test

(defun is-digit? (c)
  (>= (char-code c) 48))

(defun tok (chars)
  (if (== chars '())
    '()
    (if (is-digit? (car chars))
      (cons "D" (tok (cdr chars)))
      (cons "X" (tok (cdr chars))))))

(print (tok (string->list "1+2")))
