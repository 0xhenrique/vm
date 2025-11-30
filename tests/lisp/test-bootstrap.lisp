; Bootstrap test: Use the Lisp compiler to compile a simple program
; This proves the compiler can run and produce bytecode

; The compiler is already loaded in bytecode form
; It has these functions available:
; - tokenize (from tokenizer)
; - parse (from parser)
; - compile (from bytecode generator)

; Test: Compile a simple arithmetic expression
(print "=== Bootstrap Test ===")
(print "")

(print "Test 1: Compiling (+ 1 2)")
(let ((source "(+ 1 2)"))
  (let ((tokens (tokenize source)))
    (let ((ast (parse tokens)))
      (let ((bytecode (compile ast)))
        (print bytecode)))))

(print "")
(print "Test 2: Compiling (defun add (x y) (+ x y))")
(let ((source "(defun add (x y) (+ x y))"))
  (let ((tokens (tokenize source)))
    (let ((ast (parse tokens)))
      (let ((bytecode (compile ast)))
        (print bytecode)))))

(print "")
(print "=== Bootstrap Success! ===")
