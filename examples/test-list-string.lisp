; Test list->string behavior

(print "What does string->list give us?")
(let ((result (string->list "hello")))
  (print result)
  (print "First element:")
  (print (car result))
  (print "Is it a string?")
  (print (string? (car result))))

(print "")
(print "Can we convert it back?")
(print (list->string (string->list "hello")))

(print "")
(print "What about quoted list?")
(let ((quoted '("h" "e" "l")))
  (print quoted)
  (print "First element:")
  (print (car quoted))
  (print "Is it a string?")
  (print (string? (car quoted)))
  (print "Is it a symbol?")
  (print (symbol? (car quoted))))
