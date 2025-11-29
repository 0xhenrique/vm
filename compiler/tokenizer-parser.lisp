; Integrated tokenizer + parser

; === TOKENIZER ===

(defun is-digit? (c)
  (let ((n (char-code c)))
    (and (>= n 48) (<= n 57))))

(defun is-space? (c)
  (== (char-code c) 32))

(defun is-letter? (c)
  (let ((n (char-code c)))
    (or (and (>= n 65) (<= n 90))
        (and (>= n 97) (<= n 122)))))

(defun is-symbol-char? (c)
  (let ((n (char-code c)))
    (or (is-letter? c)
        (is-digit? c)
        (== n 43) (== n 45) (== n 42) (== n 47)
        (== n 60) (== n 62) (== n 61)
        (== n 33) (== n 63) (== n 95))))

(defun read-num-acc (chars acc)
  (if (== chars '())
    (cons (list->string acc) (cons '() '()))
    (if (is-digit? (car chars))
      (read-num-acc (cdr chars) (append acc (cons (car chars) '())))
      (cons (list->string acc) (cons chars '())))))

(defun read-symbol-acc (chars acc)
  (if (== chars '())
    (cons (list->string acc) (cons '() '()))
    (if (is-symbol-char? (car chars))
      (read-symbol-acc (cdr chars) (append acc (cons (car chars) '())))
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
            (if (is-symbol-char? (car chars))
              (let ((res (read-symbol-acc chars '())))
                (tokenize-loop (car (cdr res)) (append acc (cons (car res) '()))))
              (tokenize-loop (cdr chars) (append acc (cons "?" '()))))))))))

(defun tokenize (str)
  (tokenize-loop (string->list str) '()))

; === PARSER ===

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

(defun parse-atom (token)
  (if (is-number-string? token)
    (cons (cons "number" (cons token '())) '())
    (cons (cons "symbol" (cons token '())) '())))

(defun parse-list-items (tokens acc)
  (if (== tokens '())
    (cons acc (cons '() '()))
    (if (== (car tokens) ")")
      (cons acc (cons (cdr tokens) '()))
      (let ((parsed (parse-expr tokens)))
        (let ((expr (car parsed)))
          (let ((rest (car (cdr parsed))))
            (parse-list-items rest (append acc (cons expr '())))))))))

(defun parse-expr (tokens)
  (if (== tokens '())
    (cons '() (cons '() '()))
    (if (== (car tokens) "(")
      (let ((result (parse-list-items (cdr tokens) '())))
        (let ((items (car result)))
          (let ((rest (car (cdr result))))
            (cons (cons "list" items) (cons rest '())))))
      (let ((atom-result (parse-atom (car tokens))))
        (let ((expr (car atom-result)))
          (cons expr (cons (cdr tokens) '())))))))

(defun parse (tokens)
  (car (parse-expr tokens)))

; === TESTS ===

(print "=== Tokenizer + Parser ===")
(print "Test 1: (+ 1 2)")
(print (parse (tokenize "(+ 1 2)")))

(print "Test 2: (defun foo (x) (* x 2))")
(print (parse (tokenize "(defun foo (x) (* x 2))")))
