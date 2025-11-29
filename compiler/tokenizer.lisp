; ============================================================================
; Lisp Parser - Tokenizer
; ============================================================================
; Converts source code string into a list of tokens

; Token types: 'lparen, 'rparen, 'number, 'string, 'symbol, 'quote, 'true, 'false

; Character predicates
(defun is-whitespace (c)
  (or (== c " ")
  (or (== c "\n")
  (or (== c "\t")
      (== c "\r")))))

(defun is-digit (c)
  (let ((code (char-code c)))
    (and (>= code 48) (<= code 57))))  ; '0' to '9'

(defun is-alpha (c)
  (let ((code (char-code c)))
    (or (and (>= code 65) (<= code 90))   ; 'A' to 'Z'
        (and (>= code 97) (<= code 122))))) ; 'a' to 'z'

(defun is-symbol-char (c)
  (or (is-alpha c)
  (or (is-digit c)
  (or (== c "-")
  (or (== c "_")
  (or (== c "?")
  (or (== c "!")
  (or (== c ">")
  (or (== c "<")
  (or (== c "=")
  (or (== c "+")
  (or (== c "*")
  (or (== c "/")
      (== c "."))))))))))))))

; String helpers
(defun string-first (s)
  (if (== (string-length s) 0)
    ""
    (car (string->list s))))

(defun string-rest (s)
  (if (<= (string-length s) 1)
    ""
    (substring s 1 (string-length s))))

(defun string-empty? (s)
  (== (string-length s) 0))

; Main tokenizer
(defun tokenize (source)
  (tokenize-iter source '()))

(defun tokenize-iter (source tokens)
  (if (string-empty? source)
    (reverse-list tokens)
    (let ((c (string-first source)))
      (if (is-whitespace c)
        ; Skip whitespace
        (tokenize-iter (string-rest source) tokens)
      (if (== c ";")
        ; Skip comment to end of line
        (tokenize-iter (skip-comment source) tokens)
      (if (== c "(")
        (tokenize-iter (string-rest source) (cons '(lparen) tokens))
      (if (== c ")")
        (tokenize-iter (string-rest source) (cons '(rparen) tokens))
      (if (== c "'")
        (tokenize-iter (string-rest source) (cons '(quote) tokens))
      (if (== c "\"")
        ; String literal
        (let ((result (read-string-token (string-rest source) "")))
          (tokenize-iter (car result) (cons (cons 'string (cons (car (cdr result)) '())) tokens)))
      (if (is-digit c)
        ; Number
        (let ((result (read-number-token source "")))
          (tokenize-iter (car result) (cons (cons 'number (cons (car (cdr result)) '())) tokens)))
        ; Symbol
        (let ((result (read-symbol-token source "")))
          (let ((sym (car (cdr result))))
            (tokenize-iter (car result)
              (cons (if (== sym "true")
                      '(true)
                    (if (== sym "false")
                      '(false)
                      (cons 'symbol (cons sym '()))))
                    tokens)))))))))))))))

; Skip comment until newline
(defun skip-comment (source)
  (if (string-empty? source)
    ""
    (if (== (string-first source) "\n")
      (string-rest source)
      (skip-comment (string-rest source)))))

; Read string token (after opening quote)
(defun read-string-token (source acc)
  (if (string-empty? source)
    (cons "" (cons acc '()))  ; Error: unclosed string
    (let ((c (string-first source)))
      (if (== c "\"")
        (cons (string-rest source) (cons acc '()))
        (if (== c "\\")
          ; Handle escape (simplified - only supports \\, \", \n)
          (let ((next (string-first (string-rest source))))
            (if (== next "n")
              (read-string-token (string-rest (string-rest source)) (string-append acc "\n"))
            (if (== next "\\")
              (read-string-token (string-rest (string-rest source)) (string-append acc "\\"))
              (read-string-token (string-rest (string-rest source)) (string-append acc next)))))
          ; Regular character
          (read-string-token (string-rest source) (string-append acc c)))))))

; Read number token
(defun read-number-token (source acc)
  (if (string-empty? source)
    (cons "" (cons (string-to-int acc) '()))
    (let ((c (string-first source)))
      (if (is-digit c)
        (read-number-token (string-rest source) (string-append acc c))
        (cons source (cons (string-to-int acc) '()))))))

; Read symbol token
(defun read-symbol-token (source acc)
  (if (string-empty? source)
    (cons "" (cons acc '()))
    (let ((c (string-first source)))
      (if (is-symbol-char c)
        (read-symbol-token (string-rest source) (string-append acc c))
        (cons source (cons acc '()))))))

; String to integer
(defun string-to-int (s)
  (string-to-int-iter (string->list s) 0))

(defun string-to-int-iter (chars acc)
  (if (== chars '())
    acc
    (let ((digit (- (char-code (car chars)) 48)))
      (string-to-int-iter (cdr chars) (+ (* acc 10) digit)))))

; Reverse list
(defun reverse-list (lst)
  (reverse-iter lst '()))

(defun reverse-iter (lst acc)
  (if (== lst '())
    acc
    (reverse-iter (cdr lst) (cons (car lst) acc))))

; Test tokenizer
(print "Tokenizer loaded!")
(print "")
(print "Test 1: Simple expression")
(print (tokenize "(+ 1 2)"))
(print "Expected: ((lparen) (symbol \"+\") (number 1) (number 2) (rparen))")
(print "")
(print "Test 2: Defun")
(print (tokenize "(defun fact (n) (+ n 1))"))
(print "")
(print "Test 3: String")
(print (tokenize "(print \"hello\")"))
