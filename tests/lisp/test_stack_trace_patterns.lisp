;; SKIP
;; Reason: Demonstration file - intentionally causes error to show stack trace
; Test stack traces with pattern matching
; This should show stack traces work with pattern-matched functions

(defun factorial
  ((0) 1)
  ((n) (* n (factorial (- n 1)))))

(defun process-list
  (('()) 0)
  (((h . t)) (+ h (process-list t))))

(defun deep-nested
  ((((x))) (factorial x)))

; This will cause a type error in factorial when it tries to multiply with a string
(print "Testing stack trace with pattern matching...")
(deep-nested '(("not a number")))
(print "This should not print")
