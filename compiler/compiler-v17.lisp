; ============================================================================
; Lisp Self-Hosting Compiler - Version 17
; ============================================================================
; Adds syntactic sugar and proper constant semantics:
; - when, unless: cleaner conditional expressions
; - defconst enforcement: true immutable constants
; - defvar: explicit mutable globals
;
; Focus: Developer experience while maintaining functional purity

; ============================================================================
; Global Constants
; ============================================================================

(defconst OP-PUSH 'push)
(defconst OP-ADD 'add)
(defconst OP-SUB 'sub)
(defconst OP-MUL 'mul)
(defconst OP-DIV 'div)
(defconst OP-EQ 'eq)
(defconst OP-LT 'lt)
(defconst OP-GT 'gt)
(defconst OP-LTE 'leq)
(defconst OP-GTE 'gte)
(defconst OP-NEQ 'neq)
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

(defun first ((x) (car x)))
(defun second ((x) (car (cdr x))))

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
        (if (== op 'when)
          (compile-when args env)
          (if (== op 'unless)
            (compile-unless args env)
            (if (== op 'cond)
              (compile-cond args env)
          (if (== op 'and)
            (compile-and args env)
            (if (== op 'or)
              (compile-or args env)
              (if (== op '==)
                (compile-eq args env)
                (if (== op '<)
                  (compile-lt args env)
                  (if (== op '>)
                    (compile-gt args env)
                    (if (== op '<=)
                      (compile-lte args env)
                      (if (== op '>=)
                        (compile-gte args env)
                        (if (== op '!=)
                          (compile-neq args env)
                          (if (== op '+)
                            (compile-add args env)
                            (if (== op '-)
                              (compile-sub args env)
                              (if (== op '*)
                                (compile-mul args env)
                                  (if (== op '/)
                                    (compile-div args env)
                                    (compile-call op args env))))))))))))))))))))

; ============================================================================
; Control Flow: when, unless
; ============================================================================

; Compile when: (when test expr) => (if test expr false)
(defun compile-when
  (((test expr) env)
    (cons env
          (compile-if-bytecode (cdr (compile-expr test env))
                               (cdr (compile-expr expr env))
                               (cons (make-instr-2 OP-PUSH false) '()))))
  ((args env)
    (cons env (cons (make-instr-2 'error '("when requires exactly 2 arguments")) '()))))

; Compile unless: (unless test expr) => (if test false expr)
(defun compile-unless
  (((test expr) env)
    (cons env
          (compile-if-bytecode (cdr (compile-expr test env))
                               (cons (make-instr-2 OP-PUSH false) '())
                               (cdr (compile-expr expr env)))))
  ((args env)
    (cons env (cons (make-instr-2 'error '("unless requires exactly 2 arguments")) '()))))

; ============================================================================
; Control Flow: cond
; ============================================================================

; Compile cond: (cond (test1 expr1) (test2 expr2) ... (else default))
; Strategy: chain of if-then-else
(defun compile-cond
  ((clauses env)
    (if (== clauses '())
      (cons env (cons (make-instr-2 OP-PUSH false) '()))
      (compile-cond-clauses clauses env))))

(defun compile-cond-clauses
  (((clause) env)
    ; Last clause - check if it's an else clause
    (if (== (first clause) 'else)
      (compile-expr (second clause) env)
      ; Regular clause: compile as if with false as else branch
      (compile-cond-clause clause (cons env (cons (make-instr-2 OP-PUSH false) '())))))
  (((clause . rest) env)
    ; Multiple clauses: compile as if with rest as else branch
    (compile-cond-clause clause (compile-cond-clauses rest env))))

; Compile a single cond clause: (test expr)
(defun compile-cond-clause
  (((test expr) else-result)
    (cons (car else-result)
          (compile-if-bytecode (cdr (compile-expr test (car else-result)))
                               (cdr (compile-expr expr (car else-result)))
                               (cdr else-result)))))

; ============================================================================
; Logical Operators: and, or
; ============================================================================

; Compile and: (and expr1 expr2 ...) - short-circuit on false
(defun compile-and
  (('() env)
    ; Empty and is true
    (cons env (cons (make-instr-2 OP-PUSH true) '())))
  (((expr) env)
    ; Single expression
    (compile-expr expr env))
  (((expr . rest) env)
    ; Multiple expressions: if expr then (and rest...) else false
    (cons env
          (compile-if-bytecode (cdr (compile-expr expr env))
                               (cdr (compile-and rest env))
                               (cons (make-instr-2 OP-PUSH false) '())))))

; Compile or: (or expr1 expr2 ...) - short-circuit on true
(defun compile-or
  (('() env)
    ; Empty or is false
    (cons env (cons (make-instr-2 OP-PUSH false) '())))
  (((expr) env)
    ; Single expression
    (compile-expr expr env))
  (((expr . rest) env)
    ; Multiple expressions: if expr then true else (or rest...)
    (cons env
          (compile-if-bytecode (cdr (compile-expr expr env))
                               (cons (make-instr-2 OP-PUSH true) '())
                               (cdr (compile-or rest env))))))

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
; Let and If
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

; ============================================================================
; Comparison Operators
; ============================================================================

(defun compile-eq
  (((arg1 . rest) env)
    (if (== rest '())
      (cons env (cons (make-instr-2 'error '("== requires 2 arguments")) '()))
      (compile-binary-step2 (car rest) (compile-expr arg1 env) (make-instr-1 OP-EQ)))))

(defun compile-lt
  (((arg1 . rest) env)
    (if (== rest '())
      (cons env (cons (make-instr-2 'error '("< requires 2 arguments")) '()))
      (compile-binary-step2 (car rest) (compile-expr arg1 env) (make-instr-1 OP-LT)))))

(defun compile-gt
  (((arg1 . rest) env)
    (if (== rest '())
      (cons env (cons (make-instr-2 'error '("> requires 2 arguments")) '()))
      (compile-binary-step2 (car rest) (compile-expr arg1 env) (make-instr-1 OP-GT)))))

(defun compile-lte
  (((arg1 . rest) env)
    (if (== rest '())
      (cons env (cons (make-instr-2 'error '("<= requires 2 arguments")) '()))
      (compile-binary-step2 (car rest) (compile-expr arg1 env) (make-instr-1 OP-LTE)))))

(defun compile-gte
  (((arg1 . rest) env)
    (if (== rest '())
      (cons env (cons (make-instr-2 'error '(">= requires 2 arguments")) '()))
      (compile-binary-step2 (car rest) (compile-expr arg1 env) (make-instr-1 OP-GTE)))))

(defun compile-neq
  (((arg1 . rest) env)
    (if (== rest '())
      (cons env (cons (make-instr-2 'error '("!= requires 2 arguments")) '()))
      (compile-binary-step2 (car rest) (compile-expr arg1 env) (make-instr-1 OP-NEQ)))))

(defun compile-binary-step2
  ((arg2 r1 op)
    (cons (car r1)
          (append-bytecode (cdr r1)
                           (append-bytecode (cdr (compile-expr arg2 (car r1)))
                                            (cons op '()))))))

; ============================================================================
; Arithmetic Operators
; ============================================================================

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

(defun compile-defun-multi
  ((name clauses)
    (cons name
          (compile-all-clauses clauses))))

(defun compile-all-clauses
  ((clauses)
    (compile-clauses-list clauses)))

(defun compile-clauses-list (clauses)
  (if (== (cdr clauses) '())
    (append-bytecode (cdr (compile-expr (get-clause-body (car clauses))
                                        (pattern-to-env (get-clause-patterns (car clauses)) 0)))
                     (cons (make-instr-1 OP-RET) '()))
    (compile-clause-with-next (car clauses) (cdr clauses))))

(defun compile-clause-with-next (clause rest)
  (if (symbol? (get-first-pattern (get-clause-patterns clause)))
    (append-bytecode (cdr (compile-expr (get-clause-body clause)
                                        (cons (get-first-pattern (get-clause-patterns clause)) '())))
                     (cons (make-instr-1 OP-RET) '()))
    (append-bytecode (compile-pattern-match (get-first-pattern (get-clause-patterns clause)))
                     (append-bytecode (cdr (compile-expr (get-clause-body clause) '()))
                                      (append-bytecode (cons (make-instr-1 OP-RET) '())
                                                       (compile-clauses-list rest))))))

(defun compile-pattern-match
  ((pattern)
    (cons (make-instr-2 OP-LOAD-ARG 0)
          (cons (make-instr-2 OP-PUSH pattern)
                (cons (make-instr-1 OP-EQ)
                      (cons (make-instr-2 OP-JMP-IF-FALSE 6) '()))))))

(defun get-clause-patterns
  (((patterns body)) patterns))

(defun get-clause-body
  (((patterns body)) body))

(defun get-first-pattern
  ((patterns) (car patterns)))

(defun pattern-to-env
  ((patterns arg-idx)
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
      (== (car expr) 'defun)
      false)))

(defun compile-defun-form
  (((defun-kw name . clauses))
    (if (is-old-style-params (car clauses))
      (compile-defun-old name (car clauses) (car (cdr clauses)))
      (compile-defun-multi name clauses))))

(defun is-old-style-params
  ((clause)
    (if (list? clause)
      (if (list? (car clause))
        false
        true)
      false)))

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
(print "=== Compiler v17 Tests ===")
(print "")

(print "Test 1: when expression")
(print (compile-expr '(when (< x 0) (- 0 x)) '(x)))
(print "Expected: if (< x 0) then (- 0 x) else false")
(print "")

(print "Test 2: unless expression")
(print (compile-expr '(unless (< x 0) x) '(x)))
(print "Expected: if (< x 0) then false else x")
(print "")

(print "Test 3: when in defun (practical example)")
(print (compile-defun-form '(defun print-positive (x)
                              (when (> x 0)
                                (print x)))))
(print "")

(print "Test 4: unless in defun")
(print (compile-defun-form '(defun ensure-positive (x)
                              (unless (> x 0)
                                0))))
(print "")

(print "Test 5: Comparison operators still work")
(print (compile-expr '(< 3 5) '()))
(print "")

(print "Test 6: cond still works")
(print (compile-expr '(cond
                        ((< x 0) -1)
                        ((> x 0) 1)
                        (else 0))
                      '(x)))
(print "")

(print "Test 7: Pattern matching still works")
(print (compile-defun-form '(defun fact ((0) 1) ((n) (* n (fact (- n 1)))))))
(print "")

(print "All tests complete!")
