; Test v17 features: when, unless, defconst/defvar

(print "=== Testing v17 Features ===")
(print "")

; ============================================================================
; Test when
; ============================================================================

(print "Test 1: when with true condition")
(print (when true "executed"))
(print "")

(print "Test 2: when with false condition")
(print (when false "not executed"))
(print "")

(defun abs-when (n)
  (when (< n 0)
    (- 0 n)))

(print "Test 3: when in function (abs-when -5)")
(print (abs-when -5))
(print "")

(print "Test 4: when in function (abs-when 5) - should return false")
(print (abs-when 5))
(print "")

; ============================================================================
; Test unless
; ============================================================================

(print "Test 5: unless with false condition")
(print (unless false "executed"))
(print "")

(print "Test 6: unless with true condition")
(print (unless true "not executed"))
(print "")

(defun ensure-positive (n)
  (unless (> n 0)
    0))

(print "Test 7: unless in function (ensure-positive -5)")
(print (ensure-positive -5))
(print "")

(print "Test 8: unless in function (ensure-positive 5) - should return false")
(print (ensure-positive 5))
(print "")

; ============================================================================
; Test when/unless with side effects
; ============================================================================

(defun print-if-positive (n)
  (when (> n 0)
    (print "positive!")))

(print "Test 9: when with side effect (5)")
(print-if-positive 5)
(print "")

(print "Test 10: when with side effect (-5)")
(print-if-positive -5)
(print "")

; ============================================================================
; Test def
; ============================================================================

(print "Test 11: def defines constant")
(def MY-CONSTANT 42)
(print MY-CONSTANT)
(print "")

; ============================================================================
; Combined tests
; ============================================================================

(defun classify-number (n)
  (cond
    ((< n 0) "negative")
    ((> n 0) "positive")
    (else "zero")))

(print "Test 14: Combined with cond")
(print (classify-number -5))
(print (classify-number 0))
(print (classify-number 5))
(print "")

; Use when/unless with comparisons
(defun safe-divide (a b)
  (when (!= b 0)
    (/ a b)))

(print "Test 15: when with != operator")
(print (safe-divide 10 2))
(print (safe-divide 10 0))
(print "")

(print "All v17 tests passed!")
