; Bytecode Generator - compiles parsed s-expressions to bytecode
;
; Input: Parsed expressions from parser
;   ("number" "42")
;   ("symbol" "x")
;   ("list" expr1 expr2 ...)
;
; Output: Bytecode instructions as lists
;   ("Push" value)
;   ("Add")
;   ("Call" name argc)
;   etc.

; Helper: convert string to number
; For now, we'll represent numbers as strings in bytecode
; The VM will need to parse them
(defun string->number (str)
  ; TODO: Implement proper string->number conversion
  ; For now, just return the string and let the VM handle it
  str)

; Compile a number literal
(defun compile-number (value)
  (cons (cons "Push" (cons value '())) '()))

; Compile a symbol (variable lookup)
; For now, we'll assume all symbols are function names or built-ins
(defun compile-symbol (name env)
  ; TODO: Check if it's a parameter, local, or global
  ; For now, just return the symbol as-is for later resolution
  (cons (cons "LoadSymbol" (cons name '())) '()))

; Compile a list of expressions (for arguments, etc.)
(defun compile-exprs (exprs env acc)
  (if (== exprs '())
    acc
    (let ((instr (compile-expr (car exprs) env)))
      (compile-exprs (cdr exprs) env (append acc instr)))))

; Check if a symbol is a special form
(defun is-special-form? (name)
  (or (== name "if")
      (== name "defun")
      (== name "let")
      (== name "quote")))

; Compile an if expression
(defun compile-if (args env)
  ; (if condition then-expr else-expr)
  ; TODO: Implement proper if compilation with jumps
  ; For now, simplified version
  (if (== (list-length args) 3)
    (let ((cond-code (compile-expr (car args) env)))
      (let ((then-code (compile-expr (car (cdr args)) env)))
        (let ((else-code (compile-expr (car (cdr (cdr args))) env)))
          (append cond-code
            (append (cons (cons "JmpIfFalse" (cons "PLACEHOLDER" '())) '())
              (append then-code
                (append (cons (cons "Jmp" (cons "PLACEHOLDER" '())) '())
                  else-code)))))))
    '()))

; Compile a function call
(defun compile-call (operator args env)
  ; First compile all arguments
  (let ((args-code (compile-exprs args env '())))
    ; Then emit the call
    (let ((argc (list-length args)))
      (append args-code
        (cons (cons "Call" (cons operator (cons argc '()))) '())))))

; Compile a list expression
(defun compile-list (items env)
  (if (== items '())
    '()
    (let ((first (car items)))
      ; Check what kind of expression this is
      (if (== (car first) "symbol")
        (let ((name (car (cdr first))))
          (if (is-special-form? name)
            ; Handle special forms
            (if (== name "if")
              (compile-if (cdr items) env)
              '()) ; TODO: other special forms
            ; Regular function call
            (compile-call name (cdr items) env)))
        '()))))

; Main compile function
(defun compile-expr (expr env)
  (let ((type (car expr)))
    (if (== type "number")
      (compile-number (car (cdr expr)))
      (if (== type "symbol")
        (compile-symbol (car (cdr expr)) env)
        (if (== type "list")
          (compile-list (cdr expr) env)
          '())))))

; Top-level compile function
(defun compile (parsed-expr)
  (compile-expr parsed-expr '()))

; === TESTS ===

(print "=== Bytecode Generator Tests ===")

(print "Test 1: Compile number 42")
(print (compile (cons "number" (cons "42" '()))))

(print "Test 2: Compile symbol x")
(print (compile (cons "symbol" (cons "x" '()))))

(print "Test 3: Compile (+ 1 2)")
(print (compile (cons "list"
                  (cons (cons "symbol" (cons "+" '()))
                    (cons (cons "number" (cons "1" '()))
                      (cons (cons "number" (cons "2" '())) '()))))))
