; Enhanced Compiler with Desugaring for list, let*, cond, and, or
; This version adds fundamental Lisp primitives to reduce code nesting

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

; Additional helper functions for cleaner code
(defun cadr (lst)
  (car (cdr lst)))

(defun caddr (lst)
  (car (cdr (cdr lst))))

(defun cadddr (lst)
  (car (cdr (cdr (cdr lst)))))

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
    (cadr (car env))))

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
        (let ((name (cadr param)))
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
              (let ((val-ast (cadr binding-items)))
                (if (== (car var-ast) "symbol")
                  (let ((var-name (cadr var-ast)))
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

; === DESUGARING ===

; Desugar (list a b c) to (cons a (cons b (cons c '())))
(defun desugar-list (items)
  (if (== items '())
    (cons "list" '())  ; Empty list AST
    (cons "list"
      (cons (cons "symbol" (cons "cons" '()))
        (cons (car items)
          (cons (desugar-list (cdr items)) '()))))))

; Desugar (let* ((a 1) (b 2)) body) to (let ((a 1)) (let ((b 2)) body))
(defun desugar-let* (bindings-ast body-ast)
  (if (== (car bindings-ast) "list")
    (let ((binding-items (cdr bindings-ast)))
      (if (== binding-items '())
        ; No bindings, just return body
        body-ast
        (if (== (cdr binding-items) '())
          ; Single binding: (let ((var val)) body)
          (cons "list"
            (cons (cons "symbol" (cons "let" '()))
              (cons bindings-ast
                (cons body-ast '()))))
          ; Multiple bindings: (let ((first-var first-val)) (let* (rest...) body))
          (let ((first-binding (car binding-items)))
            (let ((rest-bindings (cons "list" (cdr binding-items))))
              (let ((inner-let* (desugar-let* rest-bindings body-ast)))
                (cons "list"
                  (cons (cons "symbol" (cons "let" '()))
                    (cons (cons "list" (cons first-binding '()))
                      (cons inner-let* '()))))))))))
    ; Invalid bindings-ast, return body
    body-ast))

; Desugar (cond (c1 e1) (c2 e2) (else e3)) to nested ifs
(defun desugar-cond (clauses)
  (if (== clauses '())
    ; No clauses: return false
    (cons "symbol" (cons "false" '()))
    (let ((first-clause (car clauses)))
      (if (== (car first-clause) "list")
        (let ((clause-items (cdr first-clause)))
          (if (>= (list-length clause-items) 2)
            (let ((condition (car clause-items)))
              (let ((result (cadr clause-items)))
                (let ((rest-clauses (cdr clauses)))
                  (if (== rest-clauses '())
                    ; Last clause: if it's "else", return result; otherwise if condition
                    (if (== (car condition) "symbol")
                      (if (== (cadr condition) "else")
                        result
                        (cons "list"
                          (cons (cons "symbol" (cons "if" '()))
                            (cons condition
                              (cons result
                                (cons (cons "symbol" (cons "false" '())) '()))))))
                      (cons "list"
                        (cons (cons "symbol" (cons "if" '()))
                          (cons condition
                            (cons result
                              (cons (cons "symbol" (cons "false" '())) '()))))))
                    ; More clauses: (if c1 e1 (cond rest...))
                    (cons "list"
                      (cons (cons "symbol" (cons "if" '()))
                        (cons condition
                          (cons result
                            (cons (desugar-cond rest-clauses) '())))))))))
            (desugar-cond (cdr clauses))))
        (desugar-cond (cdr clauses))))))

; Desugar (and a b c) to (if a (if b c false) false)
(defun desugar-and (items)
  (if (== items '())
    ; Empty and: return true
    (cons "symbol" (cons "true" '()))
    (if (== (cdr items) '())
      ; Single item: return it
      (car items)
      ; Multiple items: (if a (and b c...) false)
      (cons "list"
        (cons (cons "symbol" (cons "if" '()))
          (cons (car items)
            (cons (desugar-and (cdr items))
              (cons (cons "symbol" (cons "false" '())) '()))))))))

; Desugar (or a b c) to (if a true (if b true c))
(defun desugar-or (items)
  (if (== items '())
    ; Empty or: return false
    (cons "symbol" (cons "false" '()))
    (if (== (cdr items) '())
      ; Single item: return it
      (car items)
      ; Multiple items: (if a true (or b c...))
      (cons "list"
        (cons (cons "symbol" (cons "if" '()))
          (cons (car items)
            (cons (cons "symbol" (cons "true" '()))
              (cons (desugar-or (cdr items)) '()))))))))

; === BYTECODE COMPILATION ===

(defun compile-number (value env counter)
  (cons (cons (list "Push" value) '())
    (cons counter '())))

(defun compile-symbol (name env counter)
  (cons (cons (env-lookup name env) '())
    (cons counter '())))

(defun compile-exprs (exprs env counter acc)
  (if (== exprs '())
    (cons acc (cons counter '()))
    (let ((result (compile-expr (car exprs) env counter)))
      (let ((code (car result)))
        (let ((new-counter (cadr result)))
          (compile-exprs (cdr exprs) env new-counter (append acc code)))))))

(defun is-special-form? (name)
  (or (== name "if")
      (or (== name "defun")
          (or (== name "let")
              (or (== name "quote")
                  (or (== name "list")
                      (or (== name "let*")
                          (or (== name "cond")
                              (or (== name "and")
                                  (== name "or"))))))))))

(defun compile-if (args env counter)
  (if (== (list-length args) 3)
    (let ((counter1 (next-label-id counter)))
      (let ((counter2 (next-label-id counter1)))
        (let ((else-label (make-label "ELSE" counter)))
          (let ((end-label (make-label "END" counter1)))
            (let ((result1 (compile-expr (car args) env counter2)))
              (let ((cond-code (car result1)))
                (let ((counter3 (cadr result1)))
                  (let ((result2 (compile-expr (list-ref args 1) env counter3)))
                    (let ((then-code (car result2)))
                      (let ((counter4 (cadr result2)))
                        (let ((result3 (compile-expr (list-ref args 2) env counter4)))
                          (let ((else-code (car result3)))
                            (let ((counter5 (cadr result3)))
                              (let ((code (append cond-code
                                            (append (list (list "JmpIfFalse" else-label))
                                              (append then-code
                                                (append (list (list "Jmp" end-label))
                                                  (append (list (list "Label" else-label))
                                                    (append else-code
                                                      (list (list "Label" end-label))))))))))
                                (cons code (cons counter5 '())))))))))))))))
    (cons '() (cons counter '()))))

; Compile defun: (defun name (params) body)
; Creates environment with parameters and compiles body
(defun compile-defun (args env counter)
  (if (>= (list-length args) 3)
    (let ((name-ast (car args)))
      (let ((params-ast (list-ref args 1)))
        (let ((body-ast (list-ref args 2)))
          (if (== (car name-ast) "symbol")
            (let ((func-name (cadr name-ast)))
              (let ((param-names (extract-param-names params-ast)))
                (let ((func-env (env-push-frame param-names '() env)))
                  (let ((result (compile-expr body-ast func-env counter)))
                    (let ((body-code (car result)))
                      (let ((new-counter (cadr result)))
                        (let ((code (append body-code
                                      (list (list "Ret")
                                            (list "DefineFunction" func-name)))))
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
            (let ((result (compile-let-bindings bindings env counter 0 '())))
              (let ((init-code (car result)))
                (let ((new-counter (cadr result)))
                  (let ((let-env (env-push-frame '() var-names env)))
                    (let ((body-result (compile-expr body-ast let-env new-counter)))
                      (let ((body-code (car body-result)))
                        (let ((final-counter (cadr body-result)))
                          (let ((num-locals (list-length var-names)))
                            (let ((code (append init-code
                                          (append body-code
                                            (list (list "Slide" num-locals))))))
                              (cons code (cons final-counter '())))))))))))))))
    (cons '() (cons counter '()))))

; Compile let bindings: compile each value and emit SetLocal
(defun compile-let-bindings (bindings env counter index acc)
  (if (== bindings '())
    (cons acc (cons counter '()))
    (let ((binding (car bindings)))
      (let ((val-ast (cadr binding)))
        (let ((result (compile-expr val-ast env counter)))
          (let ((val-code (car result)))
            (let ((new-counter (cadr result)))
              (let ((set-local (list "SetLocal" index)))
                (let ((binding-code (append val-code (list set-local))))
                  (compile-let-bindings (cdr bindings) env new-counter (+ index 1)
                    (append acc binding-code)))))))))))

(defun compile-call (operator args env counter)
  (let ((result (compile-exprs args env counter '())))
    (let ((args-code (car result)))
      (let ((new-counter (cadr result)))
        (let ((argc (list-length args)))
          (cons (append args-code (cons (list "Call" operator argc) '()))
            (cons new-counter '())))))))

(defun compile-list (items env counter)
  (if (== items '())
    (cons '() (cons counter '()))
    (let ((first (car items)))
      (if (== (car first) "symbol")
        (let ((name (cadr first)))
          (if (is-special-form? name)
            (if (== name "if")
              (compile-if (cdr items) env counter)
              (if (== name "defun")
                (compile-defun (cdr items) env counter)
                (if (== name "let")
                  (compile-let (cdr items) env counter)
                  (if (== name "list")
                    (compile-expr (desugar-list (cdr items)) env counter)
                    (if (== name "let*")
                      (if (>= (list-length (cdr items)) 2)
                        (compile-expr (desugar-let* (car (cdr items)) (cadr (cdr items))) env counter)
                        (cons '() (cons counter '())))
                      (if (== name "cond")
                        (compile-expr (desugar-cond (cdr items)) env counter)
                        (if (== name "and")
                          (compile-expr (desugar-and (cdr items)) env counter)
                          (if (== name "or")
                            (compile-expr (desugar-or (cdr items)) env counter)
                            (cons '() (cons counter '()))))))))))
            (compile-call name (cdr items) env counter)))
        (cons '() (cons counter '()))))))

(defun compile-expr (expr env counter)
  (let ((type (car expr)))
    (if (== type "number")
      (compile-number (cadr expr) env counter)
      (if (== type "symbol")
        (compile-symbol (cadr expr) env counter)
        (if (== type "list")
          (compile-list (cdr expr) env counter)
          (cons '() (cons counter '())))))))

; === LABEL RESOLUTION ===

(defun labels-equal? (label1 label2)
  (if (== (car label1) (car label2))
    (== (cadr label1) (cadr label2))
    false))

(defun build-label-map-loop (code pos map)
  (if (== code '())
    map
    (let ((instr (car code)))
      (let ((opcode (car instr)))
        (if (== opcode "Label")
          (let ((label (cadr instr)))
            (let ((new-map (cons (list label pos) map)))
              (build-label-map-loop (cdr code) pos new-map)))
          (build-label-map-loop (cdr code) (+ pos 1) map))))))

(defun build-label-map (code)
  (build-label-map-loop code 0 '()))

(defun lookup-label (target-label map)
  (if (== map '())
    -999
    (let ((entry (car map)))
      (let ((label (car entry)))
        (let ((addr (cadr entry)))
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
            (let ((label (cadr instr)))
              (let ((addr (lookup-label label map)))
                (let ((resolved-instr (list opcode addr)))
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

(print "=== Enhanced Compiler with Desugaring ===")
(print "")

(print "Test 1: list desugaring")
(print "(list 1 2 3)")
(let ((ast (cons "list"
  (cons (cons "symbol" (cons "list" '()))
    (cons (cons "number" (cons "1" '()))
      (cons (cons "number" (cons "2" '()))
        (cons (cons "number" (cons "3" '())) '())))))))
  (print (compile ast)))

(print "")
(print "Test 2: let* with sequential bindings")
(print "(let* ((x 10) (y (+ x 5))) y)")
(let ((ast (cons "list"
  (cons (cons "symbol" (cons "let*" '()))
    (cons (cons "list"
      (cons (cons "list"
        (cons (cons "symbol" (cons "x" '()))
          (cons (cons "number" (cons "10" '())) '())))
        (cons (cons "list"
          (cons (cons "symbol" (cons "y" '()))
            (cons (cons "list"
              (cons (cons "symbol" (cons "+" '()))
                (cons (cons "symbol" (cons "x" '()))
                  (cons (cons "number" (cons "5" '())) '()))))
              '())))
          '())))
      (cons (cons "symbol" (cons "y" '())) '()))))))
  (print (compile ast)))

(print "")
(print "Test 3: cond with multiple clauses")
(print "(cond ((> x 10) 100) ((> x 5) 50) (else 0))")
(let ((ast (cons "list"
  (cons (cons "symbol" (cons "cond" '()))
    (cons (cons "list"
      (cons (cons "list"
        (cons (cons "symbol" (cons ">" '()))
          (cons (cons "symbol" (cons "x" '()))
            (cons (cons "number" (cons "10" '())) '()))))
        (cons (cons "number" (cons "100" '())) '())))
      (cons (cons "list"
        (cons (cons "list"
          (cons (cons "symbol" (cons ">" '()))
            (cons (cons "symbol" (cons "x" '()))
              (cons (cons "number" (cons "5" '())) '()))))
          (cons (cons "number" (cons "50" '())) '())))
        (cons (cons "list"
          (cons (cons "symbol" (cons "else" '()))
            (cons (cons "number" (cons "0" '())) '())))
          '())))))))
  (print (compile ast)))

(print "")
(print "Test 4: and with short-circuit")
(print "(and (> x 0) (< x 100))")
(let ((ast (cons "list"
  (cons (cons "symbol" (cons "and" '()))
    (cons (cons "list"
      (cons (cons "symbol" (cons ">" '()))
        (cons (cons "symbol" (cons "x" '()))
          (cons (cons "number" (cons "0" '())) '()))))
      (cons (cons "list"
        (cons (cons "symbol" (cons "<" '()))
          (cons (cons "symbol" (cons "x" '()))
            (cons (cons "number" (cons "100" '())) '()))))
        '()))))))
  (print (compile ast)))

(print "")
(print "Test 5: or with short-circuit")
(print "(or (< x 0) (> x 100))")
(let ((ast (cons "list"
  (cons (cons "symbol" (cons "or" '()))
    (cons (cons "list"
      (cons (cons "symbol" (cons "<" '()))
        (cons (cons "symbol" (cons "x" '()))
          (cons (cons "number" (cons "0" '())) '()))))
      (cons (cons "list"
        (cons (cons "symbol" (cons ">" '()))
          (cons (cons "symbol" (cons "x" '()))
            (cons (cons "number" (cons "100" '())) '()))))
        '()))))))
  (print (compile ast)))

(print "")
(print "Test 6: Combining new primitives")
(print "(let* ((a 1) (b 2)) (cond ((> a b) (list a b)) (else (list b a))))")
(let ((ast (cons "list"
  (cons (cons "symbol" (cons "let*" '()))
    (cons (cons "list"
      (cons (cons "list"
        (cons (cons "symbol" (cons "a" '()))
          (cons (cons "number" (cons "1" '())) '())))
        (cons (cons "list"
          (cons (cons "symbol" (cons "b" '()))
            (cons (cons "number" (cons "2" '())) '())))
          '())))
      (cons (cons "list"
        (cons (cons "symbol" (cons "cond" '()))
          (cons (cons "list"
            (cons (cons "list"
              (cons (cons "symbol" (cons ">" '()))
                (cons (cons "symbol" (cons "a" '()))
                  (cons (cons "symbol" (cons "b" '())) '()))))
              (cons (cons "list"
                (cons (cons "symbol" (cons "list" '()))
                  (cons (cons "symbol" (cons "a" '()))
                    (cons (cons "symbol" (cons "b" '())) '()))))
                '())))
            (cons (cons "list"
              (cons (cons "symbol" (cons "else" '()))
                (cons (cons "list"
                  (cons (cons "symbol" (cons "list" '()))
                    (cons (cons "symbol" (cons "b" '()))
                      (cons (cons "symbol" (cons "a" '())) '()))))
                  '())))
              '()))))
        '()))))))
  (print (compile ast)))

(print "")
(print "Test 7: Original defun still works")
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
  (print (compile ast))))
