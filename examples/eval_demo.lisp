;; Eval Function Demo - Runtime Code Evaluation
;; This example demonstrates the eval function for metaprogramming

(print "=== Eval Function Demo ===")

;; Basic eval - evaluate a simple expression
(print "")
(print "1. Basic Expression Evaluation:")
(print (eval "(+ 10 20)"))

;; Eval with nested operations
(print "")
(print "2. Nested Arithmetic:")
(print (eval "(* (+ 2 3) (- 10 3))"))

;; Define a function via eval
(print "")
(print "3. Defining Functions with Eval:")
(eval "(defun triple (x) (* x 3))")
(print (triple 7))

;; Lambda expressions in eval
(print "")
(print "4. Lambda Expressions:")
(print (eval "((lambda (x) (* x x)) 6)"))

;; Closures with eval
(print "")
(print "5. Closures:")
(eval "(defun make-multiplier (n) (lambda (x) (* n x)))")
(print ((make-multiplier 5) 8))

;; Conditional expressions
(print "")
(print "6. Conditional Evaluation:")
(print (eval "(if (> 10 5) (quote yes) (quote no))"))

;; Recursive function via eval
(print "")
(print "7. Recursive Functions:")
(eval "(defun factorial (n) (if (<= n 1) 1 (* n (factorial (- n 1)))))")
(print (factorial 5))

;; List operations
(print "")
(print "8. List Manipulation:")
(print (eval "(cons 1 (cons 2 (cons 3 (list))))"))

;; String operations
(print "")
(print "9. String Operations:")
(print (eval "(string-length (symbol->string (quote hello)))"))

;; Float arithmetic
(print "")
(print "10. Floating Point Math:")
(print (eval "(sqrt (* 2.0 8.0))"))

;; Multiple expressions in one eval
(print "")
(print "11. Multiple Eval Calls:")
(eval "(defun add (x y) (+ x y))")
(eval "(defun sub (x y) (- x y))")
(print (add 10 (sub 20 5)))

;; Nested lambdas with closures
(print "")
(print "12. Nested Closures:")
(print (eval "((lambda (x) ((lambda (y) (+ x y)) 10)) 5)"))

;; ========== Context-Aware Eval (Cross-Context Features) ==========

;; Eval can call functions from parent context
(print "")
(print "13. Eval Calls Parent Function:")
(defun square (x) (* x x))
(print (eval "(square 8)"))

;; Eval can access parent globals
(print "")
(print "14. Eval Accesses Parent Global:")
(def myvar 99)
(print (eval "myvar"))

;; Eval can use parent functions in new definitions
(print "")
(print "15. Eval Uses Parent in Definition:")
(defun double (x) (* x 2))
(eval "(defun quadruple (x) (double (double x)))")
(print (quadruple 7))

;; Chain of evals building on each other
(print "")
(print "16. Chained Evals:")
(eval "(defun add (x y) (+ x y))")
(eval "(defun mul (x y) (* x y))")
(print (eval "(add (mul 4 5) 3)"))

;; Complex interaction - eval with closures and parent functions
(print "")
(print "17. Complex Cross-Context:")
(defun increment (x) (+ x 1))
(eval "(defun make-add (x y) (+ (increment x) y))")
(print (make-add 10 20))

(print "")
(print "=== End of Eval Demo ===")
