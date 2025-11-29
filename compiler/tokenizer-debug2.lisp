; Debug tokenizer step by step

(defun is-digit? (c)
  (let ((n (char-code c)))
    (>= n 48)))

(defun tok1 (chars)
  (if (== chars '())
    '()
    (if (is-digit? (car chars))
      (cons "DIGIT" (tok1 (cdr chars)))
      (cons "OTHER" (tok1 (cdr chars))))))

(print (tok1 (string->list "1+2")))
