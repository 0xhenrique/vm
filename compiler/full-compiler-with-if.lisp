; Full Self-Hosted Compiler Pipeline with If Support
; String -> Tokens -> Parsed -> Bytecode (with label resolution)

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

; === TOKENIZER ===

(defun is-digit? (c)
  (let ((n (char-code c)))
    (and (>= n 48) (<= n 57))))

(defun is-space? (c)
  (== (char-code c) 32))

(defun is-letter? (c)
  (let ((n (char-code c)))
    (or (and (>= n 65) (<= n 90))
        (and (>= n 97) (<= n 122)))))

(defun is-symbol-char? (c)
  (let ((n (char-code c)))
    (or (is-letter? c)
        (is-digit? c)
        (== n 43) (== n 45) (== n 42) (== n 47)
        (== n 60) (== n 62) (== n 61)
        (== n 33) (== n 63) (== n 95))))

(defun read-num-acc (chars acc)
  (if (== chars '())
    (cons (list->string acc) (cons '() '()))
    (if (is-digit? (car chars))
      (read-num-acc (cdr chars) (append acc (cons (car chars) '())))
      (cons (list->string acc) (cons chars '())))))

(defun read-symbol-acc (chars acc)
  (if (== chars '())
    (cons (list->string acc) (cons '() '()))
    (if (is-symbol-char? (car chars))
      (read-symbol-acc (cdr chars) (append acc (cons (car chars) '())))
      (cons (list->string acc) (cons chars '())))))

(defun tokenize-loop (chars acc)
  (if (== chars '())
    acc
    (if (is-space? (car chars))
      (tokenize-loop (cdr chars) acc)
      (if (== (car chars) "(")
        (tokenize-loop (cdr chars) (append acc (cons "(" '())))
        (if (== (car chars) ")")
          (tokenize-loop (cdr chars) (append acc (cons ")" '())))
          (if (is-digit? (car chars))
            (let ((res (read-num-acc chars '())))
              (tokenize-loop (car (cdr res)) (append acc (cons (car res) '()))))
            (if (is-symbol-char? (car chars))
              (let ((res (read-symbol-acc chars '())))
                (tokenize-loop (car (cdr res)) (append acc (cons (car res) '()))))
              (tokenize-loop (cdr chars) (append acc (cons "?" '()))))))))))

(defun tokenize (str)
  (tokenize-loop (string->list str) '()))

; === PARSER ===

(defun all-digits? (chars)
  (if (== chars '())
    true
    (let ((c (car chars)))
      (let ((n (char-code c)))
        (if (and (>= n 48) (<= n 57))
          (all-digits? (cdr chars))
          false)))))

(defun is-number-string? (str)
  (if (== str "")
    false
    (all-digits? (string->list str))))

(defun parse-atom (token)
  (if (is-number-string? token)
    (cons (cons "number" (cons token '())) '())
    (cons (cons "symbol" (cons token '())) '())))

(defun parse-list-items (tokens acc)
  (if (== tokens '())
    (cons acc (cons '() '()))
    (if (== (car tokens) ")")
      (cons acc (cons (cdr tokens) '()))
      (let ((parsed (parse-expr tokens)))
        (let ((expr (car parsed)))
          (let ((rest (car (cdr parsed))))
            (parse-list-items rest (append acc (cons expr '())))))))))

(defun parse-expr (tokens)
  (if (== tokens '())
    (cons '() (cons '() '()))
    (if (== (car tokens) "(")
      (let ((result (parse-list-items (cdr tokens) '())))
        (let ((items (car result)))
          (let ((rest (car (cdr result))))
            (cons (cons "list" items) (cons rest '())))))
      (let ((atom-result (parse-atom (car tokens))))
        (let ((expr (car atom-result)))
          (cons expr (cons (cdr tokens) '())))))))

(defun parse (tokens)
  (car (parse-expr tokens)))

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

; === BYTECODE GENERATOR ===

(defun compile-number (value env counter)
  (cons (cons (cons "Push" (cons value '())) '())
    (cons counter '())))

(defun compile-symbol (name env counter)
  (cons (cons (cons "LoadSymbol" (cons name '())) '())
    (cons counter '())))

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

; === FULL PIPELINE ===

(defun compile-with-labels (parsed-expr)
  (let ((counter (make-label-counter)))
    (let ((result (compile-expr parsed-expr '() counter)))
      (car result))))

(defun compile (parsed-expr)
  (resolve-labels (compile-with-labels parsed-expr)))

(defun compile-string (source)
  (compile (parse (tokenize source))))

; === TESTS ===

(print "=== Full Compiler with If Support ===")
(print "")

(print "Test 1: (+ 1 2)")
(print (compile-string "(+ 1 2)"))

(print "")
(print "Test 2: (* 3 4)")
(print (compile-string "(* 3 4)"))

(print "")
(print "Test 3: (+ (* 2 3) 4)")
(print (compile-string "(+ (* 2 3) 4)"))

(print "")
(print "Test 4: (if true 1 0)")
(print (compile-string "(if true 1 0)"))

(print "")
(print "Test 5: (if (> x 5) 100 200)")
(print (compile-string "(if (> x 5) 100 200)"))

(print "")
(print "Test 6: Nested if - (if a (if b 1 2) 3)")
(print (compile-string "(if a (if b 1 2) 3)"))
