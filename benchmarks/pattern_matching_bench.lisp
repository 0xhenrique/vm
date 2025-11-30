; Pattern matching performance benchmark
; Tests multi-arity and nested pattern matching correctness

(print "=== Pattern Matching Performance Test ===")
(print "")

; ============================================================================
; Test 1: Multi-arity function with heavy usage
; ============================================================================

(defun fib-multi
  ((0) 0)
  ((1) 1)
  ((n) (+ (fib-multi (- n 1)) (fib-multi (- n 2)))))

(print "Test 1: Multi-arity fibonacci (fib 15)")
(def result (fib-multi 15))
(print (string-append "Result: " (number->string result)))
(print "Expected: 610")
(print "")

; ============================================================================
; Test 2: Nested pattern matching with list operations
; ============================================================================

(defun sum-nested
  (('()) 0)
  ((((x) . rest)) (+ x (sum-nested rest))))

(defun range-helper
  ((start end acc)
    (if (> start end)
      acc
      (range-helper (+ start 1) end (cons (cons start '()) acc)))))

(defun range (n) (range-helper 1 n '()))

(print "Test 2: Nested pattern sum over range(100)")
(def test-list (range 100))
(def result2 (sum-nested test-list))
(print (string-append "Result: " (number->string result2)))
(print "Expected: 5050")
(print "")

; ============================================================================
; Test 3: Deep pattern matching in tight loop
; ============================================================================

(defun extract-triple
  (((((x . _) . _) . _)) x))

(defun loop-extract
  ((0 acc) acc)
  ((n acc) (loop-extract (- n 1) (+ acc (extract-triple '(((42 2) 3) 4))))))

(print "Test 3: Deep nested extraction (1000 iterations)")
(def result3 (loop-extract 1000 0))
(print (string-append "Result: " (number->string result3)))
(print "Expected: 42000")
(print "")

(print "=== All Tests Complete ===")

