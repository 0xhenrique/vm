; Test string equality

(print "Testing string equality:")
(print (== "(" "("))
(print (== ")" ")"))
(print (== "+" "+"))

(print "Testing char from string->list:")
(print (car (string->list "(+ 1)")))
(print (== (car (string->list "(+ 1)")) "("))
(print (car (cdr (string->list "(+ 1)"))))
(print (== (car (cdr (string->list "(+ 1)"))) "+"))
