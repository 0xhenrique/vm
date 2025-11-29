; ============================================================================
; Lisp Self-Hosting Compiler - Version 14
; ============================================================================
; Adds support for defun (function definitions)
; Two-pass compilation: first collect defuns, then compile main code

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
(defconst OP-LOAD-ARG 'load-arg)
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

; Create environment from parameter list
(defun params-to-env
  (('()) '())
  (((p . rest)) (cons p (params-to-env rest))))

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
; Expression Compiler (same as v13 but uses load-arg for function params)
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
    ; Look up in environment - returns position if found, -1 if not
    (cons env
          (cons (make-instr-2 OP-LOAD-ARG (env-lookup name env)) '()))))

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
                  (compile-call op args env))))))))))

; ============================================================================
; Function Call Compilation
; ============================================================================

(defun compile-call
  ((func-name args env)
    (cons env
          (append-bytecode (compile-args args env)
                           (cons (make-instr-3 OP-CALL func-name (list-length args)) '())))))

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
; Arithmetic and Comparison
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
; Defun Compilation
; ============================================================================

; Compile a single defun: (defun name (params) body)
; Returns: (name . bytecode)
(defun compile-defun
  (((name params body))
    ; Create environment from parameters
    ; Compile body in that environment
    ; Add RET instruction at the end
    (cons name
          (append-bytecode (cdr (compile-expr body (params-to-env params)))
                           (cons (make-instr-1 OP-RET) '())))))

; ============================================================================
; Program Compilation (Two-Pass)
; ============================================================================

; compile-program: takes a list of top-level expressions
; Returns: ((functions . main-code))
; Where functions is a list of (name . bytecode) pairs

(defun compile-program
  ((exprs)
    ; First pass: extract and compile all defuns
    ; Second pass: compile remaining expressions as main code
    (cons (extract-and-compile-defuns exprs)
          (compile-main-exprs exprs))))

; Extract all defun forms and compile them
(defun extract-and-compile-defuns
  (('()) '())
  (((expr . rest))
    (if (is-defun expr)
      (cons (compile-defun (get-defun-parts expr))
            (extract-and-compile-defuns rest))
      (extract-and-compile-defuns rest))))

; Compile all non-defun expressions as main code
(defun compile-main-exprs
  (('()) '())
  (((expr . rest))
    (if (is-defun expr)
      (compile-main-exprs rest)
      (append-bytecode (cdr (compile-expr expr (make-env)))
                       (compile-main-exprs rest)))))

; Check if expression is a defun
(defun is-defun
  ((expr)
    (if (list? expr)
      (if (== (car expr) 'defun)
        true
        false)
      false)))

; Extract defun parts: (defun name (params) body) -> (name params body)
(defun get-defun-parts
  (((defun-kw name params body))
    (cons name (cons params (cons body '())))))

; ============================================================================
; Tests
; ============================================================================

(print "")
(print "=== Compiler v14 Tests ===")
(print "")

(print "Test 1: Simple defun")
(print (compile-defun '(square (x) (* x x))))
(print "Expected: (square . bytecode-with-ret)")
(print "")

(print "Test 2: Defun with multiple params")
(print (compile-defun '(add-nums (a b) (+ a b))))
(print "Expected: (add-nums . bytecode-with-ret)")
(print "")

(print "Test 3: Full program with defun and main code")
(print (compile-program '((defun square (x) (* x x))
                          (defun add (a b) (+ a b))
                          (square 5)
                          (add 10 20))))
(print "Expected: ((functions) . main-bytecode)")
(print "")

(print "All tests complete!")
