; Test what read-num-acc returns

(defun is-digit? (c)
  (let ((n (char-code c)))
    (and (>= n 48) (<= n 57))))

(defun read-num-acc (chars acc)
  (if (== chars '())
    (cons (list->string acc) (cons '() '()))
    (if (is-digit? (car chars))
      (read-num-acc (cdr chars) (append acc (cons (car chars) '())))
      (cons (list->string acc) (cons chars '())))))

(print "Test 1: read '1abc'")
(let ((res (read-num-acc (string->list "1abc") '())))
  (print res)
  (print "First element (number):")
  (print (car res))
  (print "Second element (cdr res):")
  (print (cdr res))
  (print "Remaining chars (car (cdr res)):")
  (print (car (cdr res))))
