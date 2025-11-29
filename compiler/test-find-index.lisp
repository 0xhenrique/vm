; Test find-index function

(defun find-index-helper (name lst index)
  (if (== lst '())
    -1
    (if (== (car lst) name)
      index
      (find-index-helper name (cdr lst) (+ index 1)))))

(defun find-index (name lst)
  (find-index-helper name lst 0))

(print "=== Testing find-index ===")
(print "")

(print "Test 1: Find 'x' in (x y z)")
(print (find-index "x" (cons "x" (cons "y" (cons "z" '())))))

(print "")
(print "Test 2: Find 'y' in (x y z)")
(print (find-index "y" (cons "x" (cons "y" (cons "z" '())))))

(print "")
(print "Test 3: Find 'z' in (x y z)")
(print (find-index "z" (cons "x" (cons "y" (cons "z" '())))))

(print "")
(print "Test 4: Find 'w' (not present)")
(print (find-index "w" (cons "x" (cons "y" (cons "z" '())))))

(print "")
(print "Test 5: Find in empty list")
(print (find-index "x" '()))
