; Test global variables with comments!

; Define some constants
(defconst PI 3)
(defconst MAX-VALUE 100)

; Define some variables
(defvar my-name "henrique")
(defvar counter 0)

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
(defvar counter (+ counter 5))
(print counter)
