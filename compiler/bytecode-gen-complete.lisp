; Complete Compiler with Environment Tracking for defun and let
; Integrates environment tracking into function and let compilation

; === HELPERS ===

(defun list-ref (lst n)
  (if (== n 0)
    (car lst)
    (list-ref (cdr lst) (- n 1))))

(defun reverse-helper (lst acc)
  (if (== lst '())
    acc
    (reverse-helper (cdr lst) (cons (car lst) acc))))

(defun reverse (lst)
  (reverse-helper lst '()))

; === ENVIRONMENT STRUCTURE ===

(defun make-env ()
  '())

(defun env-push-frame (params locals env)
  (cons (cons params (cons locals '())) env))

(defun env-current-params (env)
  (if (== env '())
    '()
    (car (car env))))

(defun env-current-locals (env)
  (if (== env '())
    '()
    (car (cdr (car env)))))

(defun find-index-helper (name lst index)
  (if (== lst '())
    -1
    (if (== (car lst) name)
      index
      (find-index-helper name (cdr lst) (+ index 1)))))

(defun find-index (name lst)
  (find-index-helper name lst 0))

(defun env-lookup (name env)
  (if (== env '())
    (cons "LoadGlobal" (cons name '()))
    (let ((params (env-current-params env)))
      (let ((locals (env-current-locals env)))
        (let ((param-idx (find-index name params)))
          (if (>= param-idx 0)
            (cons "LoadArg" (cons param-idx '()))
            (let ((local-idx (find-index name locals)))
              (if (>= local-idx 0)
                (cons "GetLocal" (cons local-idx '()))
                (cons "LoadGlobal" (cons name '()))))))))))

; === LABEL GENERATION ===

(defun make-label-counter ()
  '(0))

(defun next-label-id (counter)
  (let ((id (car counter)))
    (cons (+ id 1) '())))

(defun get-label-id (counter)
  (car counter))

(defun make-label (prefix counter)
  (cons (get-label-id counter) (cons prefix '())))

; === AST HELPERS ===

; Extract parameter names from defun params list
; Input: ("list" ("symbol" "x") ("symbol" "y"))
; Output: ("x" "y")
(defun extract-param-names (params-ast)
  (if (== (car params-ast) "list")
    (extract-param-names-loop (cdr params-ast) '())
    '()))

(defun extract-param-names-loop (items acc)
  (if (== items '())
    acc
    (let ((param (car items)))
      (if (== (car param) "symbol")
        (let ((name (car (cdr param))))
          (extract-param-names-loop (cdr items) (append acc (cons name '()))))
        (extract-param-names-loop (cdr items) acc)))))

; Extract let bindings
; Input: ("list" ("list" ("symbol" "x") val-expr) ...)
; Output: (("x" val-expr) ...)
(defun extract-let-bindings (bindings-ast)
  (if (== (car bindings-ast) "list")
    (extract-let-bindings-loop (cdr bindings-ast) '())
    '()))

(defun extract-let-bindings-loop (items acc)
  (if (== items '())
    acc
    (let ((binding (car items)))
      (if (== (car binding) "list")
        (let ((binding-items (cdr binding)))
          (if (>= (list-length binding-items) 2)
            (let ((var-ast (car binding-items)))
              (let ((val-ast (car (cdr binding-items))))
                (if (== (car var-ast) "symbol")
                  (let ((var-name (car (cdr var-ast))))
                    (let ((pair (cons var-name (cons val-ast '()))))
                      (extract-let-bindings-loop (cdr items) (append acc (cons pair '())))))
                  (extract-let-bindings-loop (cdr items) acc))))
            (extract-let-bindings-loop (cdr items) acc)))
        (extract-let-bindings-loop (cdr items) acc)))))

; Get variable names from bindings
(defun get-binding-names (bindings)
  (get-binding-names-loop bindings '()))

(defun get-binding-names-loop (bindings acc)
  (if (== bindings '())
    acc
    (let ((binding (car bindings)))
      (let ((name (car binding)))
        (get-binding-names-loop (cdr bindings) (append acc (cons name '())))))))

; === BYTECODE COMPILATION ===

(defun compile-number (value env counter)
  (cons (cons (cons "Push" (cons value '())) '())
    (cons counter '())))

(defun compile-symbol (name env counter)
  (let ((lookup-result (env-lookup name env)))
    (cons (cons lookup-result '())
      (cons counter '()))))

(defun compile-exprs (exprs env counter acc)
  (if (== exprs '())
    (cons acc (cons counter '()))
    (let ((result (compile-expr (car exprs) env counter)))
      (let ((code (car result)))
        (let ((new-counter (car (cdr result))))
          (compile-exprs (cdr exprs) env new-counter (append acc code)))))))

(defun is-special-form? (name)
  (or (== name "if")
      (== name "defun")
      (== name "let")
      (== name "quote")))

(defun compile-if (args env counter)
  (if (== (list-length args) 3)
    (let ((counter1 (next-label-id counter)))
      (let ((counter2 (next-label-id counter1)))
        (let ((else-label (make-label "ELSE" counter)))
          (let ((end-label (make-label "END" counter1)))
            (let ((result1 (compile-expr (car args) env counter2)))
              (let ((cond-code (car result1)))
                (let ((counter3 (car (cdr result1))))
                  (let ((result2 (compile-expr (list-ref args 1) env counter3)))
                    (let ((then-code (car result2)))
                      (let ((counter4 (car (cdr result2))))
                        (let ((result3 (compile-expr (list-ref args 2) env counter4)))
                          (let ((else-code (car result3)))
                            (let ((counter5 (car (cdr result3))))
                              (let ((code
                                (append cond-code
                                  (append (cons (cons "JmpIfFalse" (cons else-label '())) '())
                                    (append then-code
                                      (append (cons (cons "Jmp" (cons end-label '())) '())
                                        (append (cons (cons "Label" (cons else-label '())) '())
                                          (append else-code
                                            (cons (cons "Label" (cons end-label '())) '())))))))))
                                (cons code (cons counter5 '()))))))))))))))))
    (cons '() (cons counter '()))))

; Compile defun: (defun name (params) body)
; Creates environment with parameters and compiles body
(defun compile-defun (args env counter)
  (if (>= (list-length args) 3)
    (let ((name-ast (car args)))
      (let ((params-ast (list-ref args 1)))
        (let ((body-ast (list-ref args 2)))
          (if (== (car name-ast) "symbol")
            (let ((func-name (car (cdr name-ast))))
              (let ((param-names (extract-param-names params-ast)))
                ; Create new environment with parameters
                (let ((func-env (env-push-frame param-names '() env)))
                  ; Compile body in this environment
                  (let ((result (compile-expr body-ast func-env counter)))
                    (let ((body-code (car result)))
                      (let ((new-counter (car (cdr result))))
                        ; Add Ret instruction and DefineFunction
                        (let ((code (append body-code
                                      (cons (cons "Ret" '())
                                        (cons (cons "DefineFunction" (cons func-name '())) '())))))
                          (cons code (cons new-counter '())))))))))
            (cons '() (cons counter '()))))))
    (cons '() (cons counter '()))))

; Compile let: (let ((var val) ...) body)
; Compiles values, emits SetLocal for each, then compiles body
(defun compile-let (args env counter)
  (if (>= (list-length args) 2)
    (let ((bindings-ast (car args)))
      (let ((body-ast (list-ref args 1)))
        (let ((bindings (extract-let-bindings bindings-ast)))
          (let ((var-names (get-binding-names bindings)))
            ; Compile initialization expressions
            (let ((result (compile-let-bindings bindings env counter 0 '())))
              (let ((init-code (car result)))
                (let ((new-counter (car (cdr result))))
                  ; Create environment with locals
                  (let ((let-env (env-push-frame '() var-names env)))
                    ; Compile body
                    (let ((body-result (compile-expr body-ast let-env new-counter)))
                      (let ((body-code (car body-result)))
                        (let ((final-counter (car (cdr body-result))))
                          ; Combine: init + body + Slide
                          (let ((num-locals (list-length var-names)))
                            (let ((code (append init-code
                                          (append body-code
                                            (cons (cons "Slide" (cons num-locals '())) '())))))
                              (cons code (cons final-counter '())))))))))))))))
    (cons '() (cons counter '()))))

; Compile let bindings: compile each value and emit SetLocal
(defun compile-let-bindings (bindings env counter index acc)
  (if (== bindings '())
    (cons acc (cons counter '()))
    (let ((binding (car bindings)))
      (let ((val-ast (car (cdr binding))))
        (let ((result (compile-expr val-ast env counter)))
          (let ((val-code (car result)))
            (let ((new-counter (car (cdr result))))
              (let ((set-local (cons "SetLocal" (cons index '()))))
                (let ((binding-code (append val-code (cons set-local '()))))
                  (compile-let-bindings (cdr bindings) env new-counter (+ index 1)
                    (append acc binding-code)))))))))))

(defun compile-call (operator args env counter)
  (let ((result (compile-exprs args env counter '())))
    (let ((args-code (car result)))
      (let ((new-counter (car (cdr result))))
        (let ((argc (list-length args)))
          (cons (append args-code
                  (cons (cons "Call" (cons operator (cons argc '()))) '()))
            (cons new-counter '())))))))

(defun compile-list (items env counter)
  (if (== items '())
    (cons '() (cons counter '()))
    (let ((first (car items)))
      (if (== (car first) "symbol")
        (let ((name (car (cdr first))))
          (if (is-special-form? name)
            (if (== name "if")
              (compile-if (cdr items) env counter)
              (if (== name "defun")
                (compile-defun (cdr items) env counter)
                (if (== name "let")
                  (compile-let (cdr items) env counter)
                  (cons '() (cons counter '())))))
            (compile-call name (cdr items) env counter)))
        (cons '() (cons counter '()))))))

(defun compile-expr (expr env counter)
  (let ((type (car expr)))
    (if (== type "number")
      (compile-number (car (cdr expr)) env counter)
      (if (== type "symbol")
        (compile-symbol (car (cdr expr)) env counter)
        (if (== type "list")
          (compile-list (cdr expr) env counter)
          (cons '() (cons counter '())))))))

; === LABEL RESOLUTION ===

(defun labels-equal? (label1 label2)
  (if (== (car label1) (car label2))
    (== (car (cdr label1)) (car (cdr label2)))
    false))

(defun build-label-map-loop (code pos map)
  (if (== code '())
    map
    (let ((instr (car code)))
      (let ((opcode (car instr)))
        (if (== opcode "Label")
          (let ((label (car (cdr instr))))
            (let ((new-map (cons (cons label (cons pos '())) map)))
              (build-label-map-loop (cdr code) pos new-map)))
          (build-label-map-loop (cdr code) (+ pos 1) map))))))

(defun build-label-map (code)
  (build-label-map-loop code 0 '()))

(defun lookup-label (target-label map)
  (if (== map '())
    -999
    (let ((entry (car map)))
      (let ((label (car entry)))
        (let ((addr (car (cdr entry))))
          (if (labels-equal? label target-label)
            addr
            (lookup-label target-label (cdr map))))))))

(defun resolve-labels-loop (code map acc)
  (if (== code '())
    (reverse acc)
    (let ((instr (car code)))
      (let ((opcode (car instr)))
        (if (== opcode "Label")
          (resolve-labels-loop (cdr code) map acc)
          (if (or (== opcode "JmpIfFalse") (== opcode "Jmp"))
            (let ((label (car (cdr instr))))
              (let ((addr (lookup-label label map)))
                (let ((resolved-instr (cons opcode (cons addr '()))))
                  (resolve-labels-loop (cdr code) map (cons resolved-instr acc)))))
            (resolve-labels-loop (cdr code) map (cons instr acc))))))))

(defun resolve-labels (code)
  (let ((label-map (build-label-map code)))
    (resolve-labels-loop code label-map '())))

; === TOP-LEVEL COMPILE ===

(defun compile (parsed-expr)
  (let ((counter (make-label-counter)))
    (let ((env (make-env)))
      (let ((result (compile-expr parsed-expr env counter)))
        (let ((code-with-labels (car result)))
          (resolve-labels code-with-labels))))))

; === TESTS ===

(print "=== Compiler with Complete Environment Tracking ===")
(print "")

(print "Test 1: Simple defun with parameters")
(print "(defun add (x y) (+ x y))")
(let ((ast (cons "list"
  (cons (cons "symbol" (cons "defun" '()))
    (cons (cons "symbol" (cons "add" '()))
      (cons (cons "list"
        (cons (cons "symbol" (cons "x" '()))
          (cons (cons "symbol" (cons "y" '())) '())))
        (cons (cons "list"
          (cons (cons "symbol" (cons "+" '()))
            (cons (cons "symbol" (cons "x" '()))
              (cons (cons "symbol" (cons "y" '())) '()))))
          '())))))))
  (print (compile ast)))

(print "")
(print "Test 2: Let with local variable")
(print "(let ((x 42)) x)")
(let ((ast (cons "list"
  (cons (cons "symbol" (cons "let" '()))
    (cons (cons "list"
      (cons (cons "list"
        (cons (cons "symbol" (cons "x" '()))
          (cons (cons "number" (cons "42" '())) '())))
        '()))
      (cons (cons "symbol" (cons "x" '())) '()))))))
  (print (compile ast)))

(print "")
(print "Test 3: Let with computation")
(print "(let ((x 10) (y 20)) (+ x y))")
(let ((ast (cons "list"
  (cons (cons "symbol" (cons "let" '()))
    (cons (cons "list"
      (cons (cons "list"
        (cons (cons "symbol" (cons "x" '()))
          (cons (cons "number" (cons "10" '())) '())))
        (cons (cons "list"
          (cons (cons "symbol" (cons "y" '()))
            (cons (cons "number" (cons "20" '())) '())))
          '())))
      (cons (cons "list"
        (cons (cons "symbol" (cons "+" '()))
          (cons (cons "symbol" (cons "x" '()))
            (cons (cons "symbol" (cons "y" '())) '()))))
        '()))))))
  (print (compile ast)))
