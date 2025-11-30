;; SKIP
;; Reason: Demonstration file - intentionally causes errors to show suggestions
; This file demonstrates the improved error messages with suggestions

; Test 1: Undefined variable with similar name suggestion
(defun test-typo (my-variable)
  ; This should suggest 'my-variable' when we mistype it
  (print my-varaible))

; Test 2: Empty list error
(defun test-empty ()
  ; This should suggest using 'quote' for empty lists
  ())

; Test 3: Arithmetic arity error
(defun test-arity ()
  ; This should suggest the correct usage of +
  (+ 5))

; Test 4: Division by zero
(defun test-divide-by-zero (x)
  ; This should suggest checking divisor before dividing
  (/ x 0))

; Test 5: Dotted list in expression
(defun test-dotted ()
  ; This should suggest using cons instead
  (def x (1 . 2)))

(print "Testing error suggestions...")
(test-typo 42)
