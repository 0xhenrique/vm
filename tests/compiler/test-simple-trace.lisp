; Simpler debug trace

(def OP-PUSH 'push)
(def OP-LOAD-ARG 'load-arg)
(def OP-EQ 'eq)
(def OP-JMP-IF-FALSE 'jmp-if-false)
(def OP-RET 'ret)

(defun make-instr-1 (op) (cons op '()))
(defun make-instr-2 (op arg) (cons op (cons arg '())))

(defun append-bytecode (bc1 bc2)
  (if (== bc1 '())
    bc2
    (cons (car bc1) (append-bytecode (cdr bc1) bc2))))

(defun get-clause-patterns (clause) (car clause))
(defun get-clause-body (clause) (car (cdr clause)))
(defun get-first-pattern (patterns) (car patterns))

(defun compile-pattern-match (pattern)
  (cons (make-instr-2 OP-LOAD-ARG 0)
        (cons (make-instr-2 OP-PUSH pattern)
              (cons (make-instr-1 OP-EQ)
                    (cons (make-instr-2 OP-JMP-IF-FALSE 6) '())))))

(defun compile-expr (expr env)
  (if (symbol? expr)
    (cons env (cons (make-instr-2 OP-LOAD-ARG 0) '()))
    (cons env (cons (make-instr-2 OP-PUSH expr) '()))))

(defun pattern-to-env (patterns arg-idx)
  (if (symbol? (car patterns))
    (cons (car patterns) '())
    '()))

; Main compilation function
(defun compile-clauses (clauses)
  (if (== (cdr clauses) '())
    ; Single clause - no pattern test needed
    (append-bytecode (cdr (compile-expr (get-clause-body (car clauses))
                                        (pattern-to-env (get-clause-patterns (car clauses)) 0)))
                     (cons (make-instr-1 OP-RET) '()))
    ; Multiple clauses - need pattern test
    (compile-first-clause (car clauses) (cdr clauses))))

(defun compile-first-clause (clause rest)
  (if (symbol? (get-first-pattern (get-clause-patterns clause)))
    ; Variable pattern - always matches
    (append-bytecode (cdr (compile-expr (get-clause-body clause)
                                        (cons (get-first-pattern (get-clause-patterns clause)) '())))
                     (cons (make-instr-1 OP-RET) '()))
    ; Literal pattern - need test and fallthrough
    (append-bytecode (compile-pattern-match (get-first-pattern (get-clause-patterns clause)))
                     (append-bytecode (cdr (compile-expr (get-clause-body clause) '()))
                                      (append-bytecode (cons (make-instr-1 OP-RET) '())
                                                       (compile-clauses rest))))))

; Test
(print "=== Testing ===")
(print "")
(def test-clauses '(((0) true) ((n) false)))
(print "Input:")
(print test-clauses)
(print "")
(print "Compiling...")
(print "")
(def result (compile-clauses test-clauses))
(print "")
(print "=== FINAL RESULT ===")
(print result)
