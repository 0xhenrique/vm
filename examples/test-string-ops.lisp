; Test string operations for tokenizer

(print "Testing string-length:")
(print (string-length "hello"))

(print "")
(print "Testing string->list:")
(print (string->list "hi"))

(print "")
(print "Testing car of string->list:")
(print (car (string->list "hello")))

(print "")
(print "Testing substring:")
(print (substring "hello" 1 5))

(print "")
(print "Testing string comparisons:")
(print (== "a" "a"))
(print (== "a" "b"))

(print "")
(print "Testing number comparison:")
(print (== 0 0))
(print (== 1 0))
