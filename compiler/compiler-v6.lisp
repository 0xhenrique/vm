; ============================================================================
; Lisp Self-Hosting Compiler - Version 6
; ============================================================================
; Simplified version that avoids nested let issues
;
; Core design:
; - compile-* functions return (env . bytecode) pairs
; - Use helper functions to avoid deeply nested let statements

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
; Arithmetic - Binary Operations
; ============================================================================

; Helper: compile two expressions and combine with an operator
(defun compile-binary-op
  ((arg1 arg2 env op-instr)
    ; Note: We avoid nested let by directly constructing the result
    ; r1 = (compile-expr arg1 env)
    ; r2 = (compile-expr arg2 (car r1))
    ; result = (car r2 . (cdr r1) + (cdr r2) + op-instr)
    (cons (car (compile-expr arg2 (car (compile-expr arg1 env))))
          (append-bytecode (cdr (compile-expr arg1 env))
                           (append-bytecode (cdr (compile-expr arg2 (car (compile-expr arg1 env))))
                                            (cons op-instr '()))))))

; Wait, this approach has redundant compilation. Let me use a different strategy:
; Use intermediate results WITHOUT extracting via let

(defun compile-add
  (((arg1 . rest) env)
    (if (== rest '())
      (compile-expr arg1 env)
      ; For binary operations, use direct construction
      (cons (car (compile-expr (car rest) (car (compile-expr arg1 env))))
            (append-bytecode (cdr (compile-expr arg1 env))
                             (append-bytecode (cdr (compile-expr (car rest) (car (compile-expr arg1 env))))
                                              (cons (make-instr-1 OP-ADD) '())))))))

(defun compile-sub
  (((arg1 . rest) env)
    (if (== rest '())
      (cons env (append-bytecode (cdr (compile-expr arg1 env))
                                 (cons (make-instr-1 'neg) '())))
      (cons (car (compile-expr (car rest) (car (compile-expr arg1 env))))
            (append-bytecode (cdr (compile-expr arg1 env))
                             (append-bytecode (cdr (compile-expr (car rest) (car (compile-expr arg1 env))))
                                              (cons (make-instr-1 OP-SUB) '())))))))

(defun compile-mul
  (((arg1 . rest) env)
    (if (== rest '())
      (compile-expr arg1 env)
      (cons (car (compile-expr (car rest) (car (compile-expr arg1 env))))
            (append-bytecode (cdr (compile-expr arg1 env))
                             (append-bytecode (cdr (compile-expr (car rest) (car (compile-expr arg1 env))))
                                              (cons (make-instr-1 OP-MUL) '())))))))

(defun compile-div
  (((arg1 . rest) env)
    (if (== rest '())
      (compile-expr arg1 env)
      (cons (car (compile-expr (car rest) (car (compile-expr arg1 env))))
            (append-bytecode (cdr (compile-expr arg1 env))
                             (append-bytecode (cdr (compile-expr (car rest) (car (compile-expr arg1 env))))
                                              (cons (make-instr-1 OP-DIV) '())))))))

; ============================================================================
; Tests
; ============================================================================

(print "")
(print "=== Compiler v6 Tests ===")
(print "")

(print "Test 1: 42")
(print (cdr (compile-expr 42 (make-env))))
(print "")

(print "Test 2: (+ 1 2)")
(print (cdr (compile-expr '(+ 1 2) (make-env))))
(print "")

(print "Test 3: (+ 5 (+ 2 3))")
(print (cdr (compile-expr '(+ 5 (+ 2 3)) (make-env))))
(print "")
