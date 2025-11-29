; Minimal test to debug the compiler

; Load the necessary pieces
(defconst OP-PUSH 'push)
(defconst OP-ADD 'add)

(defun make-env (() '()))

(defun make-instr-1
  ((op) (cons op '())))

(defun make-instr-2
  ((op arg) (cons op (cons arg '()))))

(defun append-bytecode
  (('() bc2) bc2)
  (((h . t) bc2) (cons h (append-bytecode t bc2))))

; Simple compile-literal
(defun compile-literal
  ((value env)
    (cons env (cons (make-instr-2 OP-PUSH value) '()))))

; Test compile-literal directly
(print "Test: compile-literal 42")
(let ((result (compile-literal 42 (make-env))))
  (print result))

(print "")
(print "Test: extract env and bytecode")
(let ((r1 (compile-literal 1 (make-env))))
  (let ((env1 (car r1)))
    (let ((bc1 (cdr r1)))
      (print bc1))))
