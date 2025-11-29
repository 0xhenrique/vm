;; ============================================================
;; Error Handling Examples - Errors as Values
;; ============================================================
;;
;; This demonstrates the error handling system based on
;; Rust/Go-style errors-as-values pattern.
;;
;; A Result is either:
;;   - (ok value)    - Success with a value
;;   - (err message) - Failure with an error message
;;
;; ============================================================

(load "stdlib.lisp")

;; Example 1: Creating results
(print "=== Example 1: Creating Results ===")
(print (ok 42))
(print (err "something failed"))
(print "")

;; Example 2: Basic error checking
(print "=== Example 2: Safe Division ===")
(defun safe-divide (x y)
  (if (== y 0)
      (err "division by zero")
      (ok (/ x y))))

(print (safe-divide 10 2))
(print (safe-divide 10 0))
(print "")

;; Example 3: Using predicates
(print "=== Example 3: Predicates ===")
(print (ok? (ok 42)))
(print (ok? (err "error")))
(print (err? (ok 42)))
(print (err? (err "error")))
(print "")

;; Example 4: Extracting values
(print "=== Example 4: Unwrapping ===")
(print (unwrap (ok 100)))
(print (unwrap-err (err "failed")))
(print "")

;; Example 5: Default values
(print "=== Example 5: unwrap-or ===")
(print (unwrap-or (ok 42) 0))
(print (unwrap-or (err "error") 0))
(print "")

;; Example 6: Transforming ok values
(print "=== Example 6: map-ok ===")
(defun double (x) (* x 2))
(print (map-ok double (ok 21)))
(print (map-ok double (err "error")))
(print "")

;; Example 7: Transforming error messages
(print "=== Example 7: map-err ===")
(defun add-prefix (msg) (string-append "ERROR: " msg))
(print (map-err add-prefix (err "failed")))
(print (map-err add-prefix (ok 42)))
(print "")

;; Example 8: Chaining operations
(print "=== Example 8: and-then ===")
(defun validate-positive (x)
  (if (> x 0)
      (ok x)
      (err "must be positive")))

(defun validate-even (x)
  (if (even? x)
      (ok x)
      (err "must be even")))

(print (and-then (ok 4) validate-even))
(print (and-then (ok 5) validate-even))
(print (and-then (err "previous error") validate-even))
(print "")

;; Example 9: Pipeline
(print "=== Example 9: Validation Pipeline ===")
(print (and-then (validate-positive 4) validate-even))
(print (and-then (validate-positive -4) validate-even))
(print (and-then (validate-positive 5) validate-even))
(print "")

;; Example 10: Error recovery
(print "=== Example 10: or-else ===")
(defun use-default (error-msg) (ok 0))
(print (or-else (ok 42) use-default))
(print (or-else (err "failed") use-default))
(print "")

(print "All examples completed!")
