; Simple test of label comparison

(defun labels-equal? (label1 label2)
  (if (== (car label1) (car label2))
    (== (car (cdr label1)) (car (cdr label2)))
    false))

(print "Test labels-equal?:")
(print (labels-equal? '(0 "ELSE") '(0 "ELSE")))
(print (labels-equal? '(0 "ELSE") '(1 "ELSE")))
(print (labels-equal? '(0 "ELSE") '(0 "END")))
