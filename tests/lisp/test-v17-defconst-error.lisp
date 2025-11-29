;; EXPECT-ERROR: compile
;; This test expects a compile error when attempting to redefine a constant
; Test that redefining a constant produces an error

(print "Defining constant...")
(defconst IMPORTANT-CONSTANT 42)
(print IMPORTANT-CONSTANT)

(print "Attempting to redefine constant (should fail)...")
(defconst IMPORTANT-CONSTANT 100)

(print "This should not be reached!")
