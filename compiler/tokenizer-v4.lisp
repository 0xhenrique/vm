; Tokenizer v4 - Using cond for clarity

; String operations
(defun first-char (s)
  (car (string->list s)))

(defun rest-str (s)
  (if (< (string-length s) 2)
    ""
    (substring s 1 (string-length s))))

(defun empty-str? (s)
  (== (string-length s) 0))

; Simple tokenize - just ( ) and symbols for now
(defun tokenize (src)
  (reverse-list (tok src '())))

(defun tok (src tokens)
  (if (empty-str? src)
    tokens
    (let ((c (first-char src)))
    (let ((rest (rest-str src)))
      (cond
        ((== c " ") (tok rest tokens))
        ((== c "\n") (tok rest tokens))
        ((== c "(") (tok rest (cons 'lparen tokens)))
        ((== c ")") (tok rest (cons 'rparen tokens)))
        (else (tok rest (cons 'symbol tokens))))))))

(defun reverse-list (lst)
  (rev-iter lst '()))

(defun rev-iter (lst acc)
  (if (== lst '())
    acc
    (rev-iter (cdr lst) (cons (car lst) acc))))

; Test
(print "Tokenizer v4 with cond!")
(print (tokenize "(+ 1 2)"))
(print "Expected: (lparen symbol symbol symbol symbol rparen)")
