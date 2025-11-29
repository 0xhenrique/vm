; ============================================================================
; Lisp Self-Hosting Compiler - Version 9
; ============================================================================
; Uses continuation-passing style to avoid nested let statements

; ============================================================================
; Global Constants
; ============================================================================

(defconst OP-PUSH 'push)
(defconst OP-ADD 'add)
(defconst OP-SUB 'sub)
(defconst OP-MUL 'mul)
(defconst OP-DIV 'div)

; ============================================================================
; Helpers
; ============================================================================

(defun make-env (() '()))

(defun append-bytecode
  (('() bc2) bc2)
  (((h . t) bc2) (cons h (append-bytecode t bc2))))

(defun make-instr-1
  ((op) (cons op '())))

(defun make-instr-2
  ((op arg) (cons op (cons arg '()))))

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
    (cons env (cons (make-instr-2 'error (cons "Undefined variable" (cons name '()))) '()))))

(defun compile-list
  (('() env)
    (cons env (cons (make-instr-2 'error '("Empty list")) '())))
  (((op . args) env)
    (if (== op '+)
      (compile-add args env)
      (if (== op '-)
        (compile-sub args env)
        (if (== op '*)
          (compile-mul args env)
          (if (== op '/)
            (compile-div args env)
            (cons env (cons (make-instr-2 'error (cons "Unknown operation" (cons op '()))) '()))))))))

; ============================================================================
; Binary Operation Helpers with CPS
; ============================================================================

; Second step: given r1, compile arg2 and combine
(defun compile-binary-step2
  ((arg2 r1 op)
    ; r1 is already compiled, now compile arg2
    ; Get env from r1, compile arg2 with it, then combine
    (cons (car (compile-expr arg2 (car r1)))
          (append-bytecode (cdr r1)
                           (append-bytecode (cdr (compile-expr arg2 (car r1)))
                                            (cons op '()))))))

; ============================================================================
; Arithmetic Operations
; ============================================================================

(defun compile-add
  (((arg1 . rest) env)
    (if (== rest '())
      (compile-expr arg1 env)
      ; Pass result of arg1 to step2
      (compile-binary-step2 (car rest) (compile-expr arg1 env) (make-instr-1 OP-ADD)))))

(defun compile-sub
  (((arg1 . rest) env)
    (if (== rest '())
      ; Unary negation
      (cons (car (compile-expr arg1 env))
            (append-bytecode (cdr (compile-expr arg1 env))
                             (cons (make-instr-1 'neg) '())))
      ; Binary subtraction
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
(print "=== Compiler v9 Tests ===")
(print "")

(print "Test 1: 42")
(print (cdr (compile-expr 42 (make-env))))
(print "")

(print "Test 2: (+ 1 2)")
(print (cdr (compile-expr '(+ 1 2) (make-env))))
(print "")

(print "Test 3: (+ 5 (+ 2 3))")
(print (cdr (compile-expr '(+ 5 (+ 2 3)) (make-env))))
(print "")

(print "All tests complete!")
