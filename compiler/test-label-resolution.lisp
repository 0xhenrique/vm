; Minimal test for label resolution

; Helper: get nth element from list
(defun list-ref (lst n)
  (if (== n 0)
    (car lst)
    (list-ref (cdr lst) (- n 1))))

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

(defun compile-if-simple (cond-val then-val else-val counter)
  (let ((counter1 (next-label-id counter)))
    (let ((counter2 (next-label-id counter1)))
      (let ((else-label (make-label "ELSE" counter)))
        (let ((end-label (make-label "END" counter1)))
          (let ((cond-code (compile-number cond-val '() counter2)))
            (let ((then-code (compile-number then-val '() counter2)))
              (let ((else-code (compile-number else-val '() counter2)))
                ; Build bytecode with labels
                (let ((code
                  (append (car cond-code)
                    (append (cons (cons "JmpIfFalse" (cons else-label '())) '())
                      (append (car then-code)
                        (append (cons (cons "Jmp" (cons end-label '())) '())
                          (append (cons (cons "Label" (cons else-label '())) '())
                            (append (car else-code)
                              (cons (cons "Label" (cons end-label '())) '())))))))))
                  (cons code (cons counter2 '())))))))))))

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
    acc
    (let ((instr (car code)))
      (let ((opcode (car instr)))
        (if (== opcode "Label")
          (resolve-labels-loop (cdr code) map acc)
          (if (or (== opcode "JmpIfFalse") (== opcode "Jmp"))
            (let ((label (car (cdr instr))))
              (let ((addr (lookup-label label map)))
                (let ((resolved-instr (cons opcode (cons addr '()))))
                  (resolve-labels-loop (cdr code) map (append acc (cons resolved-instr '()))))))
            (resolve-labels-loop (cdr code) map (append acc (cons instr '())))))))))

(defun resolve-labels (code)
  (let ((label-map (build-label-map code)))
    (resolve-labels-loop code label-map '())))

; === TEST ===

(print "=== Testing Label Resolution ===")
(print "")

(print "Test 1: Simple if with numbers")
(print "Generating: if(1) then 42 else 99")

(print "With labels:")
(let ((counter (make-label-counter)))
  (let ((result (compile-if-simple "1" "42" "99" counter)))
    (print (car result))))

(print "")
(print "After resolution:")
(let ((counter (make-label-counter)))
  (let ((result (compile-if-simple "1" "42" "99" counter)))
    (print (resolve-labels (car result)))))
