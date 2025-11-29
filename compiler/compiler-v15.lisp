; ============================================================================
; Lisp Self-Hosting Compiler - Version 15
; ============================================================================
; Adds pattern matching support for multi-clause defun
;
; Supports:
; - Number literal patterns: (defun foo ((0) ...) ((n) ...))
; - Variable patterns: always match and bind
; - Multiple clauses with fallthrough

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

; Get first element
(defun first ((x) (car x)))

; ============================================================================
; Expression Compiler
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
; Let, If, Arithmetic (same as v14)
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
; Pattern Matching for Defun
; ============================================================================

; Compile multi-clause defun: (defun name ((pattern) body) ((pattern) body) ...)
; Returns: (name . bytecode)
(defun compile-defun-multi
  ((name clauses)
    (cons name
          (compile-all-clauses clauses))))

; Compile all clauses into flat bytecode
; Strategy: compile each clause, compute sizes, patch jump addresses
(defun compile-all-clauses
  ((clauses)
    (compile-clauses-list clauses)))

; Compile list of clauses
(defun compile-clauses-list (clauses)
  (if (== (cdr clauses) '())
    ; Single clause - no pattern test needed
    (append-bytecode (cdr (compile-expr (get-clause-body (car clauses))
                                        (pattern-to-env (get-clause-patterns (car clauses)) 0)))
                     (cons (make-instr-1 OP-RET) '()))
    ; Multiple clauses - need pattern test
    (compile-clause-with-next (car clauses) (cdr clauses))))

; Compile a single clause with fallthrough to rest
(defun compile-clause-with-next (clause rest)
  (if (symbol? (get-first-pattern (get-clause-patterns clause)))
    ; Variable pattern - always matches, compile body and stop
    (append-bytecode (cdr (compile-expr (get-clause-body clause)
                                        (cons (get-first-pattern (get-clause-patterns clause)) '())))
                     (cons (make-instr-1 OP-RET) '()))
    ; Literal pattern - compile test, body with RET, then rest
    (append-bytecode (compile-pattern-match (get-first-pattern (get-clause-patterns clause)))
                     (append-bytecode (cdr (compile-expr (get-clause-body clause) '()))
                                      (append-bytecode (cons (make-instr-1 OP-RET) '())
                                                       (compile-clauses-list rest))))))

; Compile pattern match test (without jump address yet)
(defun compile-pattern-match
  ((pattern)
    (cons (make-instr-2 OP-LOAD-ARG 0)
          (cons (make-instr-2 OP-PUSH pattern)
                (cons (make-instr-1 OP-EQ)
                      (cons (make-instr-2 OP-JMP-IF-FALSE 6) '()))))))

; Extract pattern list from clause: ((patterns...) body) → (patterns...)
(defun get-clause-patterns
  (((patterns body)) patterns))

; Extract body from clause
(defun get-clause-body
  (((patterns body)) body))

; Get first pattern from pattern list (for single-arg functions)
(defun get-first-pattern
  ((patterns) (car patterns)))

; Convert pattern list to environment (for single-arg functions)
(defun pattern-to-env
  ((patterns arg-idx)
    ; Get first pattern
    (if (symbol? (car patterns))
      (cons (car patterns) '())
      '())))

; ============================================================================
; Program Compilation
; ============================================================================

(defun compile-program
  ((exprs)
    (cons (extract-and-compile-defuns exprs)
          (compile-main-exprs exprs))))

(defun extract-and-compile-defuns
  (('()) '())
  (((expr . rest))
    (if (is-defun expr)
      (cons (compile-defun-form expr)
            (extract-and-compile-defuns rest))
      (extract-and-compile-defuns rest))))

(defun compile-main-exprs
  (('()) '())
  (((expr . rest))
    (if (is-defun expr)
      (compile-main-exprs rest)
      (append-bytecode (cdr (compile-expr expr (make-env)))
                       (compile-main-exprs rest)))))

(defun is-defun
  ((expr)
    (if (list? expr)
      (if (== (car expr) 'defun)
        true
        false)
      false)))

; Determine if defun is old-style or new-style and compile accordingly
(defun compile-defun-form
  (((defun-kw name . clauses))
    ; Check if first clause is old-style: (params)
    (if (is-old-style-params (car clauses))
      ; Old style: (defun name (params) body)
      (compile-defun-old name (car clauses) (car (cdr clauses)))
      ; New style: (defun name ((pattern) body) ...)
      (compile-defun-multi name clauses))))

; Check if it's old-style parameter list vs new-style pattern
(defun is-old-style-params
  ((clause)
    ; Old style if it's a list of symbols (not a nested list)
    ; New style if it's ((pattern) body)
    ; For now: old style if not a list, or first element is symbol
    (if (list? clause)
      (if (list? (car clause))
        false  ; New style: ((pattern) ...)
        true)  ; Old style: (x y z)
      false)))

; Compile old-style defun
(defun compile-defun-old
  ((name params body)
    (cons name
          (append-bytecode (cdr (compile-expr body (params-to-env params)))
                           (cons (make-instr-1 OP-RET) '())))))

(defun params-to-env
  (('()) '())
  (((p . rest)) (cons p (params-to-env rest))))

; ============================================================================
; Tests
; ============================================================================

(print "")
(print "=== Compiler v15 Tests ===")
(print "")

(print "Test 1: Old-style defun")
(print (compile-defun-form '(defun square (x) (* x x))))
(print "")

(print "Test 2: New-style defun with number pattern")
(print (compile-defun-form '(defun is-zero ((0) true) ((n) false))))
(print "Expected: pattern test for 0, then true, ret, then false, ret")
(print "")

(print "Test 3: Factorial with pattern matching")
(print (compile-defun-form '(defun fact ((0) 1) ((n) (* n (fact (- n 1)))))))
(print "")

(print "All tests complete!")
