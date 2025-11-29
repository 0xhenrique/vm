; Test what string->list returns

(print "Testing string->list:")
(print (string->list "abc"))
(print "First element:")
(print (car (string->list "abc")))
(print "Type check - is it a string?")
(print (== (car (string->list "abc")) "a"))
