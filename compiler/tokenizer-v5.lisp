; Working tokenizer using new primitives

; Character tests
(defun is-digit? (c)
  (let ((n (char-code c)))
    (and (>= n 48) (<= n 57))))

(defun is-alpha? (c)
  (let ((n (char-code c)))
    (or (and (>= n 65) (<= n 90))
        (and (>= n 97) (<= n 122)))))

(defun is-space? (c)
  (== (char-code c) 32))

; Read number from chars, returns (string . remaining-chars)
(defun read-num-loop (chars acc)
  (if (== chars '())
    (cons (list->string acc) (cons '() '()))
    (if (is-digit? (car chars))
      (read-num-loop (cdr chars) (append acc (cons (car chars) '())))
      (cons (list->string acc) (cons chars '())))))

(defun read-number (chars)
  (read-num-loop chars '()))

; Read symbol from chars
(defun read-sym-loop (chars acc)
  (if (== chars '())
    (cons (list->string acc) (cons '() '()))
    (if (is-alpha? (car chars))
      (read-sym-loop (cdr chars) (append acc (cons (car chars) '())))
      (cons (list->string acc) (cons chars '())))))

(defun read-symbol (chars)
  (read-sym-loop chars '()))

; Main tokenizer
(defun tokenize (input)
  (tok-loop (string->list input) '()))

(defun tok-loop (chars tokens)
  (if (== chars '())
    tokens
    (let ((ch (car chars)))
      (cond
        ((is-space? ch)
         (tok-loop (cdr chars) tokens))
        ((== ch "(")
         (tok-loop (cdr chars) (append tokens (cons "(" '()))))
        ((== ch ")")
         (tok-loop (cdr chars) (append tokens (cons ")" '()))))
        ((is-digit? ch)
         (let ((result (read-number chars)))
           (tok-loop (car (cdr result)) (append tokens (cons (car result) '())))))
        ((is-alpha? ch)
         (let ((result (read-symbol chars)))
           (tok-loop (car (cdr result)) (append tokens (cons (car result) '())))))
        (else
         (tok-loop (cdr chars) tokens))))))

; Tests
(print "=== Tokenizer Tests ===")
(print "Test 1:")
(print (tokenize "(+ 1 2)"))
(print "Test 2:")
(print (tokenize "(foo bar 123)"))
(print "Test 3:")
(print (tokenize "(a (b c))"))
