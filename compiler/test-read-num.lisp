; Test just the read-num-acc function

(defun is-digit? (c)
  (let ((n (char-code c)))
    (and (>= n 48) (<= n 57))))

(defun read-num-acc (chars acc)
  (if (== chars '())
    (cons (list->string acc) (cons '() '()))
    (if (is-digit? (car chars))
      (read-num-acc (cdr chars) (append acc (cons (car chars) '())))
      (cons (list->string acc) (cons chars '())))))

(print "Test read-num-acc:")
(print (read-num-acc (string->list "123abc") '()))
