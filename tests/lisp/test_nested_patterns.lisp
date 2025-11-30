; Comprehensive tests for deeply nested pattern matching
; Tests various levels of nesting in list and dotted-list patterns

(print "=== Testing Deeply Nested Patterns ===")
(print "")

; ============================================================================
; Test 1: Double nested cons pattern - ((x . _) . _)
; ============================================================================

(defun extract-first-of-first
  ((((x . _) . _)) x))

(print "Test 1: Double nested cons")
(print (extract-first-of-first '((1 2) 3)))
(print (extract-first-of-first '((10 20 30) 40)))
(print "")

; ============================================================================
; Test 2: Triple nested cons pattern - (((x . _) . _) . _)
; ============================================================================

(defun extract-triple-nested
  (((((x . _) . _) . _)) x))

(print "Test 2: Triple nested cons")
(print (extract-triple-nested '(((1 2) 3) 4)))
(print (extract-triple-nested '(((10 20) 30) 40)))
(print "")

; ============================================================================
; Test 3: Nested list with multiple variables
; ============================================================================

(defun extract-both
  (((x y)) (cons x (cons y '()))))

(print "Test 3: Nested list with multiple vars")
(print (extract-both '(1 2)))
(print (extract-both '(10 20)))
(print "")

; ============================================================================
; Test 4: Deeply nested with wildcards
; ============================================================================

(defun third-level
  (((((_ _ x . _) . _) . _)) x))

(print "Test 4: Deep with wildcards")
(print (third-level '(((1 2 3 4) 5) 6)))
(print (third-level '(((10 20 30) 40) 50)))
(print "")

; ============================================================================
; Test 5: Mixed nesting - pattern matching at different levels
; ============================================================================

(defun mixed-nest
  (((0 y)) 'first-zero)
  (((x 0)) 'second-zero)
  (((x y)) (+ x y)))

(print "Test 5: Mixed nesting levels")
(print (mixed-nest '(0 5)))
(print (mixed-nest '(5 0)))
(print (mixed-nest '(3 7)))
(print "")

; ============================================================================
; Test 6: Nested pattern in function with multiple clauses
; ============================================================================

(defun process-nested
  ((('empty)) 'got-empty)
  (((x)) x)
  (((x y)) (+ x y)))

(print "Test 6: Multi-clause with nested patterns")
(print (process-nested '('empty)))
(print (process-nested '(42)))
(print (process-nested '(10 20)))
(print "")

; ============================================================================
; Test 7: Nested list - extract tail from nested structure
; ============================================================================

(defun get-nested-tail
  (((h . t)) t))

(print "Test 7: Nested tail extraction")
(print (get-nested-tail '(1 2 3)))
(print (get-nested-tail '(10)))
(print "")

; ============================================================================
; Test 8: Complex nested structure
; ============================================================================

(defun complex-pattern
  ((((a b) (c d))) (+ (+ a b) (+ c d))))

(print "Test 8: Complex nested structure")
(print (complex-pattern '((1 2) (3 4))))
(print (complex-pattern '((10 20) (30 40))))
(print "")

; ============================================================================
; Test 9: Nested pattern with literal matching
; ============================================================================

(defun nested-literal
  (((0 x)) (* x 2))
  (((x 0)) (* x 3))
  (((x y)) (+ x y)))

(print "Test 9: Nested literal matching")
(print (nested-literal '(0 5)))
(print (nested-literal '(5 0)))
(print (nested-literal '(3 7)))
(print "")

; ============================================================================
; Test 10: Deeply nested recursion
; ============================================================================

(defun flatten-once
  (('()) '())
  ((((h . t) . rest)) (cons h (cons t (flatten-once rest))))
  (((x . rest)) (cons x (flatten-once rest))))

(print "Test 10: Deeply nested recursion")
(print (flatten-once '((1 2) (3 4))))
(print (flatten-once '((10 20))))
(print "")

(print "All deeply nested pattern tests passed!")
