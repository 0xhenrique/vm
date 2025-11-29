; Working tokenizer

(defun is-digit? (c)
  (let ((n (char-code c)))
    (and (>= n 48) (<= n 57))))

(defun is-space? (c)
  (== (char-code c) 32))

(defun is-letter? (c)
  (let ((n (char-code c)))
    (or (and (>= n 65) (<= n 90))   ; A-Z
        (and (>= n 97) (<= n 122))))) ; a-z

(defun is-symbol-char? (c)
  (let ((n (char-code c)))
    (or (is-letter? c)
        (is-digit? c)
        (== n 43)  ; +
        (== n 45)  ; -
        (== n 42)  ; *
        (== n 47)  ; /
        (== n 60)  ; <
        (== n 62)  ; >
        (== n 61)  ; =
        (== n 33)  ; !
        (== n 63)  ; ?
        (== n 95))))

; Read number
(defun read-num-acc (chars acc)
  (if (== chars '())
    (cons (list->string acc) (cons '() '()))
    (if (is-digit? (car chars))
      (read-num-acc (cdr chars) (append acc (cons (car chars) '())))
      (cons (list->string acc) (cons chars '())))))

; Read symbol
(defun read-symbol-acc (chars acc)
  (if (== chars '())
    (cons (list->string acc) (cons '() '()))
    (if (is-symbol-char? (car chars))
      (read-symbol-acc (cdr chars) (append acc (cons (car chars) '())))
      (cons (list->string acc) (cons chars '())))))

; Tokenize
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

; Tests
(print "=== Tokenizer with Symbol Recognition ===")
(print (tokenize "(+ 1 2)"))
(print (tokenize "( 10 20 )"))
(print (tokenize "(defun foo (x) (* x 2))"))
