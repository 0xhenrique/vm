; Debug: test pattern matching compilation

(defconst test-clauses '(((0) true) ((n) false)))

(print "Test clauses:")
(print test-clauses)

(print "")
(print "First clause:")
(print (car test-clauses))

(print "First clause patterns:")
(print (get-clause-patterns (car test-clauses)))

(print "First pattern:")
(print (get-first-pattern (get-clause-patterns (car test-clauses))))

(print "Is first pattern a symbol?")
(print (symbol? (get-first-pattern (get-clause-patterns (car test-clauses)))))

(print "")
(print "Second clause (rest):")
(print (cdr test-clauses))

(print "Compiling all clauses:")
(print (compile-all-clauses test-clauses))

; Helper functions
(defun get-clause-patterns (((patterns body)) patterns))
(defun get-clause-body (((patterns body)) body))
(defun get-first-pattern ((patterns) (car patterns)))

(defconst OP-PUSH 'push)
(defconst OP-LOAD-ARG 'load-arg)
(defconst OP-EQ 'eq)
(defconst OP-JMP-IF-FALSE 'jmp-if-false)
(defconst OP-RET 'ret)

(defun make-instr-1 ((op) (cons op '())))
(defun make-instr-2 ((op arg) (cons op (cons arg '()))))

(defun append-bytecode
  (('() bc2) bc2)
  (((h . t) bc2) (cons h (append-bytecode t bc2))))

(defun make-env (() '()))

(defun compile-expr
  ((expr env)
    (if (symbol? expr)
      (cons env (cons (make-instr-2 OP-LOAD-ARG 0) '()))
      (cons env (cons (make-instr-2 OP-PUSH expr) '())))))

(defun pattern-to-env
  ((patterns arg-idx)
    (if (symbol? (car patterns))
      (cons (car patterns) '())
      '())))

(defun compile-all-clauses
  ((clauses)
    (compile-clauses-impl clauses 0)))

(defun compile-clauses-impl
  (((clause) current-addr)
    (append-bytecode (cdr (compile-expr (get-clause-body clause)
                                        (pattern-to-env (get-clause-patterns clause) 0)))
                     (cons (make-instr-1 OP-RET) '())))
  (((clause . rest) current-addr)
    (compile-clause-with-jumps clause rest current-addr)))

(defun compile-clause-with-jumps
  ((clause rest current-addr)
    (if (symbol? (get-first-pattern (get-clause-patterns clause)))
      (append-bytecode (cdr (compile-expr (get-clause-body clause)
                                          (cons (get-first-pattern (get-clause-patterns clause)) '())))
                       (cons (make-instr-1 OP-RET) '()))
      (append-bytecode (compile-pattern-match (get-first-pattern (get-clause-patterns clause)))
                       (append-bytecode (cdr (compile-expr (get-clause-body clause) '()))
                                        (append-bytecode (cons (make-instr-1 OP-RET) '())
                                                         (compile-clauses-impl rest 0)))))))

(defun compile-pattern-match
  ((pattern)
    (cons (make-instr-2 OP-LOAD-ARG 0)
          (cons (make-instr-2 OP-PUSH pattern)
                (cons (make-instr-1 OP-EQ)
                      (cons (make-instr-2 OP-JMP-IF-FALSE 6) '()))))))
