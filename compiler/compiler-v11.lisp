; ============================================================================
; Lisp Self-Hosting Compiler - Version 11
; ============================================================================
; Adds support for let bindings with GetLocal and Slide instructions

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
(defconst OP-SLIDE 'slide)

; ============================================================================
; Environment Helpers
; ============================================================================

; Environment: ((bindings . ((name . depth) ...)) (depth . N))
; - bindings: map from variable names to stack depths
; - depth: current stack depth

(defun make-env (() (cons '() 0)))

(defun env-depth ((env) (cdr env)))

(defun env-bindings ((env) (car env)))

; Add a binding to the environment
(defun env-add-binding
  ((env name)
    (cons (cons (cons name (env-depth env)) (env-bindings env))
          (+ (env-depth env) 1))))

; Look up a variable in the environment
; Returns the stack depth, or -1 if not found
(defun env-lookup-impl
  ((name '()) -1)
  ((name ((var . depth) . rest))
    (if (== name var)
      depth
      (env-lookup-impl name rest))))

(defun env-lookup
  ((name env)
    (env-lookup-impl name (env-bindings env))))

; ============================================================================
; Helpers
; ============================================================================

(defun append-bytecode
  (('() bc2) bc2)
  (((h . t) bc2) (cons h (append-bytecode t bc2))))

(defun make-instr-1
  ((op) (cons op '())))

(defun make-instr-2
  ((op arg) (cons op (cons arg '()))))

(defun bytecode-length
  (('()) 0)
  (((h . t)) (+ 1 (bytecode-length t))))

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
    ; Look up variable in environment
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
                  (cons env (cons (make-instr-2 'error (cons "Unknown operation" (cons op '()))) '())))))))))))

; ============================================================================
; Let Compilation
; ============================================================================

; Compile let: (let ((var val) ...) body)
; Strategy: compile bindings recursively, building up environment
(defun compile-let
  (((bindings body) env)
    ; Count bindings first
    (compile-let-with-count bindings body env (count-bindings bindings))))

; Count number of bindings
(defun count-bindings
  (('()) 0)
  (((h . t)) (+ 1 (count-bindings t))))

; Compile let with known binding count
(defun compile-let-with-count
  ((bindings body env count)
    ; Compile bindings to build environment and generate bytecode
    (cons (car (compile-let-bindings bindings body env))
          (if (== count 0)
            (cdr (compile-let-bindings bindings body env))
            (append-bytecode (cdr (compile-let-bindings bindings body env))
                             (cons (make-instr-2 OP-SLIDE count) '()))))))

; Compile bindings: returns (final-env . bytecode)
(defun compile-let-bindings
  (('() body env)
    ; No more bindings, compile body
    (compile-expr body env))
  ((((var val) . rest) body env)
    ; Compile value, then rest with updated env
    (cons (car (compile-let-bindings rest body (env-add-binding (car (compile-expr val env)) var)))
          (append-bytecode (cdr (compile-expr val env))
                           (cdr (compile-let-bindings rest body (env-add-binding (car (compile-expr val env)) var)))))))

; ============================================================================
; Conditional Compilation
; ============================================================================

(defun compile-if
  (((cond-expr then-expr else-expr) env)
    (cons (car (compile-expr else-expr (car (compile-expr then-expr (car (compile-expr cond-expr env))))))
          (compile-if-bytecode (cdr (compile-expr cond-expr env))
                               (cdr (compile-expr then-expr (car (compile-expr cond-expr env))))
                               (cdr (compile-expr else-expr (car (compile-expr then-expr (car (compile-expr cond-expr env))))))))))

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
    (cons (car (compile-expr arg2 (car r1)))
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
      (cons (car (compile-expr arg1 env))
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
(print "=== Compiler v11 Tests ===")
(print "")

(print "Test 1: 42")
(print (cdr (compile-expr 42 (make-env))))
(print "")

(print "Test 2: (+ 1 2)")
(print (cdr (compile-expr '(+ 1 2) (make-env))))
(print "")

(print "Test 3: (let ((x 10)) x)")
(print (cdr (compile-expr '(let ((x 10)) x) (make-env))))
(print "Expected: (push 10) (get-local 0) (slide 1)")
(print "")

(print "Test 4: (let ((x 5) (y 10)) (+ x y))")
(print (cdr (compile-expr '(let ((x 5) (y 10)) (+ x y)) (make-env))))
(print "Expected: (push 5) (push 10) (get-local 0) (get-local 1) (add) (slide 2)")
(print "")

(print "Test 5: (if (== 1 1) (let ((x 100)) x) 200)")
(print (cdr (compile-expr '(if (== 1 1) (let ((x 100)) x) 200) (make-env))))
(print "")

(print "All tests complete!")
