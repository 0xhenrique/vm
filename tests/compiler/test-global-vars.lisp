; Test global variables with comments!

; Define some constants
(def PI 3)
(def MAX-VALUE 100)

; Define some variables
(def my-name "henrique")
(def counter 0)

; Test: Can we access them?
(print PI)
(print MAX-VALUE)
(print my-name)
(print counter)

; Test: Use in expressions
(print (+ PI 1))
(print (* MAX-VALUE 2))

; Test: Use in functions
(defun double-counter
  (() (* counter 2)))

(print (double-counter))

; Test: Modify a defvar (this should work)
(def counter (+ counter 5))
(print counter)
