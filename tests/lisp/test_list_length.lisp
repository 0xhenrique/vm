; Comprehensive tests for list-length primitive

(print "=== list-length Basic Tests ===")

; Test 1: Empty list
(print "Test 1: Empty list")
(print (list-length '()))  ; 0

; Test 2: Single element
(print "")
(print "Test 2: Single element")
(print (list-length '(1)))      ; 1
(print (list-length '("hello"))) ; 1

; Test 3: Multiple elements
(print "")
(print "Test 3: Multiple elements")
(print (list-length '(1 2 3)))           ; 3
(print (list-length '(a b c d e)))       ; 5
(print (list-length '(10 20 30 40 50 60 70 80 90 100))) ; 10

; Test 4: Nested lists (outer length only)
(print "")
(print "Test 4: Nested lists")
(print (list-length '((1 2) (3 4) (5 6))))  ; 3
(print (list-length '((a) (b) (c) (d))))     ; 4

; Test 5: Mixed types
(print "")
(print "Test 5: Mixed types")
(print (list-length '(1 "hello" true)))  ; 3

; Test 6: Using in conditionals
(print "")
(print "Test 6: Using in conditionals")
(defun is-empty? (lst)
  (== (list-length lst) 0))

(defun is-single? (lst)
  (== (list-length lst) 1))

(print (is-empty? '()))        ; true
(print (is-empty? '(1)))       ; false
(print (is-single? '(x)))      ; true
(print (is-single? '(x y)))    ; false

; Test 7: Comparing lengths
(print "")
(print "Test 7: Comparing lengths")
(defun longer? (lst1 lst2)
  (> (list-length lst1) (list-length lst2)))

(print (longer? '(1 2 3) '(a b)))      ; true
(print (longer? '(x) '(y z w)))        ; false

; Test 8: Safe list access
(print "")
(print "Test 8: Safe list access")
(defun safe-ref (lst idx)
  (if (< idx (list-length lst))
    (list-ref lst idx)
    "out-of-bounds"))

(print (safe-ref '(10 20 30) 1))   ; 20
(print (safe-ref '(10 20 30) 5))   ; "out-of-bounds"

; Test 9: Reverse using length and list-ref
(print "")
(print "Test 9: Reverse using list-length and list-ref")
(defun reverse-with-length (lst)
  (reverse-helper lst '() 0 (list-length lst)))

(defun reverse-helper (lst acc idx len)
  (if (== idx len)
    acc
    (reverse-helper lst
                    (cons (list-ref lst idx) acc)
                    (+ idx 1)
                    len)))

(print (reverse-with-length '(1 2 3 4)))  ; (4 3 2 1)
(print (reverse-with-length '(a b c)))    ; (c b a)

; Test 10: Comparison with recursive version
(print "")
(print "Test 10: Verify primitive matches recursive version")
(defun count-recursive (lst)
  (if (== lst '())
    0
    (+ 1 (count-recursive (cdr lst)))))

(print (list-length '(a b c d e f)))      ; 6
(print (count-recursive '(a b c d e f)))  ; 6

(print "")
(print "=== All list-length tests completed ===")
