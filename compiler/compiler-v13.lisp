; ============================================================================
; Lisp Self-Hosting Compiler - Version 13
; ============================================================================
; Adds support for function calls

; ============================================================================
; Global Constants
; ============================================================================

(defconst OP-PUSH 'push)
(defconst OP-ADD 'add)
(defconst OP-SUB 'sub)
(defconst OP-MUL 'mul)
(defconst OP-DIV 'div)
(defconst OP-EQ 'eq)
(defconst OP-JMP 'jmp)
(defconst OP-JMP-IF-FALSE 'jmp-if-false)
(defconst OP-GET-LOCAL 'get-local)
(defconst OP-CALL 'call)
(defconst OP-RET 'ret)

; ============================================================================
; Environment
; ============================================================================

(defun make-env (() '()))

(defun env-add ((env var) (cons var env)))

(defun env-lookup-impl
  ((var '() pos) -1)
  ((var (v . rest) pos)
    (if (== var v)
      pos
      (env-lookup-impl var rest (+ pos 1)))))

(defun env-lookup ((var env) (env-lookup-impl var env 0)))

; ============================================================================
; Helpers
; ============================================================================

(defun append-bytecode
  (('() bc2) bc2)
  (((h . t) bc2) (cons h (append-bytecode t bc2))))

(defun make-instr-1 ((op) (cons op '())))
(defun make-instr-2 ((op arg) (cons op (cons arg '()))))
(defun make-instr-3 ((op arg1 arg2) (cons op (cons arg1 (cons arg2 '())))))

(defun bytecode-length
  (('()) 0)
  (((h . t)) (+ 1 (bytecode-length t))))

(defun list-length
  (('()) 0)
  (((h . t)) (+ 1 (list-length t))))

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
    ; Check if it's a local variable
    (cons env
          (cons (make-instr-2 OP-GET-LOCAL (env-lookup name env)) '()))))

(defun compile-list
  (('() env)
    (cons env (cons (make-instr-2 'error '("Empty list")) '())))
  (((op . args) env)
    (if (== op 'let)
      (compile-let args env)
      (if (== op 'if)
        (compile-if args env)
        (if (== op '==)
          (compile-eq args env)
          (if (== op '+)
            (compile-add args env)
            (if (== op '-)
              (compile-sub args env)
              (if (== op '*)
                (compile-mul args env)
                (if (== op '/)
                  (compile-div args env)
                  ; Default: treat as function call
                  (compile-call op args env))))))))))

; ============================================================================
; Function Call Compilation
; ============================================================================

; Compile function call: (func-name arg1 arg2 ...)
; Bytecode: push each arg, then call instruction
(defun compile-call
  ((func-name args env)
    ; Compile all arguments
    (cons env
          (append-bytecode (compile-args args env)
                           (cons (make-instr-3 OP-CALL func-name (list-length args)) '())))))

; Compile a list of arguments (returns just bytecode, no env)
(defun compile-args
  (('() env) '())
  (((arg . rest) env)
    (append-bytecode (cdr (compile-expr arg env))
                     (compile-args rest env))))

; ============================================================================
; Let Compilation
; ============================================================================

(defun compile-let
  (((bindings body) env)
    (compile-let-helper bindings body env)))

(defun compile-let-helper
  (('() body env)
    (compile-expr body env))
  ((((var val) . rest) body env)
    (cons env
          (append-bytecode (cdr (compile-expr val env))
                           (cdr (compile-let-helper rest body (env-add env var)))))))

; ============================================================================
; Conditional Compilation
; ============================================================================

(defun compile-if
  (((cond-expr then-expr else-expr) env)
    (cons env
          (compile-if-bytecode (cdr (compile-expr cond-expr env))
                               (cdr (compile-expr then-expr env))
                               (cdr (compile-expr else-expr env))))))

(defun compile-if-bytecode
  ((cond-bc then-bc else-bc)
    (append-bytecode cond-bc
      (cons (make-instr-2 OP-JMP-IF-FALSE
                          (+ (bytecode-length cond-bc)
                             (+ 1 (+ (bytecode-length then-bc) 1))))
        (append-bytecode then-bc
          (cons (make-instr-2 OP-JMP
                              (+ (bytecode-length cond-bc)
                                 (+ 1 (+ (bytecode-length then-bc)
                                         (+ 1 (bytecode-length else-bc))))))
            else-bc))))))

; ============================================================================
; Comparison and Arithmetic
; ============================================================================

(defun compile-eq
  (((arg1 . rest) env)
    (if (== rest '())
      (cons env (cons (make-instr-2 'error '("== requires 2 arguments")) '()))
      (compile-binary-step2 (car rest) (compile-expr arg1 env) (make-instr-1 OP-EQ)))))

(defun compile-binary-step2
  ((arg2 r1 op)
    (cons (car r1)
          (append-bytecode (cdr r1)
                           (append-bytecode (cdr (compile-expr arg2 (car r1)))
                                            (cons op '()))))))

(defun compile-add
  (((arg1 . rest) env)
    (if (== rest '())
      (compile-expr arg1 env)
      (compile-binary-step2 (car rest) (compile-expr arg1 env) (make-instr-1 OP-ADD)))))

(defun compile-sub
  (((arg1 . rest) env)
    (if (== rest '())
      (cons env
            (append-bytecode (cdr (compile-expr arg1 env))
                             (cons (make-instr-1 'neg) '())))
      (compile-binary-step2 (car rest) (compile-expr arg1 env) (make-instr-1 OP-SUB)))))

(defun compile-mul
  (((arg1 . rest) env)
    (if (== rest '())
      (compile-expr arg1 env)
      (compile-binary-step2 (car rest) (compile-expr arg1 env) (make-instr-1 OP-MUL)))))

(defun compile-div
  (((arg1 . rest) env)
    (if (== rest '())
      (compile-expr arg1 env)
      (compile-binary-step2 (car rest) (compile-expr arg1 env) (make-instr-1 OP-DIV)))))

; ============================================================================
; Tests
; ============================================================================

(print "")
(print "=== Compiler v13 Tests ===")
(print "")

(print "Test 1: Basic arithmetic")
(print (cdr (compile-expr '(+ 1 2) (make-env))))
(print "")

(print "Test 2: Function call with no args")
(print (cdr (compile-expr '(foo) (make-env))))
(print "Expected: ((call foo 0))")
(print "")

(print "Test 3: Function call with one arg")
(print (cdr (compile-expr '(square 5) (make-env))))
(print "Expected: ((push 5) (call square 1))")
(print "")

(print "Test 4: Function call with two args")
(print (cdr (compile-expr '(add-nums 10 20) (make-env))))
(print "Expected: ((push 10) (push 20) (call add-nums 2))")
(print "")

(print "Test 5: Nested function calls")
(print (cdr (compile-expr '(add-nums (square 3) (square 4)) (make-env))))
(print "Expected: ((push 3) (call square 1) (push 4) (call square 1) (call add-nums 2))")
(print "")

(print "Test 6: Function call in let")
(print (cdr (compile-expr '(let ((x 5)) (square x)) (make-env))))
(print "Expected: ((push 5) (get-local 0) (call square 1))")
(print "")

(print "All tests complete!")
