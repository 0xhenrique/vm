; Tokenizer v3 - Simplified and working

; String helpers
(defun str-first (s)
  (car (string->list s)))

(defun str-rest (s)
  (if (< (string-length s) 2)
    ""
    (substring s 1 (string-length s))))

(defun str-empty? (s)
  (== (string-length s) 0))

; Character checks
(defun is-space? (c)
  (or (== c " ") (or (== c "\n") (or (== c "\t") (== c "\r")))))

(defun is-digit? (c)
  (let ((code (char-code c)))
    (and (>= code 48) (<= code 57))))

; Tokenizer
(defun tokenize (src)
  (reverse-list (tok-iter src '())))

(defun tok-iter (src tokens)
  (if (str-empty? src)
    tokens
    (let ((c (str-first src)))
      (if (is-space? c)
        (tok-iter (str-rest src) tokens)
      (if (== c "(")
        (tok-iter (str-rest src) (cons 'lparen tokens))
      (if (== c ")")
        (tok-iter (str-rest src) (cons 'rparen tokens))
      (if (== c "\"")
        (let ((result (read-str (str-rest src) "")))
          (tok-iter (car result) (cons (cons 'str (cons (car (cdr result)) '())) tokens)))
      (if (is-digit? c)
        (let ((result (read-num src 0)))
          (tok-iter (car result) (cons (cons 'num (cons (car (cdr result)) '())) tokens)))
        (let ((result (read-sym src "")))
          (tok-iter (car result) (cons (cons 'sym (cons (car (cdr result)) '())) tokens)))))))))))

; Read string
(defun read-str (src acc)
  (if (str-empty? src)
    (cons "" (cons acc '()))
    (if (== (str-first src) "\"")
      (cons (str-rest src) (cons acc '()))
      (read-str (str-rest src) (string-append acc (str-first src))))))

; Read number
(defun read-num (src acc)
  (if (str-empty? src)
    (cons "" (cons acc '()))
    (let ((c (str-first src)))
      (if (is-digit? c)
        (read-num (str-rest src) (+ (* acc 10) (- (char-code c) 48)))
        (cons src (cons acc '()))))))

; Read symbol
(defun read-sym (src acc)
  (if (str-empty? src)
    (cons "" (cons acc '()))
    (let ((c (str-first src)))
      (if (or (is-space? c) (or (== c "(") (== c ")")))
        (cons src (cons acc '()))
        (read-sym (str-rest src) (string-append acc c))))))

; Reverse list
(defun reverse-list (lst)
  (rev-iter lst '()))

(defun rev-iter (lst acc)
  (if (== lst '())
    acc
    (rev-iter (cdr lst) (cons (car lst) acc))))

; Tests
(print "Tokenizer v3!")
(print "")
(print "Test: (+ 1 2)")
(print (tokenize "(+ 1 2)"))
