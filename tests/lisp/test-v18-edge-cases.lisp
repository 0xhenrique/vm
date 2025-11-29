;; SKIP
;; Reason: Requires pattern matching and destructuring in function parameters (Phase 8)
; Test v18 edge cases and corner cases

(print "=== Testing v18 Edge Cases ===")
(print "")

; ============================================================================
; Test 1: Nested patterns - first element of each sublist
; ============================================================================

(defun first-of-first
  ((((x . _) . _)) x))

(print "Test 1: nested cons pattern")
(print (first-of-first '((1 2) 3)))
(print (first-of-first '((10 20 30) 40)))
(print "")

; ============================================================================
; Test 2: Pattern matching with mixed literals and cons
; ============================================================================

(defun count-down
  ((0) '())
  ((n) (cons n (count-down (- n 1)))))

(print "Test 2: mixed patterns")
(print (count-down 0))
(print (count-down 3))
(print (count-down 5))
(print "")

; ============================================================================
; Test 3: Multiple wildcards in pattern
; ============================================================================

(defun third
  (((_ _ x . _)) x))

(print "Test 3: multiple wildcards")
(print (third '(1 2 3 4)))
(print (third '(10 20 30)))
(print "")

; ============================================================================
; Test 4: Deep recursion with cons patterns
; ============================================================================

(defun range-helper
  ((start end acc)
    (if (> start end)
      acc
      (range-helper (+ start 1) end (cons start acc)))))

(defun range
  ((start end) (reverse (range-helper start end '()))))

(defun reverse
  (('()) '())
  (((h . t)) (append (reverse t) (cons h '()))))

(defun append
  (('() ys) ys)
  (((h . t) ys) (cons h (append t ys))))

(print "Test 4: deep recursion")
(print (range 1 5))
(print (range 10 15))
(print "")

; ============================================================================
; Test 5: Single element list pattern
; ============================================================================

(defun singleton?
  (((x . '())) true)
  ((_) false))

(print "Test 5: singleton list pattern")
(print (singleton? '(1)))
(print (singleton? '(1 2)))
(print (singleton? '()))
(print "")

; ============================================================================
; Test 6: Take first N elements with pattern matching
; ============================================================================

(defun take
  ((0 _) '())
  ((n '()) '())
  ((n (h . t)) (cons h (take (- n 1) t))))

(print "Test 6: take with patterns")
(print (take 0 '(1 2 3)))
(print (take 3 '(1 2 3 4 5)))
(print (take 2 '(1)))
(print (take 5 '()))
(print "")

(print "All v18 edge case tests passed!")
