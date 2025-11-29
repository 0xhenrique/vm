; Trace tokenizer with limited recursion

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

(defun tokenize-loop (chars acc depth)
  (if (== depth 0)
    acc
    (if (== chars '())
      acc
      (if (is-space? (car chars))
        (tokenize-loop (cdr chars) acc (- depth 1))
        (if (== (car chars) "(")
          (tokenize-loop (cdr chars) (append acc (cons "(" '())) (- depth 1))
          (if (== (car chars) ")")
            (tokenize-loop (cdr chars) (append acc (cons ")" '())) (- depth 1))
            (if (is-digit? (car chars))
              (let ((res (read-num-acc chars '())))
                (tokenize-loop (car (cdr res)) (append acc (cons (car res) '())) (- depth 1)))
              (tokenize-loop (cdr chars) (append acc (cons "?" '())) (- depth 1)))))))))

(print "Test with depth limit 20:")
(print (tokenize-loop (string->list "(+ 1 2)") '() 20))
