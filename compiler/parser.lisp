; Parser - converts tokens to s-expressions
;
; Representation:
; - Numbers: (number "42")
; - Symbols: (symbol "foo")
; - Lists: (list expr1 expr2 ...)

; Helper: check if all chars are digits
(defun all-digits? (chars)
  (if (== chars '())
    true
    (let ((c (car chars)))
      (let ((n (char-code c)))
        (if (and (>= n 48) (<= n 57))
          (all-digits? (cdr chars))
          false)))))

; Helper: check if string is a number
(defun is-number-string? (str)
  (if (== str "")
    false
    (all-digits? (string->list str))))

; Parse a single atom (number or symbol)
; Returns: (parsed-expr . remaining-tokens)
(defun parse-atom (token)
  (if (is-number-string? token)
    (cons (cons "number" (cons token '())) '())
    (cons (cons "symbol" (cons token '())) '())))

; Parse a list of expressions
; Returns: (parsed-list . remaining-tokens)
(defun parse-list-items (tokens acc)
  (if (== tokens '())
    (cons acc (cons '() '()))
    (if (== (car tokens) ")")
      (cons acc (cons (cdr tokens) '()))
      (let ((parsed (parse-expr tokens)))
        (let ((expr (car parsed)))
          (let ((rest (car (cdr parsed))))
            (parse-list-items rest (append acc (cons expr '())))))))))

; Parse a single expression
; Returns: (parsed-expr . remaining-tokens)
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

; Main parse function
(defun parse (tokens)
  (car (parse-expr tokens)))

; Tests
(print "=== Parser Tests ===")
(print "Test 1: number")
(print (parse '("42")))

(print "Test 2: symbol")
(print (parse '("foo")))

(print "Test 3: simple list")
(print (parse '("(" "+" "1" "2" ")")))
