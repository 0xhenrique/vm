; ============================================================================
; Lisp Self-Hosting Compiler - Version 10
; ============================================================================
; Adds support for if conditionals with jump instructions

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

; Calculate the length of a list (number of instructions)
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
    ; For now, treat symbols as globals or error
    (cons env (cons (make-instr-2 'load-global name) '()))))

(defun compile-list
  (('() env)
    (cons env (cons (make-instr-2 'error '("Empty list")) '())))
  (((op . args) env)
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
                (cons env (cons (make-instr-2 'error (cons "Unknown operation" (cons op '()))) '()))))))))))

; ============================================================================
; Conditional Compilation
; ============================================================================

; Compile if: (if condition then-expr else-expr)
; Bytecode structure:
;   <condition-code>
;   (jmp-if-false <else-addr>)
;   <then-code>
;   (jmp <end-addr>)
;   <else-code>
(defun compile-if
  (((cond-expr then-expr else-expr) env)
    ; Compile all three branches first
    (cons (car (compile-expr else-expr (car (compile-expr then-expr (car (compile-expr cond-expr env))))))
          ; Now build the bytecode with computed addresses
          (compile-if-bytecode (cdr (compile-expr cond-expr env))
                               (cdr (compile-expr then-expr (car (compile-expr cond-expr env))))
                               (cdr (compile-expr else-expr (car (compile-expr then-expr (car (compile-expr cond-expr env))))))))))

; Helper to build if bytecode with correct addresses
(defun compile-if-bytecode
  ((cond-bc then-bc else-bc)
    ; Calculate addresses:
    ; - cond-bc starts at current position
    ; - jmp-if-false at: len(cond-bc)
    ; - then-bc at: len(cond-bc) + 1
    ; - jmp at: len(cond-bc) + 1 + len(then-bc)
    ; - else-bc at: len(cond-bc) + 1 + len(then-bc) + 1
    ; - end at: len(cond-bc) + 1 + len(then-bc) + 1 + len(else-bc)
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
; Comparison Operations
; ============================================================================

(defun compile-eq
  (((arg1 . rest) env)
    (if (== rest '())
      (cons env (cons (make-instr-2 'error '("== requires 2 arguments")) '()))
      (compile-binary-step2 (car rest) (compile-expr arg1 env) (make-instr-1 OP-EQ)))))

; ============================================================================
; Binary Operation Helpers with CPS
; ============================================================================

(defun compile-binary-step2
  ((arg2 r1 op)
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
(print "=== Compiler v10 Tests ===")
(print "")

(print "Test 1: 42")
(print (cdr (compile-expr 42 (make-env))))
(print "")

(print "Test 2: (+ 1 2)")
(print (cdr (compile-expr '(+ 1 2) (make-env))))
(print "")

(print "Test 3: (if true 10 20)")
(print (cdr (compile-expr '(if true 10 20) (make-env))))
(print "Expected: condition + jmp-if-false + then + jmp + else")
(print "")

(print "Test 4: (if (== 1 1) 100 200)")
(print (cdr (compile-expr '(if (== 1 1) 100 200) (make-env))))
(print "")

(print "All tests complete!")
