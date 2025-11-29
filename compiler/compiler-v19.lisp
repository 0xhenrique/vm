; ============================================================================
; Lisp Self-Hosting Compiler - Version 19
; ============================================================================
; Adds file I/O and string operations - Critical self-hosting milestone!
; - File I/O: read-file, write-file, file-exists?
; - String operations: string-length, substring, string-append
; - String conversions: string->list, list->string
;
; This version enables the compiler to read source files and write bytecode,
; making self-hosting achievable!

; ============================================================================
; Global Constants - Bytecode Operations
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

; String operations (v19)
(defconst OP-STRING-LENGTH 'string-length)
(defconst OP-SUBSTRING 'substring)
(defconst OP-STRING-APPEND 'string-append)
(defconst OP-STRING-TO-LIST 'string->list)
(defconst OP-LIST-TO-STRING 'list->string)

; File I/O operations (v19)
(defconst OP-READ-FILE 'read-file)
(defconst OP-WRITE-FILE 'write-file)
(defconst OP-FILE-EXISTS 'file-exists?)

; Command-line arguments (v19)
(defconst OP-GET-ARGS 'get-args)

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
                                  (if (== op 'string-length)
                                    (compile-string-length args env)
                                    (if (== op 'substring)
                                      (compile-substring args env)
                                      (if (== op 'string-append)
                                        (compile-string-append args env)
                                        (if (== op 'string->list)
                                          (compile-string-to-list args env)
                                          (if (== op 'list->string)
                                            (compile-list-to-string args env)
                                            (if (== op 'read-file)
                                              (compile-read-file args env)
                                              (if (== op 'write-file)
                                                (compile-write-file args env)
                                                (if (== op 'file-exists?)
                                                  (compile-file-exists args env)
                                                  (if (== op 'get-args)
                                                    (compile-get-args args env)
                                                    (compile-call op args env)))))))))))))))))))))))))))))
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
; String Operations (v19)
; ============================================================================

(defun compile-string-length
  (((arg) env)
    (cons env
          (append-bytecode (cdr (compile-expr arg env))
                          (cons (make-instr-1 OP-STRING-LENGTH) '())))))

(defun compile-substring
  ((args env)
    (compile-substring-step3 (car (cdr (cdr args)))
                             (compile-substring-step2 (car (cdr args)) (compile-expr (car args) env)))))

(defun compile-substring-step2
  ((arg2 r1)
    (cons (car r1)
          (cons (cdr r1)
                (compile-expr arg2 (car r1))))))

(defun compile-substring-step3
  ((arg3 combined)
    (cons (car (cdr (cdr combined)))
          (append-bytecode (car (cdr combined))
                          (append-bytecode (cdr (cdr (cdr combined)))
                                          (cons (make-instr-1 OP-SUBSTRING) '()))))))

(defun compile-string-append
  ((args env)
    (compile-binary-step2 (car (cdr args)) (compile-expr (car args) env) (make-instr-1 OP-STRING-APPEND))))

(defun compile-string-to-list
  (((arg) env)
    (cons env
          (append-bytecode (cdr (compile-expr arg env))
                          (cons (make-instr-1 OP-STRING-TO-LIST) '())))))

(defun compile-list-to-string
  (((arg) env)
    (cons env
          (append-bytecode (cdr (compile-expr arg env))
                          (cons (make-instr-1 OP-LIST-TO-STRING) '())))))

; ============================================================================
; File I/O Operations (v19)
; ============================================================================

(defun compile-read-file
  (((path) env)
    (cons env
          (append-bytecode (cdr (compile-expr path env))
                          (cons (make-instr-1 OP-READ-FILE) '())))))

(defun compile-write-file
  ((args env)
    (compile-binary-step2 (car (cdr args)) (compile-expr (car args) env) (make-instr-1 OP-WRITE-FILE))))

(defun compile-file-exists
  (((path) env)
    (cons env
          (append-bytecode (cdr (compile-expr path env))
                          (cons (make-instr-1 OP-FILE-EXISTS) '())))))

; ============================================================================
; Command-line Arguments (v19)
; ============================================================================

(defun compile-get-args
  (('() env)
    (cons env (cons (make-instr-1 OP-GET-ARGS) '())))
  ((args env)
    (cons env (cons (make-instr-2 'error '("get-args requires exactly 0 arguments")) '()))))

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

; ============================================================================
; Bytecode Serialization - Added for Self-Hosting
; ============================================================================

; Number serialization
(defun u32-to-bytes (n)
  (cons (% n 256)
    (cons (% (/ n 256) 256)
      (cons (% (/ n 65536) 256)
        (cons (/ n 16777216) '())))))

(defun i64-to-bytes (n)
  (cons (% n 256)
    (cons (% (/ n 256) 256)
      (cons (% (/ n 65536) 256)
        (cons (% (/ n 16777216) 256)
          (cons (% (/ n 4294967296) 256)
            (cons (% (/ n 1099511627776) 256)
              (cons (% (/ n 281474976710656) 256)
                (cons (/ n 72057594037927936) '())))))))))

; String serialization
(defun string-to-bytes (s)
  (string-chars-to-bytes (string->list s)))

(defun string-chars-to-bytes (chars)
  (if (== chars '())
    '()
    (cons (char-code (car chars))
          (string-chars-to-bytes (cdr chars)))))

(defun serialize-string (s)
  (let ((bytes (string-to-bytes s)))
    (append-bytecode (u32-to-bytes (list-length bytes)) bytes)))

; Value serialization (tags: 0=int, 1=bool, 2=list, 3=symbol, 4=string)
(defun serialize-value (val)
  (if (int? val)
    (cons 0 (i64-to-bytes val))
  (if (bool? val)
    (cons 1 (cons (if val 1 0) '()))
  (if (list? val)
    (cons 2 (append-bytecode (u32-to-bytes (list-length val))
                             (serialize-value-list val)))
  (if (symbol? val)
    (cons 3 (serialize-string (symbol-to-string val)))
  (if (string? val)
    (cons 4 (serialize-string val))
    '()))))))

(defun serialize-value-list (vals)
  (if (== vals '())
    '()
    (append-bytecode (serialize-value (car vals))
                     (serialize-value-list (cdr vals)))))

; Type predicates
(defun int? (v)
  (if (list? v) false
  (if (bool? v) false
  (if (symbol? v) false
  (if (string? v) false
    true)))))

(defun bool? (v)
  (or (== v true) (== v false)))

(defun symbol-to-string (s)
  (if (symbol? s)
    (symbol->string s)
    ""))

; Instruction serialization
(defun serialize-instr (instr)
  (let ((op (car instr)))
    (if (== op OP-PUSH)
      (cons 0 (serialize-value (car (cdr instr))))
    (if (== op OP-ADD) '(1)
    (if (== op OP-SUB) '(2)
    (if (== op OP-MUL) '(3)
    (if (== op OP-DIV) '(4)
    (if (== op OP-EQ) '(18)
    (if (== op OP-LT) '(15)
    (if (== op OP-GT) '(16)
    (if (== op OP-LTE) '(5)
    (if (== op OP-GTE) '(17)
    (if (== op OP-NEQ) '(19)
    (if (== op OP-JMP)
      (cons 7 (u32-to-bytes (car (cdr instr))))
    (if (== op OP-JMP-IF-FALSE)
      (cons 6 (u32-to-bytes (car (cdr instr))))
    (if (== op OP-GET-LOCAL)
      (cons 28 (u32-to-bytes (car (cdr instr))))
    (if (== op OP-LOAD-ARG)
      (cons 10 (u32-to-bytes (car (cdr instr))))
    (if (== op OP-CALL)
      (cons 8 (append-bytecode (serialize-string (car (cdr instr)))
                               (u32-to-bytes (car (cdr (cdr instr))))))
    (if (== op OP-RET) '(9)
      '(255))))))))))))))))))))))  ; Unknown

(defun serialize-instr-list (instrs)
  (if (== instrs '())
    '()
    (append-bytecode (serialize-instr (car instrs))
                     (serialize-instr-list (cdr instrs)))))

; Bytecode section serialization
(defun serialize-bytecode-section (instrs)
  (append-bytecode (u32-to-bytes (list-length instrs))
                   (serialize-instr-list instrs)))

; Function serialization  
(defun serialize-function (func)
  (append-bytecode (serialize-string (car func))
                   (serialize-bytecode-section (cdr func))))

(defun serialize-functions (funcs)
  (if (== funcs '())
    '()
    (append-bytecode (serialize-function (car funcs))
                     (serialize-functions (cdr funcs)))))

; Complete bytecode file
(defun serialize-bytecode-file (funcs main-bc)
  (append-bytecode '(76 73 83 80 6)  ; "LISP" + version 6
    (append-bytecode (u32-to-bytes (list-length funcs))
      (append-bytecode (serialize-functions funcs)
                       (serialize-bytecode-section main-bc)))))

; ============================================================================
; CLI Main Function - Self-Hosting Entry Point
; ============================================================================

(defun compiler-main ()
  (let ((args (get-args)))
    (if (< (list-length args) 2)
      (print "Usage: compiler-v19.bc <input.lisp> <output.bc>")
      (let ((input-file (car args)))
      (let ((output-file (car (cdr args))))
        (compile-and-write input-file output-file))))))

(defun compile-and-write (input-file output-file)
  (if (file-exists? input-file)
    (let ((source (read-file input-file)))
      (print (string-append "Compiling " input-file))
      (let ((result (compile-program-from-string source)))
      (let ((bytecode (serialize-bytecode-file (car result) (cdr result))))
        (if (write-binary-file output-file bytecode)
          (print (string-append "Success! Wrote " output-file))
          (print "Error writing bytecode file")))))
    (print (string-append "Error: File not found: " input-file))))

; Placeholder for now - would need a parser in Lisp
(defun compile-program-from-string (source)
  (cons '()  ; No functions for now
        '((push 42) (ret))))  ; Simple program

(print "")
(print "=== Compiler v19 with Self-Hosting Support ===")
(print "Run with: lisp-vm compiler-v19.bc <input.lisp> <output.bc>")
