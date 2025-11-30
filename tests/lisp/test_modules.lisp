; Test module system functionality
; This tests module definitions, exports, imports, and namespace isolation

(print "=== Basic Module Definition ===")

; Define a simple math module
(module math
    (export add subtract multiply)
    (defun add (x y) (+ x y))
    (defun subtract (x y) (- x y))
    (defun multiply (x y) (* x y))
    (defun private-helper (x) (* x 2)))  ; not exported

; Test calling exported functions with qualified names
(print (format "math/add 10 5 = {}" (list (math/add 10 5))))
(print (format "math/subtract 10 5 = {}" (list (math/subtract 10 5))))
(print (format "math/multiply 10 5 = {}" (list (math/multiply 10 5))))

(print "")
(print "=== Import Module ===")

; Import the math module
(import math)

; Can still use qualified names after import
(print (format "math/add 20 10 = {}" (list (math/add 20 10))))

(print "")
(print "=== Selective Import ===")

; Define a string utilities module
(module strings
    (export greet farewell)
    (defun greet (name) (string-append "Hello, " name))
    (defun farewell (name) (string-append "Goodbye, " name)))

; Import specific symbols
(import strings greet farewell)

; Now can use unqualified names
(print (greet "World"))
(print (farewell "World"))

(print "")
(print "=== Module with Constants ===")

; Module with def (constants)
(module constants
    (export pi e)
    (def pi 314)  ; Using integer for simplicity
    (def e 271))

; Access module constants
(print (format "pi = {}" (list constants/pi)))
(print (format "e = {}" (list constants/e)))

(print "")
(print "=== Functions Using Module Constants ===")

(module circle
    (export area circumference)
    (def pi 314)
    (defun area (r) (/ (* pi (* r r)) 100))
    (defun circumference (r) (/ (* 2 (* pi r)) 100)))

(print (format "Area of circle with r=10: {}" (list (circle/area 10))))
(print (format "Circumference with r=10: {}" (list (circle/circumference 10))))

(print "")
(print "=== Recursive Functions in Modules ===")

(module factorial-mod
    (export factorial)
    (defun factorial (n)
        (if (<= n 1)
            1
            (* n (factorial (- n 1))))))

(print (format "5! = {}" (list (factorial-mod/factorial 5))))
(print (format "10! = {}" (list (factorial-mod/factorial 10))))

(print "")
(print "=== Mutual Recursion in Modules ===")

(module parity
    (export even odd)
    (defun even (n)
        (if (== n 0)
            true
            (odd (- n 1))))
    (defun odd (n)
        (if (== n 0)
            false
            (even (- n 1)))))

(print (format "even 4 = {}" (list (parity/even 4))))
(print (format "odd 4 = {}" (list (parity/odd 4))))
(print (format "even 7 = {}" (list (parity/even 7))))
(print (format "odd 7 = {}" (list (parity/odd 7))))

(print "")
(print "=== Module Using Another Module ===")

(module utils
    (export double triple)
    (defun double (x) (* x 2))
    (defun triple (x) (* x 3)))

(module calculations
    (export six-times)
    (import utils double triple)
    (defun six-times (x) (double (triple x))))

(print (format "six-times 5 = {}" (list (calculations/six-times 5))))

(print "")
(print "=== Multiple Modules Working Together ===")

(module validators
    (export is-positive is-negative)
    (defun is-positive (n) (> n 0))
    (defun is-negative (n) (< n 0)))

(module combiners
    (export classify)
    (import validators is-positive is-negative)
    (defun classify (n)
        (if (is-positive n)
            "positive"
            (if (is-negative n)
                "negative"
                "zero"))))

(print (format "classify 5 = {}" (list (combiners/classify 5))))
(print (format "classify -3 = {}" (list (combiners/classify -3))))
(print (format "classify 0 = {}" (list (combiners/classify 0))))

(print "")
(print "=== Closures in Modules ===")

(module counter
    (export make-counter make-incrementer)
    (defun make-counter ()
        (let ((count 0))
            (lambda ()
                (let ((old count))
                    old))))
    (defun make-incrementer (start)
        (lambda (n) (+ start n))))

; Use let to store closures (closures as values work correctly with let)
(let ((counter1 (counter/make-counter)))
    (print (format "counter1 = {}" (list (counter1)))))

(let ((add10 (counter/make-incrementer 10)))
    (begin
        (print (format "add10 5 = {}" (list (add10 5))))
        (print (format "add10 100 = {}" (list (add10 100))))))

(print "")
(print "=== Higher-Order Functions in Modules ===")

(module hof
    (export apply-twice compose)
    (defun apply-twice (f x) (f (f x)))
    (defun compose (f g)
        (lambda (x) (f (g x)))))

(defun square (x) (* x x))
(defun inc (x) (+ x 1))

(print (format "apply-twice square 2 = {}" (list (hof/apply-twice square 2))))
; square(square(2)) = square(4) = 16

; Use let for composed function since it's a closure
(let ((square-then-inc (hof/compose inc square)))
    (print (format "(compose inc square) 5 = {}" (list (square-then-inc 5)))))
; square(5) = 25, inc(25) = 26

(print "")
(print "=== Pattern Matching in Modules ===")

(module patterns
    (export describe-number)
    (defun describe-number
        ((0) "zero")
        ((1) "one")
        ((2) "two")
        ((_) "many")))

(print (format "describe 0 = {}" (list (patterns/describe-number 0))))
(print (format "describe 1 = {}" (list (patterns/describe-number 1))))
(print (format "describe 2 = {}" (list (patterns/describe-number 2))))
(print (format "describe 99 = {}" (list (patterns/describe-number 99))))

(print "")
(print "=== All Module Tests Passed! ===")
