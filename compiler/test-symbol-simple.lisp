; Test simple symbol recognition

(defun is-plus? (c)
  (== (char-code c) 43))

(print "Testing is-plus?:")
(print (is-plus? "+"))
(print (is-plus? "a"))
