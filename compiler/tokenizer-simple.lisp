; Debug tokenizer

(defun is-digit? ((c))
  (let ((n (char-code c)))
    (and (>= n 48) (<= n 57))))

(defun tokenize-debug ((str))
  (let ((chars (string->list str)))
    (print "Input: " )
    (print str)
    (print "As chars:")
    (print chars)
    (print "First char:")
    (print (car chars))
    (print "Is it '('?")
    (print (== (car chars) "("))
    (print "Is it digit?")
    (print (is-digit? (car chars)))))

(tokenize-debug "(+ 1 2)")
