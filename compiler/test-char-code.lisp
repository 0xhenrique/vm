; Test char-code

(print "Testing char-code:")
(print (char-code "("))
(print (char-code "+"))
(print (char-code "1"))

(print "From string->list:")
(let ((chars (string->list "(+1)")))
  (print (char-code (car chars))))
