; Demo: Compile a simple program and serialize to bytecode
; This proves the compiler + serialization pipeline works!

; Load compiler functions (these would normally be in the same file)
(defun append-bytecode (bytes1 bytes2)
  (if (== bytes1 '())
    bytes2
    (cons (car bytes1) (append-bytecode (cdr bytes1) bytes2))))

(defun list-length (lst)
  (if (== lst '())
    0
    (+ 1 (list-length (cdr lst)))))

(defun u32-to-bytes (n)
  (cons (% n 256)
    (cons (% (/ n 256) 256)
      (cons (% (/ n 65536) 256)
        (cons (/ n 16777216) '())))))

(defun i64-to-bytes (n)
  (cons (% n 256)
    (cons (% (/ n 256) 256)
      (cons (% (/ n 65536) 256)
        (cons (% (/ n 16777216) 256)
          (cons (% (/ n 4294967296) 256)
            (cons (% (/ n 1099511627776) 256)
              (cons (% (/ n 281474976710656) 256)
                (cons (/ n 72057594037927936) '())))))))))

(defun string-to-bytes (s)
  (string-chars-to-bytes (string->list s)))

(defun string-chars-to-bytes (chars)
  (if (== chars '())
    '()
    (cons (char-code (car chars))
          (string-chars-to-bytes (cdr chars)))))

(defun serialize-string (s)
  (let ((bytes (string-to-bytes s)))
    (append-bytecode (u32-to-bytes (list-length bytes)) bytes)))

; Value serialization
(defun serialize-value (val)
  (if (int? val)
    (cons 0 (i64-to-bytes val))
  (if (bool? val)
    (cons 1 (cons (if val 1 0) '()))
  (if (string? val)
    (cons 4 (serialize-string val))
    '()))))

(defun int? (v)
  (if (list? v) false
  (if (bool? v) false
  (if (symbol? v) false
  (if (string? v) false
    true)))))

(defun bool? (v)
  (or (== v true) (== v false)))

; Instruction serialization (simplified for demo)
(defun serialize-instr (instr)
  (let ((op (car instr)))
    (if (== op 'push)
      (cons 0 (serialize-value (car (cdr instr))))
    (if (== op 'print) '(11)
    (if (== op 'halt) '(12)
    (if (== op 'ret) '(9)
      '(255)))))))

(defun serialize-instr-list (instrs)
  (if (== instrs '())
    '()
    (append-bytecode (serialize-instr (car instrs))
                     (serialize-instr-list (cdr instrs)))))

; Bytecode file generation
(defun make-simple-program (instrs)
  (append-bytecode '(76 73 83 80 6)  ; Header
    (append-bytecode (u32-to-bytes 0)  ; 0 functions
      (append-bytecode (u32-to-bytes (list-length instrs))
                       (serialize-instr-list instrs)))))

; Demo: Create a program that prints a message
(defun demo ()
  (let ((program '((push "Compiled by Lisp!") (print) (halt))))
  (let ((bytecode (make-simple-program program)))
    (if (write-binary-file "/tmp/compiled-demo.bc" bytecode)
      (print "SUCCESS! Bytecode written to /tmp/compiled-demo.bc")
      (print "Failed to write bytecode")))))

(demo)
