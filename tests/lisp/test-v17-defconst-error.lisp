; Test that redefining a constant produces an error

(print "Defining constant...")
(defconst IMPORTANT-CONSTANT 42)
(print IMPORTANT-CONSTANT)

(print "Attempting to redefine constant (should fail)...")
(defconst IMPORTANT-CONSTANT 100)

(print "This should not be reached!")
