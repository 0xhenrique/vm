; Minimal tokenizer test

(defun tok-simple (chars acc)
  (if (== chars '())
    acc
    (if (== (car chars) "(")
      (tok-simple (cdr chars) (append acc (cons "LPAREN" '())))
      (if (== (car chars) ")")
        (tok-simple (cdr chars) (append acc (cons "RPAREN" '())))
        (tok-simple (cdr chars) (append acc (cons "OTHER" '())))))))

(print (tok-simple (string->list "(+)") '()))
