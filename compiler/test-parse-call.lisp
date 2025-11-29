; Test calling parse with quoted list

(defun all-digits? (chars)
  (if (== chars '())
    true
    (let ((c (car chars)))
      (let ((n (char-code c)))
        (if (and (>= n 48) (<= n 57))
          (all-digits? (cdr chars))
          false)))))

(defun is-number-string? (str)
  (print "is-number-string? called with:")
  (print str)
  (if (== str "")
    false
    (all-digits? (string->list str))))

(defun parse-atom (token)
  (print "parse-atom called with:")
  (print token)
  (if (is-number-string? token)
    (cons (cons "number" (cons token '())) '())
    (cons (cons "symbol" (cons token '())) '())))

(defun parse-expr (tokens)
  (print "parse-expr called with:")
  (print tokens)
  (if (== tokens '())
    (cons '() (cons '() '()))
    (let ((atom-result (parse-atom (car tokens))))
      (let ((expr (car atom-result)))
        (cons expr (cons (cdr tokens) '()))))))

(defun parse (tokens)
  (car (parse-expr tokens)))

(print "Calling parse with '(\"42\")")
(print (parse '("42")))
