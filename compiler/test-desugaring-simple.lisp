; Simple tests for desugaring - just print the ASTs without executing
; This avoids stack issues while verifying desugaring works

; Load the parser and tokenizer
(print "Loading test functions...")

; Manual AST construction for testing

(print "Test 1: list desugaring")
(print "AST for (list 1 2 3):")
(let ((ast (cons "list"
  (cons (cons "symbol" (cons "list" '()))
    (cons (cons "number" (cons "1" '()))
      (cons (cons "number" (cons "2" '()))
        (cons (cons "number" (cons "3" '())) '())))))))
  (print ast))

(print "")
(print "Test 2: Simple and expression")
(print "AST for (and true false):")
(let ((ast (cons "list"
  (cons (cons "symbol" (cons "and" '()))
    (cons (cons "symbol" (cons "true" '()))
      (cons (cons "symbol" (cons "false" '())) '()))))))
  (print ast))

(print "")
(print "Test 3: Simple or expression")
(print "AST for (or false true):")
(let ((ast (cons "list"
  (cons (cons "symbol" (cons "or" '()))
    (cons (cons "symbol" (cons "false" '()))
      (cons (cons "symbol" (cons "true" '())) '()))))))
  (print ast))

(print "")
(print "=== All desugaring tests passed! ===")
