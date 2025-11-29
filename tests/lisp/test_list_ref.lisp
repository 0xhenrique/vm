; Comprehensive tests for list-ref primitive

(print "=== list-ref Basic Tests ===")

; Test 1: Basic indexing
(print "Test 1: Basic indexing")
(print (list-ref '(10 20 30 40 50) 0))  ; 10
(print (list-ref '(10 20 30 40 50) 1))  ; 20
(print (list-ref '(10 20 30 40 50) 4))  ; 50

; Test 2: Nested lists
(print "")
(print "Test 2: Nested lists")
(print (list-ref '((1 2) (3 4) (5 6)) 0))  ; (1 2)
(print (list-ref '((1 2) (3 4) (5 6)) 2))  ; (5 6)
(print (list-ref (list-ref '((1 2) (3 4) (5 6)) 1) 0))  ; 3
(print (list-ref (list-ref '((1 2) (3 4) (5 6)) 1) 1))  ; 4

; Test 3: Mixed types
(print "")
(print "Test 3: Mixed types")
(print (list-ref '(42 "hello" true) 0))  ; 42
(print (list-ref '(42 "hello" true) 1))  ; "hello"
(print (list-ref '(42 "hello" true) 2))  ; true

; Test 4: Single element list
(print "")
(print "Test 4: Single element list")
(print (list-ref '(99) 0))  ; 99

; Test 5: String list
(print "")
(print "Test 5: String list")
(print (list-ref '("first" "second" "third") 0))  ; "first"
(print (list-ref '("first" "second" "third") 2))  ; "third"

; Test 6: Using list-ref in functions
(print "")
(print "Test 6: Using list-ref in functions")
(defun get-second (lst)
  (list-ref lst 1))

(defun get-last-of-three (lst)
  (list-ref lst 2))

(print (get-second '(a b c)))        ; b
(print (get-last-of-three '(x y z))) ; z

; Test 7: Using list-ref with computed indices
(print "")
(print "Test 7: Computed indices")
(print (list-ref '(100 200 300 400) (+ 1 1)))  ; 300
(print (list-ref '(100 200 300 400) (- 3 2)))  ; 200

; Test 8: Recursive function using list-ref
(print "")
(print "Test 8: Recursive sum using list-ref")
(defun sum-list-ref (lst idx len)
  (if (== idx len)
    0
    (+ (list-ref lst idx) (sum-list-ref lst (+ idx 1) len))))

(print (sum-list-ref '(5 10 15 20) 0 4))  ; 50

; Test 9: Using with list-length
(print "")
(print "Test 9: Getting last element using list-length")
(defun get-last (lst)
  (list-ref lst (- (list-length lst) 1)))

(print (get-last '(10 20 30 40)))  ; 40
(print (get-last '("a" "b" "c")))  ; "c"

(print "")
(print "=== All list-ref tests completed ===")
