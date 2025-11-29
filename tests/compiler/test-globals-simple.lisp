; Simple test of global variables with comments!

; Define constants
(defconst PI 3)
(defconst MAX 100)

; Define variables
(defvar age 35)

; Test access
(print PI)
(print MAX)
(print age)

; Test in expressions
(print (+ PI 1))
(print (+ age 10))

; Test from function
(defun get-pi (() PI))
(print (get-pi))

; Test from function with param
(defun add-to-age ((n) (+ age n)))
(print (add-to-age 5))
