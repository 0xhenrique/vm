; ============================================================================
; Lisp Self-Hosting Compiler - Version 5
; ============================================================================
; A clean, well-documented implementation of a Lisp-to-bytecode compiler
; written in Lisp itself.
;
; Core design:
; - Pure functional style: pass environment, return (env . bytecode) pairs
; - Two-pass compilation: definitions first, then main code
; - Bytecode as tagged lists: ((push 42) (add) (ret))

; ============================================================================
; Global Constants - Bytecode Operations
; ============================================================================

(defconst OP-PUSH 'push)
(defconst OP-ADD 'add)
(defconst OP-SUB 'sub)
(defconst OP-MUL 'mul)
(defconst OP-DIV 'div)
(defconst OP-CALL 'call)
(defconst OP-RET 'ret)

; ============================================================================
; Environment Helpers
; ============================================================================

; Create an empty environment
; Environment structure: ((param-names . (name1 name2 ...)))
(defun make-env (() '()))

; Get a value from the environment
; For now, environments are simple - we'll expand this later
(defun env-get
  ((env key)
    '()))  ; Placeholder

; Set a value in the environment
; Returns a new environment (functional style)
(defun env-set
  ((env key value)
    env))  ; Placeholder

; ============================================================================
; Helper Functions
; ============================================================================

; Calculate the length of a list
(defun length
  (('()) 0)
  (((h . t)) (+ 1 (length t))))

; Find the index of an element in a list
(defun index-of-impl
  ((elem '() idx) -1)
  ((elem (h . t) idx)
    (if (== elem h)
      idx
      (index-of-impl elem t (+ idx 1)))))

(defun index-of
  ((elem lst) (index-of-impl elem lst 0)))

; Create a single-argument instruction
(defun make-instr-1
  ((op) (cons op '())))

; Create a two-argument instruction
(defun make-instr-2
  ((op arg) (cons op (cons arg '()))))

; Append a list of instructions to another
(defun append-bytecode
  (('() bc2) bc2)
  (((h . t) bc2) (cons h (append-bytecode t bc2))))

; ============================================================================
; Core Compiler - Result Convention
; ============================================================================
; All compile-* functions return a pair: (env . bytecode)
; - env: the (potentially updated) environment
; - bytecode: a list of instructions like ((push 42) (add))

; Main entry point for expression compilation
(defun compile-expr
  ((expr env)
    (if (list? expr)
      (compile-list expr env)
      (if (symbol? expr)
        (compile-symbol expr env)
        ; Otherwise it's a literal (number, boolean, etc.)
        (compile-literal expr env)))))

; Compile a literal value (number, boolean, etc.)
(defun compile-literal
  ((value env)
    (cons env (cons (make-instr-2 OP-PUSH value) '()))))

; Compile a symbol (variable reference)
(defun compile-symbol
  ((name env)
    ; For now, just generate an error - we'll implement variable lookup later
    (cons env (cons (make-instr-2 'error (cons "Undefined variable:" (cons name '()))) '()))))

; Compile a list (function call or special form)
(defun compile-list
  (('() env)
    (cons env '((error "Empty list"))))
  (((op . args) env)
    (compile-operation op args env)))

; Dispatch based on operation
(defun compile-operation
  ((op args env)
    (if (== op '+)
      (compile-add args env)
      (if (== op '-)
        (compile-sub args env)
        (if (== op '*)
          (compile-mul args env)
          (if (== op '/)
            (compile-div args env)
            ; Unknown operation
            (cons env (cons (make-instr-2 'error (cons "Unknown operation:" (cons op '()))) '()))))))))

; ============================================================================
; Arithmetic Operators
; ============================================================================
; Each operator compiles its arguments (each returns (env . bytecode)),
; combines the bytecode, and appends the operation instruction.

; Addition: (+ expr1 expr2)
(defun compile-add
  (((arg1 . rest) env)
    (if (== rest '())
      ; Single argument: (+ x) => just compile x
      (compile-expr arg1 env)
      ; Two or more arguments: compile first two
      (let ((arg2 (car rest)))
        ; Compile first argument
        (let ((result1 (compile-expr arg1 env)))
          ; result1 is (env . bytecode1)
          (let ((env1 (car result1)))
            (let ((bc1 (cdr result1)))
              ; Compile second argument
              (let ((result2 (compile-expr arg2 env1)))
                ; result2 is (env . bytecode2)
                (let ((env2 (car result2)))
                  (let ((bc2 (cdr result2)))
                    ; Combine: bytecode1 + bytecode2 + (add)
                    (let ((combined (append-bytecode bc1 (append-bytecode bc2 (cons (make-instr-1 OP-ADD) '())))))
                      (cons env2 combined))))))))))))

; Subtraction: (- expr1 expr2)
(defun compile-sub
  (((arg1 . rest) env)
    (if (== rest '())
      ; Single argument: (- x) => negate
      (let ((result (compile-expr arg1 env)))
        (let ((env1 (car result)))
          (let ((bc (cdr result)))
            (cons env1 (append-bytecode bc (cons (make-instr-1 'neg) '()))))))
      ; Two or more arguments
      (let ((arg2 (car rest)))
        (let ((result1 (compile-expr arg1 env)))
          (let ((env1 (car result1)))
            (let ((bc1 (cdr result1)))
              (let ((result2 (compile-expr arg2 env1)))
                (let ((env2 (car result2)))
                  (let ((bc2 (cdr result2)))
                    (let ((combined (append-bytecode bc1 (append-bytecode bc2 (cons (make-instr-1 OP-SUB) '())))))
                      (cons env2 combined))))))))))))

; Multiplication: (* expr1 expr2)
(defun compile-mul
  (((arg1 . rest) env)
    (if (== rest '())
      (compile-expr arg1 env)
      (let ((arg2 (car rest)))
        (let ((result1 (compile-expr arg1 env)))
          (let ((env1 (car result1)))
            (let ((bc1 (cdr result1)))
              (let ((result2 (compile-expr arg2 env1)))
                (let ((env2 (car result2)))
                  (let ((bc2 (cdr result2)))
                    (let ((combined (append-bytecode bc1 (append-bytecode bc2 (cons (make-instr-1 OP-MUL) '())))))
                      (cons env2 combined))))))))))))

; Division: (/ expr1 expr2)
(defun compile-div
  (((arg1 . rest) env)
    (if (== rest '())
      (compile-expr arg1 env)
      (let ((arg2 (car rest)))
        (let ((result1 (compile-expr arg1 env)))
          (let ((env1 (car result1)))
            (let ((bc1 (cdr result1)))
              (let ((result2 (compile-expr arg2 env1)))
                (let ((env2 (car result2)))
                  (let ((bc2 (cdr result2)))
                    (let ((combined (append-bytecode bc1 (append-bytecode bc2 (cons (make-instr-1 OP-DIV) '())))))
                      (cons env2 combined))))))))))))

; ============================================================================
; Tests
; ============================================================================

(print "")
(print "=== Compiler v5 Tests ===")
(print "")

; Test 1: Compile a literal
(print "Test 1: (compile-expr 42 env)")
(let ((result (compile-expr 42 (make-env))))
  (let ((bytecode (cdr result)))
    (print bytecode)))
(print "Expected: ((push 42))")
(print "")

; Test 2: Compile simple addition
(print "Test 2: (compile-expr '(+ 1 2) env)")
(let ((result (compile-expr '(+ 1 2) (make-env))))
  (let ((bytecode (cdr result)))
    (print bytecode)))
(print "Expected: ((push 1) (push 2) (add))")
(print "")

; Test 3: Compile nested addition
(print "Test 3: (compile-expr '(+ 5 (+ 2 3)) env)")
(let ((result (compile-expr '(+ 5 (+ 2 3)) (make-env))))
  (let ((bytecode (cdr result)))
    (print bytecode)))
(print "Expected: ((push 5) (push 2) (push 3) (add) (add))")
(print "")
