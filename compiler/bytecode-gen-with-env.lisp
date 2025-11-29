; Bytecode Generator with Environment Tracking
; Tracks parameters, locals, and globals for proper variable scoping

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

; Environment is a list of frames: ((params) (locals))
; Each frame has two lists: parameters and local variables
; Example: ((("x" "y") ("temp")) (("a") ()))
;   - Current frame has params x,y and local temp
;   - Outer frame has param a and no locals

; Create empty environment
(defun make-env ()
  '())

; Create a new frame and push it onto environment
(defun env-push-frame (params locals env)
  (cons (cons params (cons locals '())) env))

; Get current frame's parameters
(defun env-current-params (env)
  (if (== env '())
    '()
    (car (car env))))

; Get current frame's locals
(defun env-current-locals (env)
  (if (== env '())
    '()
    (car (cdr (car env)))))

; Find index of name in list (returns -1 if not found)
(defun find-index-helper (name lst index)
  (if (== lst '())
    -1
    (if (== (car lst) name)
      index
      (find-index-helper name (cdr lst) (+ index 1)))))

(defun find-index (name lst)
  (find-index-helper name lst 0))

; Lookup variable in environment
; Returns: ("LoadArg" index) or ("GetLocal" index) or ("LoadGlobal" name)
(defun env-lookup (name env)
  (if (== env '())
    ; Not in any frame - it's a global
    (cons "LoadGlobal" (cons name '()))
    ; Check current frame
    (let ((params (env-current-params env)))
      (let ((locals (env-current-locals env)))
        (let ((param-idx (find-index name params)))
          (if (>= param-idx 0)
            ; Found in parameters
            (cons "LoadArg" (cons param-idx '()))
            ; Check locals
            (let ((local-idx (find-index name locals)))
              (if (>= local-idx 0)
                ; Found in locals
                (cons "GetLocal" (cons local-idx '()))
                ; Not in current frame - check outer frames
                ; For now, treat as global (TODO: support closures)
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

; === BYTECODE COMPILATION ===

(defun compile-number (value env counter)
  (cons (cons (cons "Push" (cons value '())) '())
    (cons counter '())))

(defun compile-symbol (name env counter)
  ; Lookup in environment to determine instruction type
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
              (cons '() (cons counter '())))
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

(print "=== Bytecode Generator with Environment Tracking ===")
(print "")

(print "Test 1: Number literal")
(print "(42)")
(let ((ast (cons "number" (cons "42" '()))))
  (print (compile ast)))

(print "")
(print "Test 2: Symbol lookup - should be LoadGlobal")
(print "x")
(let ((ast (cons "symbol" (cons "x" '()))))
  (print (compile ast)))

(print "")
(print "Test 3: Arithmetic with symbols")
(print "(+ x y)")
(let ((ast (cons "list"
  (cons (cons "symbol" (cons "+" '()))
    (cons (cons "symbol" (cons "x" '()))
      (cons (cons "symbol" (cons "y" '())) '()))))))
  (print (compile ast)))

(print "")
(print "Test 4: If expression")
(print "(if x 1 0)")
(let ((ast (cons "list"
  (cons (cons "symbol" (cons "if" '()))
    (cons (cons "symbol" (cons "x" '()))
      (cons (cons "number" (cons "1" '()))
        (cons (cons "number" (cons "0" '())) '())))))))
  (print (compile ast)))

(print "")
(print "Test 5: Lookup x in params (x y) - expect LoadArg 0")
(print (env-lookup "x" (env-push-frame '("x" "y") '() (make-env))))

(print "")
(print "Test 6: Lookup y in params (x y) - expect LoadArg 1")
(print (env-lookup "y" (env-push-frame '("x" "y") '() (make-env))))

(print "")
(print "Test 7: Lookup z (not in env) - expect LoadGlobal z")
(print (env-lookup "z" (env-push-frame '("x" "y") '() (make-env))))

(print "")
(print "Test 8: Lookup temp in locals - expect GetLocal 0")
(print (env-lookup "temp" (env-push-frame '("x") '("temp") (make-env))))
