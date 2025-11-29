; ============================================================================
; Lisp Self-Hosting Compiler - Version 7
; ============================================================================
; Uses global variables for temporary storage to avoid nested let issues
;
; Workaround: The VM has issues with deeply nested let statements,
; so we use global variables as temporary storage when needed.

; ============================================================================
; Global Constants
; ============================================================================

(defconst OP-PUSH 'push)
(defconst OP-ADD 'add)
(defconst OP-SUB 'sub)
(defconst OP-MUL 'mul)
(defconst OP-DIV 'div)

; Global temporaries for intermediate results
(defvar temp-r1 '())
(defvar temp-r2 '())
(defvar temp-env '())
(defvar temp-bc '())

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
; Arithmetic - Binary Operations using globals
; ============================================================================

(defun compile-add
  (((arg1 . rest) env)
    (if (== rest '())
      (compile-expr arg1 env)
      ; Use globals to avoid nested let
      (progn
        (set temp-r1 (compile-expr arg1 env))
        (set temp-r2 (compile-expr (car rest) (car temp-r1)))
        (cons (car temp-r2)
              (append-bytecode (cdr temp-r1)
                               (append-bytecode (cdr temp-r2)
                                                (cons (make-instr-1 OP-ADD) '()))))))))

(defun compile-sub
  (((arg1 . rest) env)
    (if (== rest '())
      (progn
        (set temp-r1 (compile-expr arg1 env))
        (cons (car temp-r1)
              (append-bytecode (cdr temp-r1)
                               (cons (make-instr-1 'neg) '()))))
      (progn
        (set temp-r1 (compile-expr arg1 env))
        (set temp-r2 (compile-expr (car rest) (car temp-r1)))
        (cons (car temp-r2)
              (append-bytecode (cdr temp-r1)
                               (append-bytecode (cdr temp-r2)
                                                (cons (make-instr-1 OP-SUB) '()))))))))

(defun compile-mul
  (((arg1 . rest) env)
    (if (== rest '())
      (compile-expr arg1 env)
      (progn
        (set temp-r1 (compile-expr arg1 env))
        (set temp-r2 (compile-expr (car rest) (car temp-r1)))
        (cons (car temp-r2)
              (append-bytecode (cdr temp-r1)
                               (append-bytecode (cdr temp-r2)
                                                (cons (make-instr-1 OP-MUL) '()))))))))

(defun compile-div
  (((arg1 . rest) env)
    (if (== rest '())
      (compile-expr arg1 env)
      (progn
        (set temp-r1 (compile-expr arg1 env))
        (set temp-r2 (compile-expr (car rest) (car temp-r1)))
        (cons (car temp-r2)
              (append-bytecode (cdr temp-r1)
                               (append-bytecode (cdr temp-r2)
                                                (cons (make-instr-1 OP-DIV) '()))))))))

; ============================================================================
; Tests
; ============================================================================

(print "")
(print "=== Compiler v7 Tests ===")
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
