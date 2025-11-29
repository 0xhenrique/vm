; Debug version of pattern matching to trace execution

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

(defun get-clause-patterns (((patterns body)) patterns))
(defun get-clause-body (((patterns body)) body))
(defun get-first-pattern ((patterns) (car patterns)))

(defun compile-pattern-match
  ((pattern)
    (cons (make-instr-2 OP-LOAD-ARG 0)
          (cons (make-instr-2 OP-PUSH pattern)
                (cons (make-instr-1 OP-EQ)
                      (cons (make-instr-2 OP-JMP-IF-FALSE 6) '()))))))

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

(defun compile-clauses-list
  ((clauses)
    (if (== (cdr clauses) '())
      ; Single clause
      (progn
        (print "compile-clauses-list: LAST CLAUSE")
        (print (car clauses))
        (append-bytecode (cdr (compile-expr (get-clause-body (car clauses))
                                            (pattern-to-env (get-clause-patterns (car clauses)) 0)))
                         (cons (make-instr-1 OP-RET) '())))
      ; Multiple clauses
      (progn
        (print "compile-clauses-list: MULTIPLE CLAUSES")
        (print "  clause:")
        (print (car clauses))
        (print "  rest:")
        (print (cdr clauses))
        (compile-clause-with-next (cons (car clauses) (cdr clauses)))))))

(defun compile-clause-with-next (pair)
  (print "compile-clause-with-next:")
  (print "  First pattern:")
  (print (get-first-pattern (get-clause-patterns (car pair))))
  (print "  Is symbol?")
  (print (symbol? (get-first-pattern (get-clause-patterns (car pair)))))

  (if (symbol? (get-first-pattern (get-clause-patterns (car pair))))
    (progn
      (print "  → Variable pattern, compiling body only")
      (append-bytecode (cdr (compile-expr (get-clause-body (car pair))
                                          (cons (get-first-pattern (get-clause-patterns (car pair))) '())))
                       (cons (make-instr-1 OP-RET) '())))
    (progn
      (print "  → Literal pattern, compiling with test")
      (print "  Pattern match bytecode:")
      (print (compile-pattern-match (get-first-pattern (get-clause-patterns (car pair)))))
      (print "  Body bytecode:")
      (print (cdr (compile-expr (get-clause-body (car pair)) '())))
      (print "  Now recursing on rest...")
      (append-bytecode (compile-pattern-match (get-first-pattern (get-clause-patterns (car pair))))
                       (append-bytecode (cdr (compile-expr (get-clause-body (car pair)) '()))
                                        (append-bytecode (cons (make-instr-1 OP-RET) '())
                                                         (compile-clauses-list (cdr pair)))))))))

; Test
(print "=== Testing Pattern Matching Compilation ===")
(print "")
(defconst test-clauses '(((0) true) ((n) false)))
(print "Input clauses:")
(print test-clauses)
(print "")
(print "Starting compilation...")
(print "")
(defconst result (compile-clauses-list test-clauses))
(print "")
(print "=== FINAL RESULT ===")
(print result)
