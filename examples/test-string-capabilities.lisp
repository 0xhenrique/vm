; Comprehensive string operation test

(print "=== Current String Operations ===")
(print "")

(print "1. string-length:")
(print (string-length "hello"))

(print "")
(print "2. substring:")
(print (substring "hello" 1 4))  ; "ell"

(print "")
(print "3. string-append:")
(print (string-append "hello" " world"))

(print "")
(print "4. string->list:")
(print (string->list "hi"))

(print "")
(print "5. list->string:")
(print (list->string '("h" "e" "l" "l" "o")))

(print "")
(print "6. char-code:")
(print (char-code "A"))

(print "")
(print "7. string comparisons:")
(print (== "hello" "hello"))
(print (== "hello" "world"))

(print "")
(print "=== What We're Missing ===")
(print "- string-ref (get char at index)")
(print "- string-split (split by delimiter)")
(print "- number->string conversion")
(print "- Better character predicates")
