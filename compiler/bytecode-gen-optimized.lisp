; Extended Bytecode Generator with Jump Address Resolution
; Implements two-pass compilation: labels first, then resolve to addresses

; === HELPER FUNCTIONS ===

; Helper: get nth element from list
(defun list-ref (lst n)
  (if (== n 0)
    (car lst)
    (list-ref (cdr lst) (- n 1))))

; Reverse a list (helper for efficient list building)
(defun reverse-helper (lst acc)
  (if (== lst '())
    acc
    (reverse-helper (cdr lst) (cons (car lst) acc))))

(defun reverse (lst)
  (reverse-helper lst '()))

; === LABEL GENERATION ===

; Global counter for unique labels
; Returns a counter starting at 0
(defun make-label-counter ()
  '(0))

; Increment counter and return new counter
(defun next-label-id (counter)
  (let ((id (car counter)))
    (cons (+ id 1) '())))

; Get current counter value
(defun get-label-id (counter)
  (car counter))

; Create unique labels using counter
; Returns (counter-value . prefix-symbol)
; For example: (0 . "ELSE") or (1 . "END")
(defun make-label (prefix counter)
  (cons (get-label-id counter) (cons prefix '())))

; === BYTECODE COMPILATION ===

; Compile a number literal
(defun compile-number (value env counter)
  (cons (cons (cons "Push" (cons value '())) '())
    (cons counter '())))

; Compile a symbol
(defun compile-symbol (name env counter)
  (cons (cons (cons "LoadSymbol" (cons name '())) '())
    (cons counter '())))

; Compile multiple expressions in sequence
(defun compile-exprs (exprs env counter acc)
  (if (== exprs '())
    (cons acc (cons counter '()))
    (let ((result (compile-expr (car exprs) env counter)))
      (let ((code (car result)))
        (let ((new-counter (car (cdr result))))
          (compile-exprs (cdr exprs) env new-counter (append acc code)))))))

; Check if symbol is a special form
(defun is-special-form? (name)
  (or (== name "if")
      (== name "defun")
      (== name "let")
      (== name "quote")))

; Compile if: (if cond then else)
; Generates:
;   <cond-code>
;   JmpIfFalse (else-label)
;   <then-code>
;   Jmp (end-label)
;   Label (else-label)
;   <else-code>
;   Label (end-label)
(defun compile-if (args env counter)
  (if (== (list-length args) 3)
    ; Valid if expression - compile it
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
                              ; Build the complete if bytecode
                              (let ((code
                                (append cond-code
                                  (append (cons (cons "JmpIfFalse" (cons else-label '())) '())
                                    (append then-code
                                      (append (cons (cons "Jmp" (cons end-label '())) '())
                                        (append (cons (cons "Label" (cons else-label '())) '())
                                          (append else-code
                                            (cons (cons "Label" (cons end-label '())) '())))))))))
                                (cons code (cons counter5 '()))))))))))))))))
    ; Invalid if expression
    (cons '() (cons counter '()))))

; Compile quote: (quote expr)
(defun compile-quote (args env counter)
  (cons (cons (cons "PushQuoted" (cons (car args) '())) '())
    (cons counter '())))

; Compile function call
(defun compile-call (operator args env counter)
  (let ((result (compile-exprs args env counter '())))
    (let ((args-code (car result)))
      (let ((new-counter (car (cdr result))))
        (let ((argc (list-length args)))
          (cons (append args-code
                  (cons (cons "Call" (cons operator (cons argc '()))) '()))
            (cons new-counter '())))))))

; Compile list expression
(defun compile-list (items env counter)
  (if (== items '())
    (cons '() (cons counter '()))
    (let ((first (car items)))
      (if (== (car first) "symbol")
        (let ((name (car (cdr first))))
          (if (is-special-form? name)
            ; Handle special forms
            (if (== name "if")
              (compile-if (cdr items) env counter)
              (if (== name "quote")
                (compile-quote (cdr items) env counter)
                ; TODO: defun, let
                (cons '() (cons counter '()))))
            ; Regular function call
            (compile-call name (cdr items) env counter)))
        (cons '() (cons counter '()))))))

; Main compile function with counter
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

; Helper: Check if two labels are equal
; Labels are lists like (0 "ELSE") or (1 "END")
(defun labels-equal? (label1 label2)
  (if (== (car label1) (car label2))
    (== (car (cdr label1)) (car (cdr label2)))
    false))

; Build a map of labels to positions
; Scans bytecode and records position of each Label instruction
(defun build-label-map-loop (code pos map)
  (if (== code '())
    map
    (let ((instr (car code)))
      (let ((opcode (car instr)))
        (if (== opcode "Label")
          ; This is a label definition - add to map
          (let ((label (car (cdr instr))))
            (let ((new-map (cons (cons label (cons pos '())) map)))
              (build-label-map-loop (cdr code) pos new-map)))
          ; Regular instruction - increment position
          (build-label-map-loop (cdr code) (+ pos 1) map))))))

(defun build-label-map (code)
  (build-label-map-loop code 0 '()))

; Lookup label in map
(defun lookup-label (target-label map)
  (if (== map '())
    -999 ; Error value if label not found
    (let ((entry (car map)))
      (let ((label (car entry)))
        (let ((addr (car (cdr entry))))
          (if (labels-equal? label target-label)
            addr
            (lookup-label target-label (cdr map))))))))

; OPTIMIZED: Resolve labels in bytecode using cons + reverse
; This avoids O(n²) behavior from repeated append calls
(defun resolve-labels-loop (code map acc)
  (if (== code '())
    (reverse acc)
    (let ((instr (car code)))
      (let ((opcode (car instr)))
        (if (== opcode "Label")
          ; Skip Label pseudo-instructions
          (resolve-labels-loop (cdr code) map acc)
          (if (or (== opcode "JmpIfFalse") (== opcode "Jmp"))
            ; This is a jump instruction - resolve the label
            (let ((label (car (cdr instr))))
              (let ((addr (lookup-label label map)))
                (let ((resolved-instr (cons opcode (cons addr '()))))
                  ; Use cons instead of append - O(1) operation
                  (resolve-labels-loop (cdr code) map (cons resolved-instr acc)))))
            ; Regular instruction - cons it to accumulator
            (resolve-labels-loop (cdr code) map (cons instr acc))))))))

(defun resolve-labels (code)
  (let ((label-map (build-label-map code)))
    (resolve-labels-loop code label-map '())))

; === TOP-LEVEL COMPILE ===

; Compile with automatic label resolution
(defun compile (parsed-expr)
  (let ((counter (make-label-counter)))
    (let ((result (compile-expr parsed-expr '() counter)))
      (let ((code-with-labels (car result)))
        (resolve-labels code-with-labels)))))

; === TESTS ===

(print "=== Bytecode Generator with Label Resolution ===")
(print "")

(print "Test 1: Simple if expression")
(print "(if true 42 99)")
(let ((test-if
  (cons "list"
    (cons (cons "symbol" (cons "if" '()))
      (cons (cons "symbol" (cons "true" '()))
        (cons (cons "number" (cons "42" '()))
          (cons (cons "number" (cons "99" '())) '())))))))
  (print (compile test-if)))

(print "")
(print "Test 2: If with condition")
(print "(if (> x 0) 1 -1)")
(let ((test-if2
  (cons "list"
    (cons (cons "symbol" (cons "if" '()))
      (cons (cons "list"
        (cons (cons "symbol" (cons ">" '()))
          (cons (cons "symbol" (cons "x" '()))
            (cons (cons "number" (cons "0" '())) '()))))
        (cons (cons "number" (cons "1" '()))
          (cons (cons "number" (cons "-1" '())) '())))))))
  (print (compile test-if2)))

(print "")
(print "Test 3: Nested if expressions")
(print "(if a (if b 1 2) 3)")
(let ((test-nested
  (cons "list"
    (cons (cons "symbol" (cons "if" '()))
      (cons (cons "symbol" (cons "a" '()))
        (cons (cons "list"
          (cons (cons "symbol" (cons "if" '()))
            (cons (cons "symbol" (cons "b" '()))
              (cons (cons "number" (cons "1" '()))
                (cons (cons "number" (cons "2" '())) '())))))
          (cons (cons "number" (cons "3" '())) '())))))))
  (print (compile test-nested)))

(print "")
(print "Test 4: Arithmetic expression (should still work)")
(print "(+ (* 2 3) 4)")
(let ((test-arith
  (cons "list"
    (cons (cons "symbol" (cons "+" '()))
      (cons (cons "list"
        (cons (cons "symbol" (cons "*" '()))
          (cons (cons "number" (cons "2" '()))
            (cons (cons "number" (cons "3" '())) '()))))
        (cons (cons "number" (cons "4" '())) '()))))))
  (print (compile test-arith)))
