;; SKIP
;; Reason: Requires pattern matching and destructuring in function parameters (Phase 8)
;; ============================================================================
;; Standard Library Tests
;; Phase 11: Standard Library Implementation
;; ============================================================================

;; Load the standard library
;; Note: In a real implementation, stdlib.lisp should be automatically loaded
;; For now, we'll define the functions inline or ensure they're available

;; ----------------------------------------------------------------------------
;; Helper Functions (from stdlib)
;; ----------------------------------------------------------------------------

(defun map
  ((f '()) '())
  ((f (h . t)) (cons (f h) (map f t))))

(defun filter
  ((predicate '()) '())
  ((predicate (h . t))
    (if (predicate h)
      (cons h (filter predicate t))
      (filter predicate t))))

(defun reduce
  ((f acc '()) acc)
  ((f acc (h . t)) (reduce f (f acc h) t)))

(defun length
  (('()) 0)
  (((h . t)) (+ 1 (length t))))

(defun nth
  ((0 (h . t)) h)
  ((n (h . t)) (nth (- n 1) t))
  ((n '()) '()))

(defun last
  (((h . '())) h)
  (((h . t)) (last t))
  (('()) '()))

(defun reverse-helper
  ((acc '()) acc)
  ((acc (h . t)) (reverse-helper (cons h acc) t)))

(defun reverse (lst)
  (reverse-helper '() lst))

(defun append
  (('() ys) ys)
  (((h . t) ys) (cons h (append t ys))))

(defun abs (n)
  (if (< n 0)
    (- 0 n)
    n))

(defun min
  ((a b) (if (< a b) a b)))

(defun max
  ((a b) (if (> a b) a b)))

(defun even?
  ((0) true)
  ((1) false)
  ((n) (if (< n 0)
         (even? (- 0 n))
         (even? (- n 2)))))

(defun odd? (n)
  (if (even? n) false true))

(defun identity (x) x)

(defun compose
  ((f g) (lambda (x) (f (g x)))))

(defun partial
  ((f arg) (lambda (x) (f arg x))))

(defun not (x)
  (if x false true))

(defun null?
  (('()) true)
  ((x) false))

(defun sum (lst)
  (reduce (lambda (acc x) (+ acc x)) 0 lst))

(defun product (lst)
  (reduce (lambda (acc x) (* acc x)) 1 lst))

;; ----------------------------------------------------------------------------
;; List Utilities Tests
;; ----------------------------------------------------------------------------

(print (length '()))
(print (length '(1)))
(print (length '(1 2 3)))
(print (length '(a b c d e)))

(print (nth 0 '(10 20 30)))
(print (nth 1 '(10 20 30)))
(print (nth 2 '(10 20 30)))
(print (nth 3 '(10 20 30)))

(print (last '(1)))
(print (last '(1 2 3)))
(print (last '(a b c)))
(print (last '()))

(print (reverse '()))
(print (reverse '(1)))
(print (reverse '(1 2 3)))
(print (reverse '(a b c d e)))

(print (append '() '()))
(print (append '(1 2) '(3 4)))
(print (append '(a) '(b c d)))
(print (append '() '(1 2 3)))
(print (append '(1 2 3) '()))

;; ----------------------------------------------------------------------------
;; Numeric Utilities Tests
;; ----------------------------------------------------------------------------

(print (abs 5))
(print (abs -5))
(print (abs 0))
(print (abs -100))

(print (min 3 5))
(print (min 5 3))
(print (min -5 -3))
(print (min 0 10))

(print (max 3 5))
(print (max 5 3))
(print (max -5 -3))
(print (max 0 10))

(print (even? 0))
(print (even? 1))
(print (even? 2))
(print (even? 3))
(print (even? 4))
(print (even? -2))
(print (even? -3))

(print (odd? 0))
(print (odd? 1))
(print (odd? 2))
(print (odd? 3))
(print (odd? 4))
(print (odd? -1))
(print (odd? -2))

;; ----------------------------------------------------------------------------
;; Functional Utilities Tests
;; ----------------------------------------------------------------------------

(print (identity 42))
(print (identity true))
(print (identity '(1 2 3)))

(defun add1 (x) (+ x 1))
(defun double (x) (* x 2))

(print ((compose add1 double) 5))
(print ((compose double add1) 5))

(defun add (a b) (+ a b))
(print ((partial add 10) 5))
(print ((partial add 100) 25))

;; ----------------------------------------------------------------------------
;; Higher-Order Functions Tests (from Phase 8)
;; ----------------------------------------------------------------------------

(print (map (lambda (x) (* x 2)) '(1 2 3 4 5)))
(print (map (lambda (x) (+ x 1)) '(10 20 30)))
(print (map identity '(1 2 3)))

(print (filter (lambda (x) (> x 2)) '(1 2 3 4 5)))
(print (filter even? '(1 2 3 4 5 6 7 8)))
(print (filter odd? '(1 2 3 4 5 6 7 8)))

(print (reduce (lambda (acc x) (+ acc x)) 0 '(1 2 3 4 5)))
(print (reduce (lambda (acc x) (* acc x)) 1 '(1 2 3 4 5)))

;; ----------------------------------------------------------------------------
;; Additional Utility Functions Tests
;; ----------------------------------------------------------------------------

(print (not true))
(print (not false))

(print (null? '()))
(print (null? '(1)))
(print (null? 42))

(print (sum '(1 2 3 4 5)))
(print (sum '()))
(print (sum '(10)))

(print (product '(1 2 3 4 5)))
(print (product '()))
(print (product '(10)))

;; ----------------------------------------------------------------------------
;; Combined Tests - Using Multiple Functions Together
;; ----------------------------------------------------------------------------

(print (sum (map (lambda (x) (* x x)) '(1 2 3 4))))

(print (length (filter even? '(1 2 3 4 5 6 7 8 9 10))))

(print (reverse (map (lambda (x) (* x 2)) '(1 2 3 4 5))))

(print (reduce max 0 '(3 7 2 9 4 1)))

(print (reduce min 100 '(3 7 2 9 4 1)))

(defun square (x) (* x x))
(print (sum (map square (filter odd? '(1 2 3 4 5 6 7 8 9 10)))))

(print ((compose (partial map (lambda (x) (* x 2)))
                 (partial filter (lambda (x) (> x 5))))
        '(1 2 3 4 5 6 7 8 9 10)))

;; Test nested list operations
(print (map length '((1 2) (1 2 3) () (1))))

;; Test chaining operations
(print (last (reverse '(1 2 3 4 5))))

;; Test append with multiple operations
(print (append (filter even? '(1 2 3 4)) (filter odd? '(1 2 3 4))))

;; End of tests
