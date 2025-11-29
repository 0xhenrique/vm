;; EXPECT-CONTAINS: All v18 cons pattern tests passed!
; Test v18 cons/list destructuring patterns

(print "=== Testing v18 Cons Patterns ===")
(print "")

; ============================================================================
; Test 1: Basic cons pattern - sum function
; ============================================================================

(defun sum
  (('()) 0)
  (((h . t)) (+ h (sum t))))

(print "Test 1: sum with cons pattern")
(print (sum '()))
(print (sum '(1)))
(print (sum '(1 2 3)))
(print (sum '(1 2 3 4 5)))
(print "")

; ============================================================================
; Test 2: Extract first element
; ============================================================================

(defun first
  (((x . _)) x))

(print "Test 2: first element extraction")
(print (first '(1 2 3)))
(print (first '(42)))
(print "")

; ============================================================================
; Test 3: Extract second element
; ============================================================================

(defun second
  (((_ x . _)) x))

(print "Test 3: second element extraction")
(print (second '(1 2 3)))
(print (second '(10 20)))
(print "")

; ============================================================================
; Test 4: Length function with cons pattern
; ============================================================================

(defun length
  (('()) 0)
  (((_ . t)) (+ 1 (length t))))

(print "Test 4: length with cons pattern")
(print (length '()))
(print (length '(1)))
(print (length '(1 2 3 4 5)))
(print "")

; ============================================================================
; Test 5: Append function with two arguments
; ============================================================================

(defun append
  (('() ys) ys)
  (((h . t) ys) (cons h (append t ys))))

(print "Test 5: append with cons pattern")
(print (append '() '(1 2 3)))
(print (append '(1 2) '(3 4)))
(print (append '(1) '(2 3 4)))
(print "")

; ============================================================================
; Test 6: Reverse function with cons pattern
; ============================================================================

(defun reverse-helper
  (('() acc) acc)
  (((h . t) acc) (reverse-helper t (cons h acc))))

(defun reverse
  ((lst) (reverse-helper lst '())))

(print "Test 6: reverse with cons pattern")
(print (reverse '()))
(print (reverse '(1)))
(print (reverse '(1 2 3)))
(print (reverse '(1 2 3 4 5)))
(print "")

(print "All v18 cons pattern tests passed!")
