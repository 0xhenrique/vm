; ============================================================================
; Lisp Self-Hosting Compiler - Version 8
; ============================================================================
; Uses helper functions to reduce nesting depth and avoid VM stack issues

; ============================================================================
; Global Constants
; ============================================================================

(defconst OP-PUSH 'push)
(defconst OP-ADD 'add)
(defconst OP-SUB 'sub)
(defconst OP-MUL 'mul)
(defconst OP-DIV 'div)

; ============================================================================
; Helpers
; ============================================================================

(defun make-env (() '()))

(defun append-bytecode
  (('() bc2) bc2)
  (((h . t) bc2) (cons h (append-bytecode t bc2))))

(defun make-instr-1
  ((op) (cons op '())))

(defun make-instr-2
  ((op arg) (cons op (cons arg '()))))

; Result accessors
(defun result-env ((r) (car r)))
(defun result-bytecode ((r) (cdr r)))

; ============================================================================
; Core Compiler
; ============================================================================

(defun compile-expr
  ((expr env)
    (if (list? expr)
      (compile-list expr env)
      (if (symbol? expr)
        (compile-symbol expr env)
        (compile-literal expr env)))))

(defun compile-literal
  ((value env)
    (cons env (cons (make-instr-2 OP-PUSH value) '()))))

(defun compile-symbol
  ((name env)
    (cons env (cons (make-instr-2 'error (cons "Undefined variable" (cons name '()))) '()))))

(defun compile-list
  (('() env)
    (cons env (cons (make-instr-2 'error '("Empty list")) '())))
  (((op . args) env)
    (if (== op '+)
      (compile-add args env)
      (if (== op '-)
        (compile-sub args env)
        (if (== op '*)
          (compile-mul args env)
          (if (== op '/)
            (compile-div args env)
            (cons env (cons (make-instr-2 'error (cons "Unknown operation" (cons op '()))) '()))))))))

; ============================================================================
; Binary Operation Helpers
; ============================================================================

; Helper to build result from two compiled sub-expressions
(defun make-binary-result
  ((r1 r2 op)
    (cons (result-env r2)
          (append-bytecode (result-bytecode r1)
                           (append-bytecode (result-bytecode r2)
                                            (cons op '()))))))

; Compile two arguments in sequence
(defun compile-two-args
  ((arg1 arg2 env)
    (cons (compile-expr arg1 env)
          (compile-expr arg2 (result-env (compile-expr arg1 env))))))

; ============================================================================
; Arithmetic Operations
; ============================================================================

(defun compile-add
  (((arg1 . rest) env)
    (if (== rest '())
      (compile-expr arg1 env)
      (make-binary-result (compile-expr arg1 env)
                          (compile-expr (car rest) (result-env (compile-expr arg1 env)))
                          (make-instr-1 OP-ADD)))))

(defun compile-sub
  (((arg1 . rest) env)
    (if (== rest '())
      (cons (result-env (compile-expr arg1 env))
            (append-bytecode (result-bytecode (compile-expr arg1 env))
                             (cons (make-instr-1 'neg) '())))
      (make-binary-result (compile-expr arg1 env)
                          (compile-expr (car rest) (result-env (compile-expr arg1 env)))
                          (make-instr-1 OP-SUB)))))

(defun compile-mul
  (((arg1 . rest) env)
    (if (== rest '())
      (compile-expr arg1 env)
      (make-binary-result (compile-expr arg1 env)
                          (compile-expr (car rest) (result-env (compile-expr arg1 env)))
                          (make-instr-1 OP-MUL)))))

(defun compile-div
  (((arg1 . rest) env)
    (if (== rest '())
      (compile-expr arg1 env)
      (make-binary-result (compile-expr arg1 env)
                          (compile-expr (car rest) (result-env (compile-expr arg1 env)))
                          (make-instr-1 OP-DIV)))))

; ============================================================================
; Tests
; ============================================================================

(print "")
(print "=== Compiler v8 Tests ===")
(print "")

(print "Test 1: 42")
(print (result-bytecode (compile-expr 42 (make-env))))
(print "")

(print "Test 2: (+ 1 2)")
(print (result-bytecode (compile-expr '(+ 1 2) (make-env))))
(print "")

(print "Test 3: (+ 5 (+ 2 3))")
(print (result-bytecode (compile-expr '(+ 5 (+ 2 3)) (make-env))))
(print "")
