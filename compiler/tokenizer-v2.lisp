; Tokenizer v2 - Fixed version

; String helpers
(defun str-first (s)
  (if (== (string-length s) 0)
    ""
    (car (string->list s))))

(defun str-rest (s)
  (if (< (string-length s) 2)
    ""
    (substring s 1 (string-length s))))

(defun str-empty? (s)
  (== (string-length s) 0))

; Character type checks
(defun is-space? (c)
  (or (== c " ")
  (or (== c "\n")
  (or (== c "\t")
      (== c "\r")))))

(defun is-digit? (c)
  (let ((code (char-code c)))
    (if (< code 48) false
    (if (> code 57) false
      true))))

; Simple tokenizer
(defun tokenize (src)
  (tok-iter src '()))

(defun tok-iter (src tokens)
  (if (str-empty? src)
    (reverse-list tokens)
    (let ((c (str-first src)))
      (if (is-space? c)
        (tok-iter (str-rest src) tokens)
      (if (== c "(")
        (tok-iter (str-rest src) (cons (list 'lparen) tokens))
      (if (== c ")")
        (tok-iter (str-rest src) (cons (list 'rparen) tokens))
      (if (== c "\"")
        ; String - read until closing quote
        (let ((result (read-string (str-rest src) "")))
          (tok-iter (car result) (cons (list 'string (car (cdr result))) tokens)))
      (if (is-digit? c)
        ; Number
        (let ((result (read-number src 0)))
          (tok-iter (car result) (cons (list 'number (car (cdr result))) tokens)))
        ; Symbol
        (let ((result (read-symbol src "")))
          (tok-iter (car result) (cons (list 'symbol (car (cdr result))) tokens)))))))))))

; Read string literal
(defun read-string (src acc)
  (if (str-empty? src)
    (list "" acc)  ; Unclosed string
    (if (== (str-first src) "\"")
      (list (str-rest src) acc)
      (read-string (str-rest src) (string-append acc (str-first src))))))

; Read number
(defun read-number (src acc)
  (if (str-empty? src)
    (list "" acc)
    (let ((c (str-first src)))
      (if (is-digit? c)
        (read-number (str-rest src) (+ (* acc 10) (- (char-code c) 48)))
        (list src acc)))))

; Read symbol
(defun read-symbol (src acc)
  (if (str-empty? src)
    (list "" acc)
    (let ((c (str-first src)))
      (if (or (is-space? c) (or (== c "(") (== c ")")))
        (list src acc)
        (read-symbol (str-rest src) (string-append acc c))))))

; Reverse list
(defun reverse-list (lst)
  (rev-iter lst '()))

(defun rev-iter (lst acc)
  (if (== lst '())
    acc
    (rev-iter (cdr lst) (cons (car lst) acc))))

; List helper
(defun list (a) (cons a '()))
(defun list (a b) (cons a (cons b '())))

; Tests
(print "Tokenizer v2 loaded!")
(print "")
(print "Test 1: Simple")
(print (tokenize "(+ 1 2)"))
(print "")
(print "Test 2: String")
(print (tokenize "(print \"hello\")"))
(print "")
(print "Test 3: Defun")
(print (tokenize "(defun add (x y) (+ x y))"))
