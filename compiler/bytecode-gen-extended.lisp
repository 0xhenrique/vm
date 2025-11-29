; Extended Bytecode Generator with Special Forms
; Adds support for defun, let, and proper if

; Helper: get nth element from list
(defun list-ref (lst n)
  (if (== n 0)
    (car lst)
    (list-ref (cdr lst) (- n 1))))

; Compile a number literal
(defun compile-number (value env)
  (cons (cons "Push" (cons value '())) '()))

; Compile a symbol - check environment
(defun compile-symbol (name env)
  ; For now, simple implementation
  ; TODO: Check if it's a parameter, local, or global
  (cons (cons "LoadSymbol" (cons name '())) '()))

; Compile multiple expressions in sequence
(defun compile-exprs (exprs env acc)
  (if (== exprs '())
    acc
    (let ((code (compile-expr (car exprs) env)))
      (compile-exprs (cdr exprs) env (append acc code)))))

; Check if symbol is a special form
(defun is-special-form? (name)
  (or (== name "if")
      (== name "defun")
      (== name "let")
      (== name "quote")))

; Compile defun: (defun name (params) body)
(defun compile-defun (args env)
  (if (>= (list-length args) 3)
    ; args = (name (params) body)
    (let ((fname (car (cdr (car args)))))
      (let ((params (list-ref args 1)))
        ; Extract param names from params list
        ; For now, simplified - assume params is ("list" ("symbol" "x") ...)
        (let ((body-expr (list-ref args 2)))
          ; Compile function body with params in environment
          ; TODO: Add params to environment
          (let ((body-code (compile-expr body-expr env)))
            (append body-code
              (cons (cons "DefineFunction" (cons fname '())) '()))))))
    '()))

; Compile let: (let ((var val)) body)
(defun compile-let (args env)
  (if (>= (list-length args) 2)
    ; args = (bindings body)
    (let ((bindings (car args)))
      (let ((body (list-ref args 1)))
        ; For each binding, compile value and emit SetLocal
        ; TODO: Implement proper let compilation
        (compile-expr body env)))
    '()))

; Compile if: (if cond then else)
(defun compile-if (args env)
  (if (== (list-length args) 3)
    (let ((cond-code (compile-expr (car args) env)))
      (let ((then-code (compile-expr (list-ref args 1) env)))
        (let ((else-code (compile-expr (list-ref args 2) env)))
          ; Emit: cond-code, JmpIfFalse(after-then), then-code, Jmp(after-else), else-code
          (append cond-code
            (append (cons (cons "JmpIfFalse" (cons "THEN-END" '())) '())
              (append then-code
                (append (cons (cons "Jmp" (cons "IF-END" '())) '())
                  else-code)))))))
    '()))

; Compile quote: (quote expr)
(defun compile-quote (args env)
  ; Return the expression as literal data
  ; For now, just push the expression
  (cons (cons "PushQuoted" (cons (car args) '())) '()))

; Compile function call
(defun compile-call (operator args env)
  (let ((args-code (compile-exprs args env '())))
    (let ((argc (list-length args)))
      (append args-code
        (cons (cons "Call" (cons operator (cons argc '()))) '())))))

; Compile list expression
(defun compile-list (items env)
  (if (== items '())
    '()
    (let ((first (car items)))
      (if (== (car first) "symbol")
        (let ((name (car (cdr first))))
          (if (is-special-form? name)
            ; Handle special forms
            (if (== name "if")
              (compile-if (cdr items) env)
              (if (== name "defun")
                (compile-defun (cdr items) env)
                (if (== name "let")
                  (compile-let (cdr items) env)
                  (if (== name "quote")
                    (compile-quote (cdr items) env)
                    '()))))
            ; Regular function call
            (compile-call name (cdr items) env)))
        '()))))

; Main compile function
(defun compile-expr (expr env)
  (let ((type (car expr)))
    (if (== type "number")
      (compile-number (car (cdr expr)) env)
      (if (== type "symbol")
        (compile-symbol (car (cdr expr)) env)
        (if (== type "list")
          (compile-list (cdr expr) env)
          '())))))

; Top-level compile
(defun compile (parsed-expr)
  (compile-expr parsed-expr '()))

; === TESTS ===

(print "=== Extended Bytecode Generator ===")

(print "Test 1: (if (> x 0) 1 -1)")
(print (compile (cons "list"
  (cons (cons "symbol" (cons "if" '()))
    (cons (cons "list"
      (cons (cons "symbol" (cons ">" '()))
        (cons (cons "symbol" (cons "x" '()))
          (cons (cons "number" (cons "0" '())) '()))))
      (cons (cons "number" (cons "1" '()))
        (cons (cons "number" (cons "-1" '())) '())))))))

(print "Test 2: (+ 1 2)")
(print (compile (cons "list"
  (cons (cons "symbol" (cons "+" '()))
    (cons (cons "number" (cons "1" '()))
      (cons (cons "number" (cons "2" '())) '()))))))
