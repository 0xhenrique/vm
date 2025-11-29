; Test what primitives would help

(print "=== What Works ===")
(print (string->list "hi"))
(print (list->string (string->list "hello")))

(print "")
(print "=== What Would Help ===")

(print "1. Need: string-ref (get nth char)")
(print "   Workaround: (car (cdr (string->list s))) for 2nd char")

(print "")
(print "2. Need: append as primitive")
(print "   Currently: must implement recursively")

(print "")
(print "3. Need: number->string")
(print "   Currently: no way to convert 42 to string")

(print "")
(print "4. Need: Better char predicates")
(print "   is-digit?, is-alpha?, is-space?")
(print "   Currently: must check char-code ranges")

(print "")
(print "5. Would help: length as primitive")
(print "   Currently: must count recursively")
