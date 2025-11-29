;; Tests for Phase 12a: Parallel Collections
;; pmap, pfilter, preduce

(print "=== Testing pmap ===")

;; pmap with simple function
(defun square (x) (* x x))
(print (pmap square '(1 2 3 4 5)))
;; Expected: (1 4 9 16 25)

;; pmap with lambda
(print (pmap (lambda (x) (+ x 10)) '(1 2 3)))
;; Expected: (11 12 13)

;; pmap with empty list
(print (pmap square '()))
;; Expected: ()

;; pmap with single element
(print (pmap (lambda (x) (* x 2)) '(42)))
;; Expected: (84)

(print "=== Testing pfilter ===")

;; pfilter with simple predicate
(defun is_even (x) (== (% x 2) 0))
(print (pfilter is_even '(1 2 3 4 5 6)))
;; Expected: (2 4 6)

;; pfilter with lambda
(print (pfilter (lambda (x) (> x 5)) '(1 3 5 7 9 11)))
;; Expected: (7 9 11)

;; pfilter with empty list
(print (pfilter (lambda (x) (> x 0)) '()))
;; Expected: ()

;; pfilter where none match
(print (pfilter (lambda (x) (> x 100)) '(1 2 3 4 5)))
;; Expected: ()

;; pfilter where all match
(print (pfilter (lambda (x) (> x 0)) '(1 2 3 4 5)))
;; Expected: (1 2 3 4 5)

(print "=== Testing preduce ===")

;; preduce for sum
(defun add (a b) (+ a b))
(print (preduce '(1 2 3 4 5) 0 add))
;; Expected: 15

;; preduce for product
(defun mul (a b) (* a b))
(print (preduce '(1 2 3 4 5) 1 mul))
;; Expected: 120

;; preduce with lambda
(print (preduce '(1 2 3 4) 0 (lambda (acc x) (+ acc (* x x)))))
;; Expected: 30 (1^2 + 2^2 + 3^2 + 4^2)

;; preduce with empty list
(print (preduce '() 42 (lambda (a b) (+ a b))))
;; Expected: 42

;; preduce with single element
(print (preduce '(5) 10 (lambda (a b) (+ a b))))
;; Expected: 15

(print "=== Testing combined operations ===")

;; Chain: pmap then pfilter
(def squared (pmap (lambda (x) (* x x)) '(1 2 3 4 5)))
(print (pfilter (lambda (x) (> x 10)) squared))
;; Expected: (16 25)

;; Chain: pmap, pfilter, preduce
(def doubled (pmap (lambda (x) (* x 2)) '(1 2 3 4 5)))
(def evens (pfilter (lambda (x) (== (% x 4) 0)) doubled))
(print (preduce evens 0 (lambda (a b) (+ a b))))
;; Expected: 12 (4 + 8)

;; Using pmap for data transformation
(defun make-pair (x) (cons x (cons (* x 2) '())))
(print (pmap make-pair '(1 2 3)))
;; Expected: ((1 2) (2 4) (3 6))

;; Using pfilter for validation
(defun valid-positive (x) (> x 0))
(print (pfilter valid-positive '(-5 -3 0 2 4 6)))
;; Expected: (2 4 6)

;; Using preduce for aggregation
(defun max-val (a b) (if (> a b) a b))
(print (preduce '(3 7 2 9 1 8) 0 max-val))
;; Expected: 9

(print "=== All parallel collections tests completed ===")
