; Test string predicates and utilities

(print "=== Testing String Predicates ===")
(print "")

(print "Testing string-starts-with?:")
(print (string-starts-with? "hello world" "hello"))
(print (string-starts-with? "hello world" "world"))
(print (string-starts-with? "" ""))

(print "")
(print "Testing string-ends-with?:")
(print (string-ends-with? "hello world" "world"))
(print (string-ends-with? "hello world" "hello"))
(print (string-ends-with? "test.lisp" ".lisp"))

(print "")
(print "Testing string-contains?:")
(print (string-contains? "hello world" "lo wo"))
(print (string-contains? "hello world" "xyz"))
(print (string-contains? "functional programming" "func"))

(print "")
(print "Testing string-upcase:")
(print (string-upcase "hello"))
(print (string-upcase "Hello World"))
(print (string-upcase "123abc"))

(print "")
(print "Testing string-downcase:")
(print (string-downcase "HELLO"))
(print (string-downcase "Hello World"))
(print (string-downcase "123ABC"))

(print "")
(print "Testing stdlib string utilities:")
(print (string-empty? ""))
(print (string-empty? "a"))
(print (string-blank? "   "))
(print (string-blank? "  x  "))

(print "")
(print "Testing string-repeat:")
(print (string-repeat "ab" 3))
(print (string-repeat "-" 5))

(print "")
(print "Testing string-reverse:")
(print (string-reverse "hello"))
(print (string-reverse "12345"))

(print "")
(print "=== All String Tests Complete! ===")
