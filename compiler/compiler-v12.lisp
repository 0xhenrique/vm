; ============================================================================
; Lisp Self-Hosting Compiler - Version 12
; ============================================================================
; Simplified let implementation without Slide (for now)

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

; ============================================================================
; Environment: Simple list of variable names
; ============================================================================
; Environment is just a list of variable names in stack order
; To look up a variable, find its position in the list

(defun make-env (() '()))

; Add a variable to the environment
(defun env-add
  ((env var)
    (cons var env)))

; Look up a variable and return its stack position
; Stack grows upward, so position 0 is the most recent binding
(defun env-lookup-impl
  ((var '() pos) -1)
  ((var (v . rest) pos)
    (if (== var v)
      pos
      (env-lookup-impl var rest (+ pos 1)))))

(defun env-lookup
  ((var env)
    (env-lookup-impl var env 0)))

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
    ; Look up variable position
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
; Let Compilation (Simplified - no Slide)
; ============================================================================

; Compile let: (let ((var val) ...) body)
(defun compile-let
  (((bindings body) env)
    (compile-let-helper bindings body env)))

(defun compile-let-helper
  (('() body env)
    ; No more bindings, compile body
    (compile-expr body env))
  ((((var val) . rest) body env)
    ; Strategy: compile val, then continue with var added to env    ; This means we compile val with current env
    ; Then compile rest with var added
    ; Bytecode: val-code + rest-code
    (cons env  ; Return original env (let doesn't modify outer scope)
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
(print "=== Compiler v12 Tests ===")
(print "")

(print "Test 1: 42")
(print (cdr (compile-expr 42 (make-env))))
(print "")

(print "Test 2: (+ 1 2)")
(print (cdr (compile-expr '(+ 1 2) (make-env))))
(print "")

(print "Test 3: (if true 10 20)")
(print (cdr (compile-expr '(if true 10 20) (make-env))))
(print "")

(print "Test 4: (let ((x 10)) x)")
(print (cdr (compile-expr '(let ((x 10)) x) (make-env))))
(print "Expected: (push 10) (get-local 0)")
(print "")

(print "Test 5: (let ((x 5) (y 10)) (+ x y))")
(print (cdr (compile-expr '(let ((x 5) (y 10)) (+ x y)) (make-env))))
(print "Expected: (push 5) (push 10) (get-local 1) (get-local 0) (add)")
(print "Note: y is at pos 0 (top of stack), x is at pos 1")
(print "")

(print "All tests complete!")
