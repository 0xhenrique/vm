; Test exact tokenizer with same input

(defun is-digit? (c)
  (let ((n (char-code c)))
    (and (>= n 48) (<= n 57))))

(defun is-space? (c)
  (== (char-code c) 32))

(defun read-num-acc (chars acc)
  (if (== chars '())
    (cons (list->string acc) (cons '() '()))
    (if (is-digit? (car chars))
      (read-num-acc (cdr chars) (append acc (cons (car chars) '())))
      (cons (list->string acc) (cons chars '())))))

(defun tokenize-loop (chars acc)
  (if (== chars '())
    acc
    (if (is-space? (car chars))
      (tokenize-loop (cdr chars) acc)
      (if (== (car chars) "(")
        (tokenize-loop (cdr chars) (append acc (cons "(" '())))
        (if (== (car chars) ")")
          (tokenize-loop (cdr chars) (append acc (cons ")" '())))
          (if (is-digit? (car chars))
            (let ((res (read-num-acc chars '())))
              (tokenize-loop (car (cdr res)) (append acc (cons (car res) '()))))
            (tokenize-loop (cdr chars) (append acc (cons "?" '())))))))))

(defun tokenize (str)
  (tokenize-loop (string->list str) '()))

(print "Test: (+ 1 2)")
(print (tokenize "(+ 1 2)"))
