;; SKIP
;; Reason: Demonstration file - intentionally causes error to show stack trace
; Test stack traces with deep recursion
; This will show the full call stack in recursive functions

(defun countdown
  ((0) (/ 42 0))  ; Division by zero at base case
  ((n) (+ 1 (countdown (- n 1)))))  ; Prevent tail call optimization

(print "Testing stack trace with deep recursion...")
(countdown 5)
(print "This should not print")
