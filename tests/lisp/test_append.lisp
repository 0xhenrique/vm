; Comprehensive tests for append primitive

(print "=== append Basic Tests ===")

; Test 1: Append two simple lists
(print "Test 1: Basic append")
(print (append '(1 2 3) '(4 5 6)))  ; (1 2 3 4 5 6)
(print (append '(a b) '(c d)))      ; (a b c d)

; Test 2: Append with empty list
(print "")
(print "Test 2: Append with empty lists")
(print (append '() '(1 2 3)))       ; (1 2 3)
(print (append '(1 2 3) '()))       ; (1 2 3)
(print (append '() '()))            ; ()

; Test 3: Append single element lists
(print "")
(print "Test 3: Single element lists")
(print (append '(1) '(2)))          ; (1 2)
(print (append '(x) '(y)))          ; (x y)

; Test 4: Append strings
(print "")
(print "Test 4: Lists of strings")
(print (append '("hello") '("world")))  ; ("hello" "world")
(print (append '("a" "b") '("c" "d" "e")))  ; ("a" "b" "c" "d" "e")

; Test 5: Append nested lists
(print "")
(print "Test 5: Nested lists")
(print (append '((1 2) (3 4)) '((5 6))))  ; ((1 2) (3 4) (5 6))
(print (append '((a)) '((b) (c))))        ; ((a) (b) (c))

; Test 6: Append mixed types
(print "")
(print "Test 6: Mixed types")
(print (append '(1 "hello" true) '(42 "world" false)))

; Test 7: Multiple appends
(print "")
(print "Test 7: Multiple appends")
(print (append (append '(1 2) '(3 4)) '(5 6)))  ; (1 2 3 4 5 6)
(print (append '(a) (append '(b) '(c))))        ; (a b c)

; Test 8: Using append in functions
(print "")
(print "Test 8: Append in functions")
(defun concat-lists (l1 l2 l3)
  (append (append l1 l2) l3))

(print (concat-lists '(1) '(2) '(3)))  ; (1 2 3)
(print (concat-lists '(a b) '(c d) '(e f)))  ; (a b c d e f)

; Test 9: Building lists with append
(print "")
(print "Test 9: Building lists")
(defun build-list (n)
  (if (== n 0)
    '()
    (append (build-list (- n 1)) (cons n '()))))

(print (build-list 5))  ; (1 2 3 4 5)

; Test 10: Append vs cons pattern
(print "")
(print "Test 10: Append vs cons")
(print (cons 1 '(2 3)))     ; (1 2 3) - cons adds to front
(print (append '(1) '(2 3))) ; (1 2 3) - append concatenates
(print (append '(2 3) '(1))) ; (2 3 1) - order matters

; Test 11: Recursive flatten using append
(print "")
(print "Test 11: Flatten one level")
(defun flatten-one (lst)
  (if (== lst '())
    '()
    (append (car lst) (flatten-one (cdr lst)))))

(print (flatten-one '((1 2) (3 4) (5 6))))  ; (1 2 3 4 5 6)

; Test 12: Reverse using append
(print "")
(print "Test 12: Reverse using append")
(defun reverse-append (lst)
  (if (== lst '())
    '()
    (append (reverse-append (cdr lst)) (cons (car lst) '()))))

(print (reverse-append '(1 2 3 4)))  ; (4 3 2 1)
(print (reverse-append '(a b c)))    ; (c b a)

; Test 13: Comparison with recursive append
(print "")
(print "Test 13: Verify primitive matches recursive version")
(defun append-recursive (l1 l2)
  (if (== l1 '())
    l2
    (cons (car l1) (append-recursive (cdr l1) l2))))

(print (append '(1 2 3) '(4 5 6)))              ; (1 2 3 4 5 6)
(print (append-recursive '(1 2 3) '(4 5 6)))    ; (1 2 3 4 5 6)

(print "")
(print "=== All append tests completed ===")
