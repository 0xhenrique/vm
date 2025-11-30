;; SKIP
;; Reason: Demonstration file - intentionally causes error to show stack trace
; Test full stack traces
; This should fail and show the complete call stack

(defun level3 (x)
  (/ x 0))  ; Division by zero

(defun level2 (x)
  (+ 1 (level3 x)))  ; Add 1 to prevent tail call optimization

(defun level1 (x)
  (+ 2 (level2 x)))  ; Add 2 to prevent tail call optimization

(defun main ()
  (+ 3 (level1 42)))  ; Add 3 to prevent tail call optimization

(print "Testing stack traces...")
(main)
(print "This should not print")
